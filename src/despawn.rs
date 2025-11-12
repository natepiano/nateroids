use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::actor::Deaderoid;
use crate::actor::Health;
use crate::actor::MissilePosition;
use crate::actor::Nateroid;
use crate::actor::actor_template::DeathCorner;
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

/// Sets transparency on all descendant entities with StandardMaterial
/// Clones materials to avoid affecting other entities using the same material
fn set_transparency_for_descendants(
    entity: Entity,
    alpha: f32,
    children_query: &Query<&Children>,
    materials: &mut Assets<StandardMaterial>,
    material_query: &Query<&MeshMaterial3d<StandardMaterial>>,
    commands: &mut Commands,
) {
    // Iterate over all descendants using Bevy's built-in iterator
    for descendant in children_query.iter_descendants(entity) {
        if let Ok(material_handle) = material_query.get(descendant) {
            // Clone the material to avoid affecting other entities
            if let Some(original_material) = materials.get(&material_handle.0) {
                let mut cloned_material = original_material.clone();
                cloned_material.base_color.set_alpha(alpha);
                cloned_material.alpha_mode = AlphaMode::Blend;

                // Add the cloned material and update the entity's material handle
                let new_handle = materials.add(cloned_material);
                commands
                    .entity(descendant)
                    .insert(MeshMaterial3d(new_handle));
            }
        }
    }
}

/// Calculates velocity toward a boundary corner based on the death corner strategy.
/// Velocity is calculated to reach the corner in exactly `death_duration` seconds.
fn calculate_death_velocity(
    position: Vec3,
    boundary: &Boundary,
    death_duration: f32,
    death_corner: DeathCorner,
) -> Vec3 {
    let half_size = boundary.transform.scale / 2.0;
    let center = boundary.transform.translation;

    // All 8 corners of the boundary cube
    let corners = [
        Vec3::new(
            center.x - half_size.x,
            center.y - half_size.y,
            center.z - half_size.z,
        ), // Back bottom-left
        Vec3::new(
            center.x + half_size.x,
            center.y - half_size.y,
            center.z - half_size.z,
        ), // Back bottom-right
        Vec3::new(
            center.x - half_size.x,
            center.y + half_size.y,
            center.z - half_size.z,
        ), // Back top-left
        Vec3::new(
            center.x + half_size.x,
            center.y + half_size.y,
            center.z - half_size.z,
        ), // Back top-right
        Vec3::new(
            center.x - half_size.x,
            center.y - half_size.y,
            center.z + half_size.z,
        ), // Front bottom-left
        Vec3::new(
            center.x + half_size.x,
            center.y - half_size.y,
            center.z + half_size.z,
        ), // Front bottom-right
        Vec3::new(
            center.x - half_size.x,
            center.y + half_size.y,
            center.z + half_size.z,
        ), // Front top-left
        Vec3::new(
            center.x + half_size.x,
            center.y + half_size.y,
            center.z + half_size.z,
        ), // Front top-right
    ];

    // Select target corner based on strategy
    let target_corner = match death_corner {
        DeathCorner::Nearest => {
            // Find nearest corner
            corners
                .iter()
                .min_by(|a, b| {
                    let dist_a = position.distance_squared(**a);
                    let dist_b = position.distance_squared(**b);
                    dist_a.partial_cmp(&dist_b).unwrap()
                })
                .copied()
                .unwrap_or(corners[0])
        },
        DeathCorner::Random => {
            // Randomly select one corner
            let mut rng = rand::rng();
            corners[rng.random_range(0..8)]
        },
    };

    // Calculate velocity to reach corner in exactly death_duration seconds
    // velocity = (target_position - current_position) / time
    (target_corner - position) / death_duration
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
    children_query: Query<&Children>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (entity, health, transform, nateroid, name) in query.iter() {
        if health.0 <= 0.0 {
            if nateroid.is_some() {
                let entity_name = name.map(|n| (*n).as_str()).unwrap_or("Unknown");
                debug!(
                    "☠️ despawn_dead_entities: Adding Deaderoid to {} (health: {})",
                    entity_name, health.0
                );

                // Calculate velocity to reach target corner in death_duration
                let death_velocity = calculate_death_velocity(
                    transform.translation,
                    &boundary,
                    config.death_duration_secs,
                    config.death_corner,
                );

                // Set 75% transparency (alpha 0.25) on all child materials
                set_transparency_for_descendants(
                    entity,
                    0.25,
                    &children_query,
                    &mut materials,
                    &material_query,
                    &mut commands,
                );

                // Nateroid - start death animation
                commands
                    .entity(entity)
                    .insert((
                        Deaderoid {
                            initial_scale:   transform.scale,
                            target_shrink:   config.death_shrink_pct,
                            shrink_duration: config.death_duration_secs,
                            elapsed_time:    0.0,
                            current_shrink:  1.0,
                            initial_alpha:   0.25,
                            target_alpha:    0.10,
                            current_alpha:   0.25,
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

fn animate_dying_nateroids(
    mut query: Query<(&mut Deaderoid, &mut Transform, Entity)>,
    time: Res<Time>,
    children_query: Query<&Children>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (mut deaderoid, mut transform, entity) in query.iter_mut() {
        // Update elapsed time
        deaderoid.elapsed_time += time.delta_secs();

        // Calculate progress (0.0 to 1.0)
        let progress = (deaderoid.elapsed_time / deaderoid.shrink_duration).min(1.0);

        // Linear interpolation from 1.0 (full size) to target_shrink
        deaderoid.current_shrink = 1.0 - (1.0 - deaderoid.target_shrink) * progress;

        // Apply shrinking to transform
        transform.scale = deaderoid.initial_scale * deaderoid.current_shrink;

        // Apply ease-in curve (cubic) for alpha fade - stays visible, then rapidly fades at end
        let eased_alpha_progress = progress * progress * progress;
        deaderoid.current_alpha = deaderoid.initial_alpha
            - (deaderoid.initial_alpha - deaderoid.target_alpha) * eased_alpha_progress;

        // Apply alpha to all descendant materials
        for descendant in children_query.iter_descendants(entity) {
            if let Ok(material_handle) = material_query.get(descendant) {
                if let Some(material) = materials.get_mut(&material_handle.0) {
                    material.base_color.set_alpha(deaderoid.current_alpha);
                }
            }
        }

        // Note: Velocity is constant (set once in despawn_dead_entities)
        // Despawn happens in teleport system when Deaderoid entities teleport
    }
}
