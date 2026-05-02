use std::collections::VecDeque;
use std::ops::Range;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_kana::Position;
use bevy_kana::ToF32;
use rand::Rng;
use rand::RngExt;

use super::Nateroid;
use super::NateroidSettings;
use crate::actor::actor_settings;
use crate::actor::actor_settings::ColliderType;
use crate::actor::actor_settings::Spawnability;
use crate::actor::constants::NATEROID_SPAWN_HISTORY_LEN;
use crate::actor::constants::NATEROID_SPAWN_MAX_ATTEMPTS;
use crate::actor::constants::NATEROID_WARN_THROTTLE_INTERVAL_SECS;
use crate::actor::constants::SPAWN_WINDOW;
use crate::actor::game_layer::GameLayer;
use crate::playfield::BoundaryVolume;

#[derive(Clone, Copy, PartialEq, Eq)]
enum SpawnResult {
    Success,
    Failure,
}

#[derive(Resource)]
pub struct NateroidSpawnStats {
    /// Ring buffer tracking last N spawn attempts
    attempts:          VecDeque<SpawnResult>,
    last_warning_time: f32,
}

impl Default for NateroidSpawnStats {
    fn default() -> Self {
        Self {
            attempts:          VecDeque::with_capacity(NATEROID_SPAWN_HISTORY_LEN),
            last_warning_time: 0.0,
        }
    }
}

impl NateroidSpawnStats {
    fn record_attempt(&mut self, result: SpawnResult) {
        self.attempts.push_back(result);
        if self.attempts.len() > NATEROID_SPAWN_HISTORY_LEN {
            self.attempts.pop_front();
        }
    }

    pub fn success_rate(&self) -> f32 {
        if self.attempts.is_empty() {
            1.0 // No data - assume field is not crowded
        } else {
            let successes = self
                .attempts
                .iter()
                .filter(|&&result| result == SpawnResult::Success)
                .count();
            successes.to_f32() / self.attempts.len().to_f32()
        }
    }

    pub fn attempts_count(&self) -> usize { self.attempts.len() }

    pub fn successes_count(&self) -> usize {
        self.attempts
            .iter()
            .filter(|&&result| result == SpawnResult::Success)
            .count()
    }
}

pub(super) fn spawn_nateroid(
    mut commands: Commands,
    mut nateroid_settings: ResMut<NateroidSettings>,
    time: Res<Time>,
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    spatial_query: SpatialQuery,
    mut spawn_stats: ResMut<NateroidSpawnStats>,
) {
    if nateroid_settings.spawnability == Spawnability::Disabled {
        return;
    }

    let Some(spawn_timer) = nateroid_settings.spawn_timer.as_mut() else {
        return;
    };
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return;
    }

    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    // Pre-validate: only spawn if we can find a valid position
    let current_time = time.elapsed_secs();
    let Some(transform) =
        initialize_transform(boundary_transform, &nateroid_settings, &spatial_query)
    else {
        spawn_stats.record_attempt(SpawnResult::Failure);

        // Check if we should output warning (once per second)
        if current_time - spawn_stats.last_warning_time >= NATEROID_WARN_THROTTLE_INTERVAL_SECS {
            let success_rate = spawn_stats.success_rate() * 100.0;
            warn!(
                "Nateroid spawn: {} / {} attempts ({success_rate:.0}%) in the last {} spawns",
                spawn_stats.successes_count(),
                spawn_stats.attempts_count(),
                spawn_stats.attempts_count()
            );
            spawn_stats.last_warning_time = current_time;
        }
        return;
    };

    spawn_stats.record_attempt(SpawnResult::Success);

    // Check if we should output stats (once per second, even on success)
    if current_time - spawn_stats.last_warning_time >= NATEROID_WARN_THROTTLE_INTERVAL_SECS {
        let success_rate = spawn_stats.success_rate() * 100.0;
        let successes = spawn_stats.successes_count();
        let attempts = spawn_stats.attempts_count();

        // Only warn if there were failures
        if successes < attempts {
            warn!(
                "Nateroid spawn: {successes} / {attempts} attempts ({success_rate:.0}%) in the last {attempts} spawns"
            );
        }
        spawn_stats.last_warning_time = current_time;
    }

    commands.spawn((Nateroid, Name::new("Nateroid"), transform));
}

