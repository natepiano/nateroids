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
use super::constants::MISSILE_BASE_VELOCITY;
use super::constants::MISSILE_COLLIDER_MARGIN;
use super::constants::MISSILE_COLLISION_DAMAGE;
use super::constants::MISSILE_FORWARD_DISTANCE_SCALAR;
use super::constants::MISSILE_HEALTH;
use super::constants::MISSILE_MASS;
use super::constants::MISSILE_RESTITUTION;
use super::constants::MISSILE_SCALE;
use super::constants::MISSILE_SPAWN_TIMER_SECONDS;
use super::constants::NATEROID_ANGULAR_DAMPING;
use super::constants::NATEROID_ANGULAR_VELOCITY;
use super::constants::NATEROID_COLLIDER_MARGIN;
use super::constants::NATEROID_COLLISION_DAMAGE;
use super::constants::NATEROID_DEATH_DURATION_SECS;
use super::constants::NATEROID_DEATH_SHRINK_PCT;
use super::constants::NATEROID_DENSITY_CULLING_THRESHOLD;
use super::constants::NATEROID_HEALTH;
use super::constants::NATEROID_INITIAL_ALPHA;
use super::constants::NATEROID_LINEAR_DAMPING;
use super::constants::NATEROID_LINEAR_VELOCITY;
use super::constants::NATEROID_MASS;
use super::constants::NATEROID_RESTITUTION;
use super::constants::NATEROID_SCALE_UP;
use super::constants::NATEROID_SPAWN_TIMER_SECONDS;
use super::constants::NATEROID_TARGET_ALPHA;
use super::constants::SPACESHIP_ANGULAR_DAMPING;
use super::constants::SPACESHIP_COLLIDER_MARGIN;
use super::constants::SPACESHIP_COLLISION_DAMAGE;
use super::constants::SPACESHIP_HEALTH;
use super::constants::SPACESHIP_INITIAL_POSITION;
use super::constants::SPACESHIP_LINEAR_DAMPING;
use super::constants::SPACESHIP_MASS;
use super::constants::SPACESHIP_RESTITUTION;
use super::constants::SPACESHIP_SCALE;
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
                collider_margin:          MISSILE_COLLIDER_MARGIN,
                collider_type:            ColliderType::Cuboid,
                collision_damage:         MISSILE_COLLISION_DAMAGE,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Missile],
                    [GameLayer::Asteroid],
                ),
                gravity_scale:            0.,
                health:                   MISSILE_HEALTH,
                linear_damping:           None,
                locked_axes:              LockedAxes::new().lock_translation_z(),
                mass:                     MISSILE_MASS,
                max_angular_velocity:     MAX_MISSILE_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_MISSILE_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              MISSILE_RESTITUTION,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      Some(MISSILE_SPAWN_TIMER_SECONDS),
                transform:                Transform::from_rotation(
                    Quat::from_rotation_x(GLTF_ROTATION_X)
                        * Quat::from_rotation_z(std::f32::consts::PI),
                )
                .with_scale(Vec3::splat(MISSILE_SCALE)),
                spawn_timer:              None,
            },
            forward_distance_scalar: MISSILE_FORWARD_DISTANCE_SCALAR,
            base_velocity:           MISSILE_BASE_VELOCITY,
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
                angular_damping:          Some(NATEROID_ANGULAR_DAMPING),
                collider:                 Collider::cuboid(1., 1., 1.),
                collider_margin:          NATEROID_COLLIDER_MARGIN,
                collider_type:            ColliderType::Ball,
                collision_damage:         NATEROID_COLLISION_DAMAGE,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Asteroid],
                    [
                        GameLayer::Asteroid,
                        GameLayer::Missile,
                        GameLayer::Spaceship,
                    ],
                ),
                gravity_scale:            0.,
                health:                   NATEROID_HEALTH,
                linear_damping:           Some(NATEROID_LINEAR_DAMPING),
                locked_axes:              LockedAxes::new().lock_translation_z(),
                mass:                     NATEROID_MASS,
                max_angular_velocity:     MAX_NATEROID_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_NATEROID_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              NATEROID_RESTITUTION,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      Some(NATEROID_SPAWN_TIMER_SECONDS),
                transform:                Transform::from_scale(Vec3::splat(NATEROID_SCALE_UP)),
                spawn_timer:              None,
            },
            linear_velocity:           NATEROID_LINEAR_VELOCITY,
            angular_velocity:          NATEROID_ANGULAR_VELOCITY,
            death_duration_secs:       NATEROID_DEATH_DURATION_SECS,
            death_shrink_pct:          NATEROID_DEATH_SHRINK_PCT,
            death_corner:              DeathCorner::Directional,
            initial_alpha:             NATEROID_INITIAL_ALPHA,
            target_alpha:              NATEROID_TARGET_ALPHA,
            density_culling_threshold: NATEROID_DENSITY_CULLING_THRESHOLD,
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
                angular_damping:          Some(SPACESHIP_ANGULAR_DAMPING),
                collider:                 Collider::cuboid(1., 1., 1.),
                collider_margin:          SPACESHIP_COLLIDER_MARGIN,
                collider_type:            ColliderType::Cuboid,
                collision_damage:         SPACESHIP_COLLISION_DAMAGE,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Spaceship],
                    [GameLayer::Asteroid, GameLayer::Boundary],
                ),
                gravity_scale:            0.,
                health:                   SPACESHIP_HEALTH,
                linear_damping:           Some(SPACESHIP_LINEAR_DAMPING),
                locked_axes:              LockedAxes::new()
                    .lock_rotation_x()
                    .lock_rotation_y()
                    .lock_translation_z(),
                mass:                     SPACESHIP_MASS,
                max_angular_velocity:     MAX_SPACESHIP_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_SPACESHIP_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              SPACESHIP_RESTITUTION,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      None,
                transform:                Transform::from_trs(
                    SPACESHIP_INITIAL_POSITION,
                    Quat::from_rotation_x(GLTF_ROTATION_X),
                    Vec3::splat(SPACESHIP_SCALE),
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
