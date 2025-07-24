/// this file is separated from actor_spawner just for the convenience of
/// editing default values all the logic is in actor_spawner the reason we
/// don't only use a bundle is we want to use an inspector to change defaults so
/// a new bundle is constructed on each spawn and if the inspector changed
/// anything, it will be reflected in the newly created entity. each of these
/// can be thought of as an ActorConfig
use crate::actor::{
    ColliderType,
    actor_spawner::{
        ActorConfig,
        ActorKind,
        SpawnPositionBehavior,
        VelocityBehavior,
    },
};
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_rapier3d::{
    dynamics::LockedAxes,
    geometry::Group,
    prelude::CollisionGroups,
};

pub const GROUP_SPACESHIP: Group = Group::GROUP_1;
pub const GROUP_ASTEROID: Group = Group::GROUP_2;
pub const GROUP_MISSILE: Group = Group::GROUP_3;

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct MissileConfig(pub ActorConfig);

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct NateroidConfig(pub ActorConfig);

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct SpaceshipConfig(pub ActorConfig);

// todo: #rustquestion - why isn't rustfmt lining these up? it does if i get of
// default
impl Default for MissileConfig {
    fn default() -> Self {
        Self(ActorConfig {
            actor_kind: ActorKind::Missile,
            collision_damage: 50.,
            collision_groups: CollisionGroups::new(GROUP_MISSILE, GROUP_ASTEROID),
            health: 1.,
            mass: 0.1,
            // #todo: #handle3d
            rotation: Some(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            spawn_position_behavior: SpawnPositionBehavior::ForwardFromParent { distance: 0.5 },
            scalar: 2.5,
            spawn_timer_seconds: Some(1.0 / 20.0),
            velocity_behavior: VelocityBehavior::RelativeToParent {
                base_velocity:           85.0,
                inherit_parent_velocity: true,
            },
            ..default()
        })
    }
}

impl Default for NateroidConfig {
    fn default() -> Self {
        Self(ActorConfig {
            actor_kind: ActorKind::Nateroid,
            collider_type: ColliderType::Cuboid,
            collision_damage: 10.,
            health: 200.,
            mass: 1.0,
            restitution: 0.3,
            spawn_position_behavior: SpawnPositionBehavior::RandomWithinBounds {
                scale_factor: Vec3::new(0.5, 0.5, 0.0),
            },
            velocity_behavior: VelocityBehavior::Random {
                linvel: 30.0,
                angvel: 4.0,
            },
            spawn_timer_seconds: Some(2.),
            ..default()
        })
    }
}

impl Default for SpaceshipConfig {
    fn default() -> Self {
        Self(ActorConfig {
            actor_kind: ActorKind::Spaceship,
            collision_damage: 50.,
            collision_groups: CollisionGroups::new(GROUP_SPACESHIP, GROUP_ASTEROID),
            health: 500.,
            mass: 10.0,
            locked_axes: LockedAxes::ROTATION_LOCKED_X
                | LockedAxes::ROTATION_LOCKED_Y
                | LockedAxes::TRANSLATION_LOCKED_Z,
            restitution: 0.1,
            // #todo: #handle3d
            rotation: Some(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            scalar: 0.8,
            spawn_position_behavior: SpawnPositionBehavior::Fixed(Vec3::new(0.0, -20.0, 0.0)),
            velocity_behavior: VelocityBehavior::Fixed(Vec3::ZERO),
            ..default()
        })
    }
}
