use avian3d::prelude::*;
use bevy::prelude::*;
use rand::Rng;

use crate::actor::Deaderoid;
use crate::actor::Health;
use crate::actor::MissilePosition;
use crate::actor::Nateroid;
use crate::actor::NateroidDeathMaterials;
use crate::actor::actor_template::DeathCorner;
use crate::actor::actor_template::NateroidConfig;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::state::GameState;
use crate::traits::UsizeExt;

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

/// Calculates velocity toward a boundary corner based on the death corner strategy.
/// Velocity is calculated to reach the corner in exactly `death_duration` seconds.
fn calculate_death_velocity(
    position: Vec3,
    current_velocity: Vec3,
    boundary: &Boundary,
    death_duration: f32,
    death_corner: DeathCorner,
) -> Vec3 {
    const EPSILON: f32 = 0.001;
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
                    dist_a.total_cmp(&dist_b)
                })
                .copied()
                .unwrap_or(corners[0])
        },
        DeathCorner::Random => {
            // Randomly select one corner
            let mut rng = rand::rng();
            corners[rng.random_range(0..8)]
        },
        DeathCorner::Directional => {
            // Find corner most aligned with current velocity direction
            let velocity_dir = current_velocity.normalize_or_zero();

            // Calculate dot product for each corner (how aligned it is with velocity)
            let corner_scores: Vec<(Vec3, f32)> = corners
                .iter()
                .map(|&corner| {
                    let to_corner = (corner - position).normalize_or_zero();
                    let dot = velocity_dir.dot(to_corner);
                    (corner, dot)
                })
                .collect();

            // Find maximum dot product (most aligned)
            let max_dot = corner_scores
                .iter()
                .map(|(_, dot)| *dot)
                .max_by(f32::total_cmp)
                .unwrap_or(0.0);

            // Collect all corners within epsilon of max (handles ties)
            let best_corners: Vec<Vec3> = corner_scores
                .iter()
                .filter(|(_, dot)| (dot - max_dot).abs() < EPSILON)
                .map(|(corner, _)| *corner)
                .collect();

            // If multiple corners equally aligned, randomly pick one
            if best_corners.len() > 1 {
                let mut rng = rand::rng();
                best_corners[rng.random_range(0..best_corners.len())]
            } else {
                best_corners.first().copied().unwrap_or(corners[0])
            }
        },
    };

    // Calculate velocity to reach corner in exactly death_duration seconds
    // velocity = (target_position - current_position) / time
    (target_corner - position) / death_duration
}

fn despawn_dead_entities(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Health,
            &Transform,
            &LinearVelocity,
            Option<&Nateroid>,
            Option<&Name>,
        ),
        Without<Deaderoid>,
    >,
    config: Res<NateroidConfig>,
    boundary: Res<Boundary>,
    death_materials: Option<Res<NateroidDeathMaterials>>,
    children_query: Query<&Children>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    for (entity, health, transform, linear_velocity, nateroid, name) in query.iter() {
        if health.0 <= 0.0 {
            if nateroid.is_some() {
                let entity_name = name.map_or("Unknown", |n| (*n).as_str());
                debug!(
                    "☠️ despawn_dead_entities: Adding Deaderoid to {} (health: {})",
                    entity_name, health.0
                );

                // Calculate velocity to reach target corner in death_duration
                let death_velocity = calculate_death_velocity(
                    transform.translation,
                    linear_velocity.0,
                    &boundary,
                    config.death_duration_secs,
                    config.death_corner,
                );

                // Nateroid - start death animation
                commands
                    .entity(entity)
                    .insert((
                        Deaderoid {
                            initial_scale:          transform.scale,
                            target_shrink:          config.death_shrink_pct,
                            shrink_duration:        config.death_duration_secs,
                            elapsed_time:           0.0,
                            current_shrink:         1.0,
                            current_material_index: 0,
                        },
                        CollisionLayers::NONE,
                        LinearVelocity(death_velocity),
                    ))
                    .remove::<LockedAxes>();

                // Apply initial materials (index 0, alpha 0.25) immediately
                if let Some(death_materials) = &death_materials
                    && !death_materials.materials.is_empty()
                {
                    let materials_for_level = &death_materials.materials[0];
                    let mut material_index = 0;

                    for descendant in children_query.iter_descendants(entity) {
                        if material_query.get(descendant).is_ok()
                            && material_index < materials_for_level.len()
                        {
                            commands.entity(descendant).insert(MeshMaterial3d(
                                materials_for_level[material_index].clone(),
                            ));
                            material_index += 1;
                        }
                    }

                    debug!(
                        "💀 {entity_name}: Applied initial materials (index 0, alpha {:.2}) to {material_index} descendants",
                        config.initial_alpha
                    );
                }
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
    mut query: Query<(&mut Deaderoid, &mut Transform, Entity, Option<&Name>)>,
    time: Res<Time>,
    death_materials: Option<Res<NateroidDeathMaterials>>,
    children_query: Query<&Children>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    nateroid_config: Res<NateroidConfig>,
    mut commands: Commands,
) {
    // Early return if materials haven't been precomputed yet
    let Some(death_materials) = death_materials else {
        return;
    };

    for (mut deaderoid, mut transform, entity, name) in &mut query {
        let entity_name = name.map_or("Unknown", |n| (*n).as_str());

        // Update elapsed time
        deaderoid.elapsed_time += time.delta_secs();

        // Calculate progress (0.0 to 1.0)
        let progress = (deaderoid.elapsed_time / deaderoid.shrink_duration).min(1.0);

        // Linear interpolation from 1.0 (full size) to target_shrink
        // FMA optimization (faster + more precise): 1.0 - (1.0 - deaderoid.target_shrink) *
        // progress
        deaderoid.current_shrink = (1.0 - deaderoid.target_shrink).mul_add(-progress, 1.0);

        // Apply shrinking to transform
        transform.scale = deaderoid.initial_scale * deaderoid.current_shrink;

        // Apply ease-out curve (inverse cubic) for material swapping - fades rapidly at first,
        // then slows down (exponential decay)
        let eased_progress = 1.0 - (1.0 - progress).powi(3);
        // Safe: eased_progress is 0.0-1.0, bounded by array size, result is valid index
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let new_index = (eased_progress * (death_materials.materials.len() - 1).to_f32()) as usize;

        // Only swap materials when index changes
        if new_index != deaderoid.current_material_index {
            let old_index = deaderoid.current_material_index;
            deaderoid.current_material_index = new_index;

            // Calculate the alpha value for this level
            // FMA optimization (faster + more precise): initial_alpha - (new_index as f32 * 0.01)
            let alpha = new_index
                .to_f32()
                .mul_add(-0.01, nateroid_config.initial_alpha);

            debug!(
                "💀 {entity_name}: Material swap {old_index} → {new_index} | progress: {:.3} → {:.3} | alpha: {:.2}",
                progress, eased_progress, alpha
            );

            // Get the precomputed materials for this transparency level
            let materials_for_level = &death_materials.materials[new_index];

            // Swap material handles for all descendants
            let mut material_index = 0;
            for descendant in children_query.iter_descendants(entity) {
                if material_query.get(descendant).is_ok()
                    && material_index < materials_for_level.len()
                {
                    commands
                        .entity(descendant)
                        .insert(MeshMaterial3d(materials_for_level[material_index].clone()));
                    material_index += 1;
                }
            }

            debug!("💀 {entity_name}: Swapped materials on {material_index} descendants");
        }

        // Note: Velocity is constant (set once in despawn_dead_entities)
        // Despawn happens in teleport system when Deaderoid entities teleport
    }
}
