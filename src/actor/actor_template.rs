//! this file is separated from actor_spawner just for the convenience of
//! editing default values all the logic is in actor_spawner -
//! we want to use an inspector to change defaults so
//! a new bundle is constructed on each spawn and if the inspector changed
//! anything, it will be reflected in the newly created entity. each of these
//! can be thought of as an ActorConfig
use std::ops::Deref;
use std::ops::DerefMut;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use super::actor_spawner::ActorConfig;
use super::actor_spawner::ActorKind;
use super::actor_spawner::ColliderType;

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
pub struct MissileConfig {
    pub actor_config:            ActorConfig,
    pub forward_distance_scalar: f32,
    pub base_velocity:           f32,
}

impl Default for MissileConfig {
    fn default() -> Self {
        Self {
            actor_config:            ActorConfig {
                actor_kind: ActorKind::Missile,
                collision_damage: 50.,
                collision_layers: CollisionLayers::new([GameLayer::Missile], [GameLayer::Asteroid]),
                health: 1.,
                mass: 0.1,
                spawn_timer_seconds: Some(1.0 / 20.0),
                transform: Transform::from_rotation(Quat::from_rotation_x(
                    std::f32::consts::FRAC_PI_2,
                ))
                .with_scale(Vec3::splat(2.5)),
                ..default()
            },
            forward_distance_scalar: 7.0,
            base_velocity:           85.0,
        }
    }
}

impl Deref for MissileConfig {
    type Target = ActorConfig;

    fn deref(&self) -> &Self::Target { &self.actor_config }
}

impl DerefMut for MissileConfig {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.actor_config }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct NateroidConfig {
    pub actor_config: ActorConfig,
    pub linvel:       f32,
    pub angvel:       f32,
}

impl Default for NateroidConfig {
    fn default() -> Self {
        Self {
            actor_config: ActorConfig {
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
                spawn_timer_seconds: Some(2.),
                ..default()
            },
            linvel:       30.0,
            angvel:       4.0,
        }
    }
}

impl Deref for NateroidConfig {
    type Target = ActorConfig;

    fn deref(&self) -> &Self::Target { &self.actor_config }
}

impl DerefMut for NateroidConfig {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.actor_config }
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
            transform: Transform::from_translation(Vec3::new(0.0, -20.0, 0.0))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(0.8)),
            ..default()
        })
    }
}

impl Deref for SpaceshipConfig {
    type Target = ActorConfig;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for SpaceshipConfig {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
