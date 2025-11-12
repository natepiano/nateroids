use std::collections::VecDeque;
use std::ops::Range;

use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use super::Teleporter;
use super::actor_config::Health;
use super::actor_config::LOCKED_AXES_2D;
use super::actor_config::insert_configured_components;
use super::actor_template::GameLayer;
use super::actor_template::NateroidConfig;
use crate::game_input::just_pressed;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::traits::TransformExt;

// half the size of the boundary and only in the x,y plane
const SPAWN_WINDOW: Vec3 = Vec3::new(0.5, 0.5, 0.0);

#[derive(Resource)]
pub struct NateroidSpawnStats {
    /// Ring buffer tracking last N spawn attempts (true = success, false = failure)
    pub attempts:          VecDeque<bool>,
    pub last_warning_time: f32,
}

impl Default for NateroidSpawnStats {
    fn default() -> Self {
        Self {
            attempts:          VecDeque::with_capacity(50),
            last_warning_time: 0.0,
        }
    }
}

impl NateroidSpawnStats {
    const MAX_ATTEMPTS: usize = 50;

    pub fn record_attempt(&mut self, success: bool) {
        self.attempts.push_back(success);
        if self.attempts.len() > Self::MAX_ATTEMPTS {
            self.attempts.pop_front();
        }
    }

    pub fn success_rate(&self) -> f32 {
        if self.attempts.is_empty() {
            1.0 // No data - assume field is not crowded
        } else {
            let successes = self.attempts.iter().filter(|&&success| success).count();
            successes as f32 / self.attempts.len() as f32
        }
    }

    pub fn attempts_count(&self) -> usize { self.attempts.len() }

    pub fn successes_count(&self) -> usize {
        self.attempts.iter().filter(|&&success| success).count()
    }
}

pub struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NateroidSpawnStats>()
            .add_observer(initialize_nateroid)
            .add_systems(
                Update,
                (
                    spawn_nateroid
                        .in_set(InGameSet::EntityUpdates)
                        .run_if(toggle_active(true, GameAction::SuppressNateroids)),
                    kill_testaroid_on_teleport.in_set(InGameSet::EntityUpdates),
                    spawn_testaroid.in_set(InGameSet::EntityUpdates).run_if(just_pressed(GameAction::SpawnTestaroid)),
                ),
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

#[derive(Component, Debug)]
pub struct Deaderoid {
    pub initial_scale:   Vec3,
    pub target_shrink:   f32,
    pub shrink_duration: f32,
    pub elapsed_time:    f32,
    pub current_shrink:  f32,
}

/// Test nateroid component with configurable spawn position and velocity
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Testaroid {
    pub position: Vec3,
    pub velocity: Vec3,
}

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

fn kill_testaroid_on_teleport(
    mut commands: Commands,
    query: Query<(Entity, &Teleporter), With<Testaroid>>,
) {
    for (entity, teleporter) in query.iter() {
        if teleporter.just_teleported {
            commands.entity(entity).insert(Health(-1.0));
        }
    }
}

fn spawn_testaroid(
    mut commands: Commands,
) {

    let testaroid = Testaroid
        {position:Vec3::new(-159.,-75.,0.),velocity:Vec3::new(-10.,0.,0.)};

    commands.spawn((Nateroid, Name::new("Nateroid"), testaroid ));
}

/// Calculates velocity toward the nearest back wall corner for dying nateroids
fn calculate_death_velocity(position: Vec3, boundary: &Boundary) -> Vec3 {
    let half_size = boundary.transform.scale / 2.0;
    let center = boundary.transform.translation;

    // Four corners of the back wall (negative Z)
    let back_z = center.z - half_size.z;
    let corners = [
        Vec3::new(center.x - half_size.x, center.y - half_size.y, back_z), // Bottom-left
        Vec3::new(center.x + half_size.x, center.y - half_size.y, back_z), // Bottom-right
        Vec3::new(center.x - half_size.x, center.y + half_size.y, back_z), // Top-left
        Vec3::new(center.x + half_size.x, center.y + half_size.y, back_z), // Top-right
    ];

    // Find nearest corner
    let nearest_corner = corners
        .iter()
        .min_by(|a, b| {
            let dist_a = position.distance_squared(**a);
            let dist_b = position.distance_squared(**b);
            dist_a.partial_cmp(&dist_b).unwrap()
        })
        .copied()
        .unwrap_or(Vec3::new(0.0, 0.0, back_z));

    // Calculate direction toward nearest corner
    let direction = (nearest_corner - position).normalize_or_zero();
    direction * 20.0 // Velocity magnitude
}

fn initialize_nateroid(
    nateroid: On<Add, Nateroid>,
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut config: ResMut<NateroidConfig>,
    spatial_query: SpatialQuery,
    mut spawn_stats: ResMut<NateroidSpawnStats>,
    time: Res<Time>,
    test_query: Query<&Testaroid>,
) {
    // Check if this is a testaroid
    if let Ok(testaroid) = test_query.get(nateroid.entity) {
        // Testaroid: spawn with configured position and velocity
        // Dies immediately, death velocity drags portal along wall toward corner
        let scale = config.actor_config.transform.scale;
        let transform = Transform::from_translation(testaroid.position).with_scale(scale);

        commands.entity(nateroid.entity).insert((
            transform,
            LinearVelocity(testaroid.velocity),
            AngularVelocity(Vec3::ZERO),
        ));

        insert_configured_components(&mut commands, &mut config.actor_config, nateroid.entity);

        // Kill immediately so it has approaching portal when it becomes deaderoid
        commands.entity(nateroid.entity).insert(Health(-1.0));
        return;
    }

    // Normal nateroid initialization
    let current_time = time.elapsed_secs();

    let Some(transform) = initialize_transform(&boundary, &config, &spatial_query) else {
        spawn_stats.record_attempt(false);
        commands.entity(nateroid.entity).despawn();

        // Check if we should output warning (once per second)
        if current_time - spawn_stats.last_warning_time >= 1.0 {
            let success_rate = spawn_stats.success_rate() * 100.0;
            warn!(
                "Nateroid spawn: {} / {} attempts ({:.0}%) in the last {} spawns",
                spawn_stats.successes_count(),
                spawn_stats.attempts_count(),
                success_rate,
                spawn_stats.attempts_count()
            );
            spawn_stats.last_warning_time = current_time;
        }
        return;
    };

    spawn_stats.record_attempt(true);

    // Check if we should output stats (once per second, even on success)
    if current_time - spawn_stats.last_warning_time >= 1.0 {
        let success_rate = spawn_stats.success_rate() * 100.0;
        let successes = spawn_stats.successes_count();
        let attempts = spawn_stats.attempts_count();

        // Only warn if there were failures
        if successes < attempts {
            warn!(
                "Nateroid spawn: {} / {} attempts ({:.0}%) in the last {} spawns",
                successes, attempts, success_rate, attempts
            );
        }
        spawn_stats.last_warning_time = current_time;
    }

    // Calculate random velocities for nateroid
    let (linear_velocity, angular_velocity) =
        calculate_nateroid_velocity(config.linear_velocity, config.angular_velocity);

    commands
        .entity(nateroid.entity)
        .insert(transform)
        .insert(linear_velocity)
        .insert(angular_velocity);

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
