use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_kana::Position;

use super::Deaderoid;
use super::Health;
use super::Nateroid;
use super::actor_template::GameLayer;
use super::actor_template::NateroidSettings;
use super::constants::INSTANT_DEATH_HEALTH;
use super::nateroid::NateroidSpawnStats;
use super::spaceship::Spaceship;
use crate::despawn;
use crate::playfield::Boundary;
use crate::playfield::BoundaryVolume;
use crate::schedule::InGameSet;

pub(super) struct TeleportPlugin;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FieldDensity {
    Open,
    Crowded,
}

#[derive(Resource, Default)]
struct TeleportCollisionState {
    last_field_density: Option<FieldDensity>,
}

#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
pub enum TeleportStatus {
    #[default]
    Ready,
    JustTeleported,
}

#[derive(Component, Reflect, Debug, Default, Clone)]
pub struct Teleporter {
    pub status:   TeleportStatus,
    pub position: Option<Position>,
}

#[derive(EntityEvent)]
struct Teleported {
    entity:   Entity,
    position: Vec3,
    rotation: Quat,
    collider: Collider,
}

fn on_teleported(
    event: On<Teleported>,
    mut params: ParamSet<(SpatialQuery, Query<&mut Health, With<Nateroid>>)>,
    mut commands: Commands,
    spawn_stats: Res<NateroidSpawnStats>,
    nateroid_settings: Res<NateroidSettings>,
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
    let field_density = if spawn_success_rate < nateroid_settings.density_culling_threshold {
        FieldDensity::Crowded
    } else {
        FieldDensity::Open
    };

    // Kill overlapping asteroids (but not the teleporting entity)
    // Only kill nateroid-on-nateroid overlaps if field is crowded
    let is_teleporting_nateroid = nateroid_query.get(event.entity).is_ok();

    // Debug logging - only log when crowded state changes
    if (!overlapping_asteroids.is_empty() || !overlapping_spaceship.is_empty())
        && collision_state.last_field_density != Some(field_density)
    {
        info!(
            "🔍 Teleport collision detected - attempts: {}, successes: {}, rate: {:.1}%, threshold: {:.1}%, density: {field_density:?}, is_nateroid: {is_teleporting_nateroid}",
            spawn_stats.attempts_count(),
            spawn_stats.successes_count(),
            spawn_success_rate * 100.0,
            nateroid_settings.density_culling_threshold * 100.0,
        );
        collision_state.last_field_density = Some(field_density);
    }

    for entity in overlapping_asteroids {
        if entity == event.entity {
            continue;
        }

        if let Ok(mut health) = nateroid_query.get_mut(entity)
            && (!is_teleporting_nateroid || field_density == FieldDensity::Crowded)
        {
            info!(
                "💀 Killing overlapping nateroid - spaceship_teleported: {}, density: {field_density:?}",
                !is_teleporting_nateroid
            );
            commands.entity(entity).insert(CollisionLayers::NONE);
            health.0 = INSTANT_DEATH_HEALTH;
        }
    }

    // If a nateroid teleported onto the spaceship, always kill the nateroid
    if is_teleporting_nateroid
        && !overlapping_spaceship.is_empty()
        && let Ok(mut health) = nateroid_query.get_mut(event.entity)
    {
        info!("💀 Nateroid teleported onto spaceship - killing nateroid");
        commands.entity(event.entity).insert(CollisionLayers::NONE);
        health.0 = INSTANT_DEATH_HEALTH;
    }
}

fn teleport_at_boundary(
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    mut commands: Commands,
    mut teleporting_entities: Query<
        (
            Entity,
            &mut Transform,
            &mut Teleporter,
            &Collider,
            Option<&Name>,
            Option<&Spaceship>,
            Option<&Deaderoid>,
        ),
        Without<BoundaryVolume>,
    >,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    for (entity, mut transform, mut teleporter, collider, name, is_spaceship, is_deaderoid) in
        &mut teleporting_entities
    {
        let original_position = Position(transform.translation);

        let teleported_position =
            Boundary::calculate_teleport_position(original_position, boundary_transform);

        if teleported_position == original_position {
            teleporter.status = TeleportStatus::Ready;
            teleporter.position = None;
        } else {
            // If this is a dying nateroid, despawn it instead of teleporting
            if is_deaderoid.is_some() {
                despawn::despawn(&mut commands, entity);
                continue;
            }

            // Only log spaceship teleports
            if is_spaceship.is_some() {
                let entity_name = name.map_or("Spaceship", Name::as_str);
                debug!(
                    "🔄 {entity_name} teleporting: from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
                    original_position.x,
                    original_position.y,
                    original_position.z,
                    teleported_position.x,
                    teleported_position.y,
                    teleported_position.z
                );
            }

            transform.translation = *teleported_position;
            teleporter.status = TeleportStatus::JustTeleported;
            teleporter.position = Some(teleported_position);

            // Trigger event to handle overlapping entities
            commands.trigger(Teleported {
                entity,
                position: *teleported_position,
                rotation: transform.rotation,
                collider: collider.clone(),
            });
        }
    }
}