pub(super) fn initialize_nateroid(
    nateroid: On<Add, Nateroid>,
    mut commands: Commands,
    mut nateroid_settings: ResMut<NateroidSettings>,
) {
    // Normal nateroid: transform already set by `spawn_nateroid`, just add velocities
    let (linear_velocity, angular_velocity) = calculate_nateroid_velocity(
        nateroid_settings.linear_velocity,
        nateroid_settings.angular_velocity,
    );

    commands
        .entity(nateroid.entity)
        .insert(linear_velocity)
        .insert(angular_velocity);

    actor_settings::insert_configured_components(
        &mut commands,
        &mut nateroid_settings.actor_settings,
        nateroid.entity,
    );
}

fn initialize_transform(
    boundary_transform: &Transform,
    nateroid_settings: &NateroidSettings,
    spatial_query: &SpatialQuery,
) -> Option<Transform> {
    let bounds = Transform {
        translation: boundary_transform.translation,
        scale: boundary_transform.scale * SPAWN_WINDOW,
        ..default()
    };

    let scale = nateroid_settings.actor_settings.transform.scale;
    let filter =
        SpatialQueryFilter::from_mask(LayerMask::from([GameLayer::Spaceship, GameLayer::Asteroid]));

    for _ in 0..NATEROID_SPAWN_MAX_ATTEMPTS {
        let position = get_random_position_within_bounds(&bounds);
        let rotation = get_random_rotation();

        // Approximate collider for spawn overlap check — the real collider is
        // computed from child AABBs after the entity spawns.
        let spawn_collider = match nateroid_settings.actor_settings.collider_type {
            ColliderType::Ball => {
                Collider::sphere(nateroid_settings.actor_settings.collider_margin)
            },
            ColliderType::Cuboid => {
                let margin = nateroid_settings.actor_settings.collider_margin;
                Collider::cuboid(margin, margin, margin)
            },
        };
        let intersections =
            spatial_query.shape_intersections(&spawn_collider, *position, rotation, &filter);

        if intersections.is_empty() {
            return Some(Transform {
                translation: *position,
                rotation,
                scale,
            });
        }
    }

    None
}

fn get_random_position_within_bounds(bounds: &Transform) -> Position {
    let mut rng = rand::rng();
    let half_scale = bounds.scale.abs() / 2.0; // Use absolute value to ensure positive scale
    let min = bounds.translation - half_scale;
    let max = bounds.translation + half_scale;

    Position::new(
        get_random_component(min.x, max.x, &mut rng),
        get_random_component(min.y, max.y, &mut rng),
        get_random_component(min.z, max.z, &mut rng),
    )
}

fn get_random_component(min: f32, max: f32, rng: &mut impl Rng) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        min // If the range is effectively zero, just return the min value
    } else {
        rng.random_range(min.min(max)..=min.max(max)) // Ensure min is always less than max
    }
}

fn get_random_rotation() -> Quat {
    let mut rng = rand::rng();
    Quat::from_euler(
        EulerRot::XYZ,
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
    )
}

fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rand::rng();
    let x = if range_x.start < range_x.end {
        rng.random_range(range_x)
    } else {
        0.0
    };
    let y = if range_y.start < range_y.end {
        rng.random_range(range_y)
    } else {
        0.0
    };
    let z = if range_z.start < range_z.end {
        rng.random_range(range_z)
    } else {
        0.0
    };

    Vec3::new(x, y, z)
}

fn calculate_nateroid_velocity(
    linear_velocity: f32,
    angular_velocity: f32,
) -> (LinearVelocity, AngularVelocity) {
    (
        LinearVelocity(random_vec3(
            -linear_velocity..linear_velocity,
            -linear_velocity..linear_velocity,
            0.0..0.0,
        )),
        AngularVelocity(random_vec3(
            -angular_velocity..angular_velocity,
            -angular_velocity..angular_velocity,
            -angular_velocity..angular_velocity,
        )),
    )
}
