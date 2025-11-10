use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actor::Deaderoid;
use crate::actor::Health;
use crate::actor::MissilePosition;
use crate::actor::Nateroid;
use crate::actor::actor_template::NateroidConfig;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::state::GameState;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                despawn_dead_entities,
                despawn_missiles,
                animate_dying_nateroids,
            )
                .in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnEnter(GameState::Splash), despawn_all_entities)
        .add_systems(OnEnter(GameState::GameOver), despawn_all_entities)
        .add_systems(OnExit(GameState::Splash), despawn_splash);
    }
}

fn despawn_missiles(mut commands: Commands, query: Query<(Entity, &MissilePosition)>) {
    for (entity, missile) in query.iter() {
        if missile.traveled_distance >= missile.total_distance {
            despawn(&mut commands, entity);
        }
    }
}

/// Uses `try_despawn` because entities can be queued for despawn multiple times in a frame
/// (e.g., missile reaching max distance AND taking lethal damage simultaneously)
pub fn despawn(commands: &mut Commands, entity: Entity) { commands.entity(entity).try_despawn(); }

/// Calculates velocity toward the nearest back wall corner.
/// Back wall has 4 corners at -Z boundary.
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

#[allow(clippy::type_complexity)]
fn despawn_dead_entities(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Health,
            &Transform,
            Option<&Nateroid>,
            Option<&Name>,
        ),
        Without<Deaderoid>,
    >,
    config: Res<NateroidConfig>,
    boundary: Res<Boundary>,
) {
    for (entity, health, transform, nateroid, name) in query.iter() {
        if health.0 <= 0.0 {
            if nateroid.is_some() {
                let entity_name = name.map(|n| (*n).as_str()).unwrap_or("Unknown");
                info!(
                    "☠️ despawn_dead_entities: Adding Deaderoid to {} (health: {})",
                    entity_name, health.0
                );

                // Calculate shrink rate based on death duration
                let shrink_rate = (1.0 - config.death_shrink_pct) / config.death_duration_secs;

                // Find nearest back wall corner and vector toward it
                let death_velocity = calculate_death_velocity(transform.translation, &boundary);

                // Nateroid - start death animation
                commands
                    .entity(entity)
                    .insert((
                        Deaderoid {
                            initial_scale: transform.scale,
                            target_shrink: config.death_shrink_pct,
                            shrink_rate,
                            current_shrink: 1.0,
                        },
                        CollisionLayers::NONE,
                        LinearVelocity(death_velocity),
                    ))
                    .remove::<LockedAxes>();
            } else {
                // Other entities - despawn immediately
                despawn(&mut commands, entity);
            }
        }
    }
}

fn despawn_all_entities(mut commands: Commands, query: Query<Entity, With<Health>>) {
    println!("GameOver");
    for entity in query.iter() {
        despawn(&mut commands, entity);
    }
}

fn despawn_splash(mut commands: Commands, query: Query<Entity, With<crate::splash::SplashText>>) {
    for entity in query.iter() {
        despawn(&mut commands, entity);
    }
}

fn animate_dying_nateroids(mut query: Query<(&mut Deaderoid, &mut Transform)>, time: Res<Time>) {
    for (mut deaderoid, mut transform) in query.iter_mut() {
        // Gradually shrink based on shrink_rate
        deaderoid.current_shrink -= deaderoid.shrink_rate * time.delta_secs();
        deaderoid.current_shrink = deaderoid.current_shrink.max(deaderoid.target_shrink);

        // Apply shrinking to transform
        transform.scale = deaderoid.initial_scale * deaderoid.current_shrink;

        // Note: despawn happens in teleport system when Deaderoid entities teleport
    }
}
