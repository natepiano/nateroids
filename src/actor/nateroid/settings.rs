use std::ops::Deref;
use std::ops::DerefMut;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;

use crate::actor::actor_settings::ActorSettings;
use crate::actor::actor_settings::ColliderType;
use crate::actor::actor_settings::Spawnability;
use crate::actor::constants::MAX_NATEROID_ANGULAR_VELOCITY;
use crate::actor::constants::MAX_NATEROID_LINEAR_VELOCITY;
use crate::actor::constants::NATEROID_ANGULAR_DAMPING;
use crate::actor::constants::NATEROID_ANGULAR_VELOCITY;
use crate::actor::constants::NATEROID_COLLIDER_MARGIN;
use crate::actor::constants::NATEROID_COLLISION_DAMAGE;
use crate::actor::constants::NATEROID_DEATH_DURATION_SECS;
use crate::actor::constants::NATEROID_DEATH_SHRINK_PERCENTAGE;
use crate::actor::constants::NATEROID_DENSITY_CULLING_THRESHOLD;
use crate::actor::constants::NATEROID_HEALTH;
use crate::actor::constants::NATEROID_INITIAL_ALPHA;
use crate::actor::constants::NATEROID_LINEAR_DAMPING;
use crate::actor::constants::NATEROID_LINEAR_VELOCITY;
use crate::actor::constants::NATEROID_MASS;
use crate::actor::constants::NATEROID_RESTITUTION;
use crate::actor::constants::NATEROID_SCALE_UP;
use crate::actor::constants::NATEROID_SPAWN_TIMER_SECONDS;
use crate::actor::constants::NATEROID_TARGET_ALPHA;
use crate::actor::game_layer::GameLayer;
use crate::camera::RenderLayer;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCorner {
    Nearest,
    Random,
    Directional,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource)]
pub struct NateroidSettings {
    pub actor_settings:            ActorSettings,
    pub linear_velocity:           f32,
    pub angular_velocity:          f32,
    pub death_duration_secs:       f32,
    pub death_shrink_percentage:   f32,
    pub death_corner:              DeathCorner,
    pub initial_alpha:             f32,
    pub target_alpha:              f32,
    pub density_culling_threshold: f32,
}

impl Default for NateroidSettings {
    fn default() -> Self {
        Self {
            actor_settings:            ActorSettings {
                spawnability:             Spawnability::Enabled,
                angular_damping:          Some(NATEROID_ANGULAR_DAMPING),
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
            death_shrink_percentage:   NATEROID_DEATH_SHRINK_PERCENTAGE,
            death_corner:              DeathCorner::Directional,
            initial_alpha:             NATEROID_INITIAL_ALPHA,
            target_alpha:              NATEROID_TARGET_ALPHA,
            density_culling_threshold: NATEROID_DENSITY_CULLING_THRESHOLD,
        }
    }
}

impl Deref for NateroidSettings {
    type Target = ActorSettings;

    fn deref(&self) -> &Self::Target { &self.actor_settings }
}

impl DerefMut for NateroidSettings {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.actor_settings }
}
