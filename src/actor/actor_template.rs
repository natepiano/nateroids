//! this file is separated from actor_spawner just for the convenience of
//! editing default values all the logic is in actor_spawner -
//! we want to use an inspector to change defaults so
//! a new bundle is constructed on each spawn and if the inspector changed
//! anything, it will be reflected in the newly created entity. each of these
//! can be thought of as an ActorConfig
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use super::actor_spawner::ActorConfig;
use super::actor_spawner::ActorKind;
use super::actor_spawner::ColliderType;
use super::actor_spawner::SpawnPosition;
use super::actor_spawner::VelocityBehavior;

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Spaceship,
    Asteroid,
    Missile,
    Boundary,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct MissileConfig(pub ActorConfig);

impl Default for MissileConfig {
    fn default() -> Self {
        Self(ActorConfig {
            actor_kind: ActorKind::Missile,
            collision_damage: 50.,
            collision_layers: CollisionLayers::new([GameLayer::Missile], [GameLayer::Asteroid]),
            health: 1.,
            mass: 0.1,
            rotation: Some(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            spawn_position: SpawnPosition::Missile {
                forward_distance_scalar: 7.0,
            },
            mesh_scalar: 2.5,
            spawn_timer_seconds: Some(1.0 / 20.0),
            velocity_behavior: VelocityBehavior::Missile {
                base_velocity: 85.0,
            },
            ..default()
        })
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct NateroidConfig(pub ActorConfig);

impl Default for NateroidConfig {
    fn default() -> Self {
        Self(ActorConfig {
            actor_kind: ActorKind::Nateroid,
            collider_type: ColliderType::Ball,
            collision_damage: 10.,
            collision_layers: CollisionLayers::new(
                [GameLayer::Asteroid],
                [
                    GameLayer::Asteroid,
                    GameLayer::Missile,
                    GameLayer::Spaceship,
                ],
            ),
            health: 200.,
            mass: 1.0,
            restitution: 0.3,
            spawn_position: SpawnPosition::Asteroid {
                scale_factor: Vec3::new(0.5, 0.5, 0.0),
            },
            velocity_behavior: VelocityBehavior::Nateroid {
                linvel: 30.0,
                angvel: 4.0,
            },
            spawn_timer_seconds: Some(2.),
            ..default()
        })
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct SpaceshipConfig(pub ActorConfig);

impl Default for SpaceshipConfig {
    fn default() -> Self {
        Self(ActorConfig {
            actor_kind: ActorKind::Spaceship,
            collision_damage: 50.,
            collision_layers: CollisionLayers::new(
                [GameLayer::Spaceship],
                [GameLayer::Asteroid, GameLayer::Boundary],
            ),
            health: 500.,
            mass: 10.0,
            locked_axes: LockedAxes::new()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_translation_z(),
            restitution: 0.1,
            // #todo: #handle3d
            rotation: Some(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            mesh_scalar: 0.8,
            spawn_position: SpawnPosition::Spaceship(Vec3::new(0.0, -20.0, 0.0)),
            velocity_behavior: VelocityBehavior::Spaceship(Vec3::ZERO),
            ..default()
        })
    }
}
