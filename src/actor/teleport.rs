use avian3d::prelude::*;
use bevy::prelude::*;

use super::Deaderoid;
use super::Health;
use super::Nateroid;
use super::NateroidSpawnStats;
use super::actor_template::GameLayer;
use super::actor_template::NateroidConfig;
use super::spaceship::Spaceship;
use crate::despawn::despawn;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;

pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TeleportCollisionState>()
            .add_observer(on_teleported)
            .add_systems(
                FixedUpdate,
                teleport_at_boundary.in_set(InGameSet::EntityUpdates),
            );
    }
}

#[derive(Resource, Default)]
struct TeleportCollisionState {
    last_field_crowded: Option<bool>,
}

#[derive(Component, Reflect, Debug, Default, Clone)]
pub struct Teleporter {
    pub just_teleported:          bool,
    pub last_teleported_position: Option<Vec3>,
    pub last_teleported_normal:   Option<Dir3>,
}

#[derive(EntityEvent)]
struct Teleported {
    entity:   Entity,
    position: Vec3,
    rotation: Quat,
    collider: Collider,
}

#[allow(clippy::type_complexity)]
fn on_teleported(
    event: On<Teleported>,
    mut params: ParamSet<(
        SpatialQuery,
        Query<(&mut CollisionLayers, &mut Health), With<Nateroid>>,
    )>,
    spawn_stats: Res<NateroidSpawnStats>,
    config: Res<NateroidConfig>,
    mut collision_state: ResMut<TeleportCollisionState>,
) {
    // First, do all spatial queries (collect results before mutating)
    let asteroid_filter = SpatialQueryFilter::from_mask(LayerMask::from([GameLayer::Asteroid]));
    let spaceship_filter = SpatialQueryFilter::from_mask(LayerMask::from([GameLayer::Spaceship]));

    let overlapping_asteroids = params.p0().shape_intersections(
        &event.collider,
        event.position,
        event.rotation,
        &asteroid_filter,
    );

    let overlapping_spaceship = params.p0().shape_intersections(
        &event.collider,
        event.position,
        event.rotation,
        &spaceship_filter,
    );

    // Then, mutate nateroid health/collision layers
    let mut nateroid_query = params.p1();

    // Check if we should be aggressive based on spawn success rate
    // Lower spawn success rate = field is crowded = be more aggressive
    let spawn_success_rate = spawn_stats.success_rate();
    let field_is_crowded = spawn_success_rate < config.density_culling_threshold;

    // Kill overlapping asteroids (but not the teleporting entity)
    // Only kill nateroid-on-nateroid overlaps if field is crowded
    let is_teleporting_nateroid = nateroid_query.get(event.entity).is_ok();

    // Debug logging - only log when crowded state changes
    if (!overlapping_asteroids.is_empty() || !overlapping_spaceship.is_empty())
        && collision_state.last_field_crowded != Some(field_is_crowded)
    {
        info!(
            "🔍 Teleport collision detected - attempts: {}, successes: {}, rate: {:.1}%, threshold: {:.1}%, crowded: {}, is_nateroid: {}",
            spawn_stats.attempts_count(),
            spawn_stats.successes_count(),
            spawn_success_rate * 100.0,
            config.density_culling_threshold * 100.0,
            field_is_crowded,
            is_teleporting_nateroid
        );
        collision_state.last_field_crowded = Some(field_is_crowded);
    }

    for entity in overlapping_asteroids {
        if entity == event.entity {
            continue;
        }

        if let Ok((mut collision_layers, mut health)) = nateroid_query.get_mut(entity) {
            // Always kill if spaceship teleported, or if field is crowded
            if !is_teleporting_nateroid || field_is_crowded {
                info!(
                    "💀 Killing overlapping nateroid - spaceship_teleported: {}, field_crowded: {}",
                    !is_teleporting_nateroid, field_is_crowded
                );
                *collision_layers = CollisionLayers::NONE;
                health.0 = -1.0;
            }
        }
    }

    // If a nateroid teleported onto the spaceship, always kill the nateroid
    if is_teleporting_nateroid
        && !overlapping_spaceship.is_empty()
        && let Ok((mut collision_layers, mut health)) = nateroid_query.get_mut(event.entity)
    {
        info!("💀 Nateroid teleported onto spaceship - killing nateroid");
        *collision_layers = CollisionLayers::NONE;
        health.0 = -1.0;
    }
}

#[allow(clippy::type_complexity)]
pub fn teleport_at_boundary(
    boundary: Res<Boundary>,
    mut commands: Commands,
    mut teleporting_entities: Query<(
        Entity,
        &mut Transform,
        &mut Teleporter,
        &Collider,
        Option<&Name>,
        Option<&Spaceship>,
        Option<&Deaderoid>,
    )>,
) {
    for (entity, mut transform, mut teleporter, collider, name, is_spaceship, is_deaderoid) in
        teleporting_entities.iter_mut()
    {
        let original_position = transform.translation;

        let teleported_position = boundary.calculate_teleport_position(original_position);

        if teleported_position != original_position {
            // If this is a dying nateroid, despawn it instead of teleporting
            if is_deaderoid.is_some() {
                despawn(&mut commands, entity);
                continue;
            }

            // Only log spaceship teleports
            if is_spaceship.is_some() {
                let entity_name = name.map(|n| (*n).as_str()).unwrap_or("Spaceship");
                debug!(
                    "🔄 {} teleporting: from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
                    entity_name,
                    original_position.x,
                    original_position.y,
                    original_position.z,
                    teleported_position.x,
                    teleported_position.y,
                    teleported_position.z
                );
            }

            transform.translation = teleported_position;
            teleporter.just_teleported = true;
            teleporter.last_teleported_position = Some(teleported_position);
            teleporter.last_teleported_normal =
                Some(boundary.get_normal_for_position(teleported_position));

            // Trigger event to handle overlapping entities
            commands.trigger(Teleported {
                entity,
                position: teleported_position,
                rotation: transform.rotation,
                collider: collider.clone(),
            });
        } else {
            teleporter.just_teleported = false;
            teleporter.last_teleported_position = None;
            teleporter.last_teleported_normal = None;
        }
    }
}
