use std::ops::Range;

use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use super::Teleporter;
use super::actor_config::LOCKED_AXES_2D;
use super::actor_config::insert_configured_components;
use super::actor_template::GameLayer;
use super::actor_template::NateroidConfig;
use super::spaceship::Spaceship;
use crate::global_input::GameAction;
use crate::global_input::toggle_active;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::traits::TransformExt;

// half the size of the boundary and only in the x,y plane
const SPAWN_WINDOW: Vec3 = Vec3::new(0.5, 0.5, 0.0);
// Maximum allowed velocities to prevent physics explosions.
// Lower limits needed with SubstepCount(1) to prevent popcorn bursts.
pub const MAX_NATEROID_LINEAR_VELOCITY: f32 = 80.0;
pub const MAX_NATEROID_ANGULAR_VELOCITY: f32 = 20.0;

#[derive(Resource, Default)]
struct NateroidSpawnStats {
    attempts:          u32,
    successes:         u32,
    last_warning_time: f32,
}

pub struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NateroidSpawnStats>()
            .add_observer(initialize_nateroid)
            .add_systems(
                Update,
                spawn_nateroid
                    .in_set(InGameSet::EntityUpdates)
                    .run_if(toggle_active(true, GameAction::SuppressNateroids)),
            )
            .add_systems(
                FixedUpdate,
                clamp_nateroid_velocity
                    .after(PhysicsSystems::StepSimulation)
                    .in_set(InGameSet::EntityUpdates),
            );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Nateroid;

fn spawn_nateroid(mut commands: Commands, mut config: ResMut<NateroidConfig>, time: Res<Time>) {
    if !config.spawnable {
        return;
    }

    let spawn_timer = config.spawn_timer.as_mut().unwrap();
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return;
    }

    commands.spawn((Nateroid, Name::new("Nateroid")));
}

fn initialize_nateroid(
    nateroid: On<Add, Nateroid>,
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut config: ResMut<NateroidConfig>,
    spatial_query: SpatialQuery,
    mut spawn_stats: ResMut<NateroidSpawnStats>,
    time: Res<Time>,
) {
    spawn_stats.attempts += 1;
    let current_time = time.elapsed_secs();

    let Some(transform) = initialize_transform(&boundary, &config, &spatial_query) else {
        commands.entity(nateroid.entity).despawn();

        // Check if we should output warning (once per second)
        if current_time - spawn_stats.last_warning_time >= 1.0 {
            let success_rate = if spawn_stats.attempts > 0 {
                (spawn_stats.successes as f32 / spawn_stats.attempts as f32) * 100.0
            } else {
                0.0
            };
            warn!(
                "Nateroid spawn: {} / {} attempts ({:.0}%) in the last second",
                spawn_stats.successes, spawn_stats.attempts, success_rate
            );
            spawn_stats.attempts = 0;
            spawn_stats.successes = 0;
            spawn_stats.last_warning_time = current_time;
        }
        return;
    };

    spawn_stats.successes += 1;

    // Check if we should output stats (once per second, even on success)
    if current_time - spawn_stats.last_warning_time >= 1.0 {
        let success_rate = if spawn_stats.attempts > 0 {
            (spawn_stats.successes as f32 / spawn_stats.attempts as f32) * 100.0
        } else {
            0.0
        };
        // Only warn if there were failures
        if spawn_stats.successes < spawn_stats.attempts {
            warn!(
                "Nateroid spawn: {} / {} attempts ({:.0}%) in the last second",
                spawn_stats.successes, spawn_stats.attempts, success_rate
            );
        }
        spawn_stats.attempts = 0;
        spawn_stats.successes = 0;
        spawn_stats.last_warning_time = current_time;
    }

    // Calculate random velocities for nateroid
    let (linear_velocity, angular_velocity) =
        calculate_nateroid_velocity(config.linear_velocity, config.angular_velocity);

    commands
        .entity(nateroid.entity)
        .insert(transform)
        .insert(linear_velocity)
        .insert(angular_velocity)
        .insert(MaxLinearSpeed(MAX_NATEROID_LINEAR_VELOCITY))
        .insert(MaxAngularSpeed(MAX_NATEROID_ANGULAR_VELOCITY));

    insert_configured_components(&mut commands, &mut config.actor_config, nateroid.entity);
}

fn initialize_transform(
    boundary: &Boundary,
    nateroid_config: &NateroidConfig,
    spatial_query: &SpatialQuery,
) -> Option<Transform> {
    const MAX_ATTEMPTS: u32 = 20;

    let bounds = Transform {
        translation: boundary.transform.translation,
        scale: boundary.transform.scale * SPAWN_WINDOW,
        ..default()
    };

    let scale = nateroid_config.actor_config.transform.scale;
    let filter =
        SpatialQueryFilter::from_mask(LayerMask::from([GameLayer::Spaceship, GameLayer::Asteroid]));

    for _ in 0..MAX_ATTEMPTS {
        let position = get_random_position_within_bounds(&bounds);
        let rotation = get_random_rotation();

        let intersections = spatial_query.shape_intersections(
            &nateroid_config.actor_config.collider,
            position,
            rotation,
            &filter,
        );

        if intersections.is_empty() {
            return Some(Transform::from_trs(position, rotation, scale));
        }
    }

    None
}

fn get_random_position_within_bounds(bounds: &Transform) -> Vec3 {
    let mut rng = rand::rng();
    let half_scale = bounds.scale.abs() / 2.0; // Use absolute value to ensure positive scale
    let min = bounds.translation - half_scale;
    let max = bounds.translation + half_scale;

    Vec3::new(
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

fn calculate_nateroid_velocity(linvel: f32, angvel: f32) -> (LinearVelocity, AngularVelocity) {
    (
        LinearVelocity(random_vec3(-linvel..linvel, -linvel..linvel, 0.0..0.0)),
        AngularVelocity(random_vec3(
            -angvel..angvel,
            -angvel..angvel,
            -angvel..angvel,
        )),
    )
}

/// Clamp velocities to prevent physics explosions from collision accumulation
fn clamp_nateroid_velocity(
    mut entities: Query<
        (&mut LinearVelocity, &mut AngularVelocity),
        Or<(With<Nateroid>, With<Spaceship>)>,
    >,
) {
    for (mut linear_velocity, mut angular_velocity) in entities.iter_mut() {
        // Clamp linear velocity
        let linear_speed = linear_velocity.length();
        if linear_speed > MAX_NATEROID_LINEAR_VELOCITY {
            **linear_velocity = linear_velocity.normalize() * MAX_NATEROID_LINEAR_VELOCITY;
        }

        // Clamp angular velocity
        let angular_speed = angular_velocity.length();
        if angular_speed > MAX_NATEROID_ANGULAR_VELOCITY {
            **angular_velocity = angular_velocity.normalize() * MAX_NATEROID_ANGULAR_VELOCITY;
        }
    }
}
