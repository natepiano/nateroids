//! this file is separated from `actor_spawner` just for the convenience of
//! editing default values all the logic is in `actor_spawner` -
//! we want to use an inspector to change defaults so
//! a new bundle is constructed on each spawn and if the inspector changed
//! anything, it will be reflected in the newly created entity. each of these
//! can be thought of as an `ActorConfig`
use std::ops::Deref;
use std::ops::DerefMut;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use super::Aabb;
use super::actor_config::ActorConfig;
use super::actor_config::ColliderType;
use super::actor_config::GLTF_ROTATION_X;
use super::constants::MAX_MISSILE_ANGULAR_VELOCITY;
use super::constants::MAX_MISSILE_LINEAR_VELOCITY;
use super::constants::MAX_NATEROID_ANGULAR_VELOCITY;
use super::constants::MAX_NATEROID_LINEAR_VELOCITY;
use super::constants::MAX_SPACESHIP_ANGULAR_VELOCITY;
use super::constants::MAX_SPACESHIP_LINEAR_VELOCITY;
use crate::camera::RenderLayer;
use crate::traits::TransformExt;

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
                spawnable:                true,
                aabb:                     Aabb::default(),
                angular_damping:          None,
                collider:                 Collider::cuboid(1., 1., 1.),
                collider_margin:          1.0,
                collider_type:            ColliderType::Cuboid,
                collision_damage:         50.,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Missile],
                    [GameLayer::Asteroid],
                ),
                gravity_scale:            0.,
                health:                   1.,
                linear_damping:           None,
                locked_axes:              LockedAxes::new().lock_translation_z(),
                mass:                     0.1,
                max_angular_velocity:     MAX_MISSILE_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_MISSILE_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              0.1,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      Some(1.0 / 20.0),
                transform:                Transform::from_rotation(
                    Quat::from_rotation_x(GLTF_ROTATION_X)
                        * Quat::from_rotation_z(std::f32::consts::PI),
                )
                .with_scale(Vec3::splat(2.5)),
                spawn_timer:              None,
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

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCorner {
    Nearest,
    Random,
    Directional,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct NateroidConfig {
    pub actor_config:              ActorConfig,
    pub linear_velocity:           f32,
    pub angular_velocity:          f32,
    pub death_duration_secs:       f32,
    pub death_shrink_pct:          f32,
    pub death_corner:              DeathCorner,
    pub initial_alpha:             f32,
    pub target_alpha:              f32,
    pub density_culling_threshold: f32,
}

impl Default for NateroidConfig {
    fn default() -> Self {
        Self {
            actor_config:              ActorConfig {
                spawnable:                true,
                aabb:                     Aabb::default(),
                angular_damping:          Some(0.001),
                collider:                 Collider::cuboid(1., 1., 1.),
                collider_margin:          1.0 / 3.0,
                collider_type:            ColliderType::Ball,
                collision_damage:         10.,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Asteroid],
                    [
                        GameLayer::Asteroid,
                        GameLayer::Missile,
                        GameLayer::Spaceship,
                    ],
                ),
                gravity_scale:            0.,
                health:                   200.,
                linear_damping:           Some(0.001),
                locked_axes:              LockedAxes::new().lock_translation_z(),
                mass:                     1.0,
                max_angular_velocity:     MAX_NATEROID_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_NATEROID_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              0.3,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      Some(2.0),
                transform:                Transform::default(),
                spawn_timer:              None,
            },
            linear_velocity:           35.0,
            angular_velocity:          4.5,
            death_duration_secs:       3.,
            death_shrink_pct:          0.3,
            death_corner:              DeathCorner::Directional,
            initial_alpha:             0.35,
            target_alpha:              0.05,
            density_culling_threshold: 0.01,
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
pub struct SpaceshipConfig {
    pub actor_config: ActorConfig,
}

impl Default for SpaceshipConfig {
    fn default() -> Self {
        Self {
            actor_config: ActorConfig {
                spawnable:                true,
                aabb:                     Aabb::default(),
                angular_damping:          Some(0.1),
                collider:                 Collider::cuboid(1., 1., 1.),
                collider_margin:          1.0,
                collider_type:            ColliderType::Cuboid,
                collision_damage:         50.,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Spaceship],
                    [GameLayer::Asteroid, GameLayer::Boundary],
                ),
                gravity_scale:            0.,
                health:                   5000.,
                linear_damping:           Some(0.05),
                locked_axes:              LockedAxes::new()
                    .lock_rotation_x()
                    .lock_rotation_y()
                    .lock_translation_z(),
                mass:                     10.0,
                max_angular_velocity:     MAX_SPACESHIP_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_SPACESHIP_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              0.1,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      None,
                transform:                Transform::from_trs(
                    Vec3::new(0.0, -20.0, 0.0),
                    Quat::from_rotation_x(GLTF_ROTATION_X),
                    Vec3::splat(2.0),
                ),
                spawn_timer:              None,
            },
        }
    }
}

impl Deref for SpaceshipConfig {
    type Target = ActorConfig;

    fn deref(&self) -> &Self::Target { &self.actor_config }
}

impl DerefMut for SpaceshipConfig {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.actor_config }
}
