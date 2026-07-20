use std::f32::consts::PI;
use std::ops::Range;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_kana::Position;
use rand::Rng;
use rand::RngExt;
use rand::rng;

use super::Nateroid;
use super::NateroidSettings;
use super::constants::NATEROID_ENTITY_NAME;
use crate::actor::constants::NATEROID_SPAWN_MAX_ATTEMPTS;
use crate::actor::constants::NATEROID_WARN_THROTTLE_INTERVAL_SECS;
use crate::actor::constants::SPAWN_WINDOW;
use crate::actor::game_layer::GameLayer;
use crate::actor::settings;
use crate::actor::settings::ColliderType;
use crate::actor::settings::Spawnability;
use crate::actor::spawn_stats::NateroidSpawnStats;
use crate::actor::spawn_stats::SpawnResult;
use crate::playfield::BoundaryVolume;

pub(super) fn spawn_nateroid(
    mut commands: Commands,
    mut nateroid_settings: ResMut<NateroidSettings>,
    time: Res<Time>,
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    spatial_query: SpatialQuery,
    mut nateroid_spawn_stats: ResMut<NateroidSpawnStats>,
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

    let current_time = time.elapsed_secs();
    let Some(transform) =
        initialize_transform(boundary_transform, &nateroid_settings, &spatial_query)
    else {
        nateroid_spawn_stats.record_attempt(SpawnResult::Failure);

        // `NateroidSpawnStats::last_warning_time` throttles failed-spawn
        // diagnostics to `NATEROID_WARN_THROTTLE_INTERVAL_SECS`.
        if current_time - nateroid_spawn_stats.last_warning_time
            >= NATEROID_WARN_THROTTLE_INTERVAL_SECS
        {
            let success_rate = nateroid_spawn_stats.success_rate() * 100.0;
            warn!(
                "Nateroid spawn: {} / {} attempts ({success_rate:.0}%) in the last {} spawns",
                nateroid_spawn_stats.successes_count(),
                nateroid_spawn_stats.attempts_count(),
                nateroid_spawn_stats.attempts_count()
            );
            nateroid_spawn_stats.last_warning_time = current_time;
        }
        return;
    };

    nateroid_spawn_stats.record_attempt(SpawnResult::Success);

    if current_time - nateroid_spawn_stats.last_warning_time >= NATEROID_WARN_THROTTLE_INTERVAL_SECS
    {
        let success_rate = nateroid_spawn_stats.success_rate() * 100.0;
        let successes = nateroid_spawn_stats.successes_count();
        let attempts = nateroid_spawn_stats.attempts_count();

        if successes < attempts {
            warn!(
                "Nateroid spawn: {successes} / {attempts} attempts ({success_rate:.0}%) in the last {attempts} spawns"
            );
        }
        nateroid_spawn_stats.last_warning_time = current_time;
    }

    let (linear_velocity, angular_velocity) = calculate_nateroid_velocity(
        nateroid_settings.linear_velocity,
        nateroid_settings.angular_velocity,
    );

    commands.spawn_scene(bsn! {
        Nateroid
        template_value(Name::new(NATEROID_ENTITY_NAME))
        template_value(transform)
        template_value(linear_velocity)
        template_value(angular_velocity)
        settings::configured_actor_scene(nateroid_settings.actor_settings.clone())
    });
    nateroid_settings.actor_settings.reset_spawn_timer();
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
    let spatial_query_filter =
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
        let intersections = spatial_query.shape_intersections(
            &spawn_collider,
            *position,
            rotation,
            &spatial_query_filter,
        );

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
    let mut rng = rng();
    let half_scale = bounds.scale.abs() / 2.0;
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
        min
    } else {
        rng.random_range(min.min(max)..=min.max(max))
    }
}

fn get_random_rotation() -> Quat {
    let mut rng = rng();
    Quat::from_euler(
        EulerRot::XYZ,
        rng.random_range(-PI..PI),
        rng.random_range(-PI..PI),
        rng.random_range(-PI..PI),
    )
}

fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rng();
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
