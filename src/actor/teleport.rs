use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_kana::Position;

use super::Deaderoid;
use super::Health;
use super::Nateroid;
use super::constants::INSTANT_DEATH_HEALTH;
use super::game_layer::GameLayer;
use super::nateroid::NateroidSettings;
use super::nateroid::NateroidSpawnStats;
use super::spaceship::Spaceship;
use crate::despawn;
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
            calculate_teleport_position(original_position, boundary_transform);

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

/// Wraps a position to the opposite side of the boundary on any axis where it has exited.
///
/// Returns the original position unchanged if it is fully inside the boundary.
fn calculate_teleport_position(position: Position, transform: &Transform) -> Position {
    let boundary_min = transform.translation - transform.scale / 2.0;
    let boundary_max = transform.translation + transform.scale / 2.0;

    let mut teleport_position = *position;

    if position.x >= boundary_max.x {
        let offset = position.x - boundary_max.x;
        teleport_position.x = boundary_min.x + offset;
    } else if position.x <= boundary_min.x {
        let offset = boundary_min.x - position.x;
        teleport_position.x = boundary_max.x - offset;
    }

    if position.y >= boundary_max.y {
        let offset = position.y - boundary_max.y;
        teleport_position.y = boundary_min.y + offset;
    } else if position.y <= boundary_min.y {
        let offset = boundary_min.y - position.y;
        teleport_position.y = boundary_max.y - offset;
    }

    if position.z >= boundary_max.z {
        let offset = position.z - boundary_max.z;
        teleport_position.z = boundary_min.z + offset;
    } else if position.z <= boundary_min.z {
        let offset = boundary_min.z - position.z;
        teleport_position.z = boundary_max.z - offset;
    }

    Position(teleport_position)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLOAT_EPSILON: f32 = 0.000_001;

    fn create_test_transform(size: Vec3) -> Transform {
        Transform {
            translation: Vec3::ZERO,
            scale: size,
            ..default()
        }
    }

    fn boundary_extents(transform: &Transform) -> (Vec3, Vec3) {
        let half_size = transform.scale / 2.0;
        (
            transform.translation - half_size,
            transform.translation + half_size,
        )
    }

    fn wrap_from_max_axis(position: f32, axis_min: f32, axis_max: f32) -> f32 {
        axis_min + (position - axis_max)
    }

    fn wrap_from_min_axis(position: f32, axis_min: f32, axis_max: f32) -> f32 {
        axis_max - (axis_min - position)
    }

    fn assert_float_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= FLOAT_EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_position_eq(actual: Position, expected: Position) {
        assert_float_eq(actual.x, expected.x);
        assert_float_eq(actual.y, expected.y);
        assert_float_eq(actual.z, expected.z);
    }

    #[test]
    fn test_no_teleport_when_inside_boundary() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        let inside_positions = vec![
            Position::new(0.0, 0.0, 0.0),
            Position::new(10.0, 0.0, 0.0),
            Position::new(0.0, 20.0, 0.0),
            Position::new(0.0, 0.0, 30.0),
            Position::new(-10.0, -20.0, -30.0),
        ];

        for pos in inside_positions {
            let result = calculate_teleport_position(pos, &transform);
            assert_position_eq(result, pos);
        }
    }

    #[test]
    fn test_teleport_right_face_to_left() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(55.0, 0.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_left_face_to_right() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(-60.0, 0.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_min_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_top_face_to_bottom() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(0.0, 53.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_bottom_face_to_top() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(0.0, -58.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_min_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_front_face_to_back() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(0.0, 0.0, 52.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                position.y,
                wrap_from_max_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }

    #[test]
    fn test_teleport_back_face_to_front() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(0.0, 0.0, -57.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                position.y,
                wrap_from_min_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }

    #[test]
    fn test_teleport_preserves_offset_on_other_axes() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(55.0, 20.0, -10.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_edge_wrapping() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(53.0, 52.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_corner_wrapping() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(55.0, 58.0, 52.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                wrap_from_max_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }

    #[test]
    fn test_teleport_large_offset() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(200.0, 0.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_with_non_centered_boundary() {
        let transform = Transform {
            translation: Vec3::new(100.0, 50.0, -25.0),
            scale: Vec3::new(200.0, 100.0, 50.0),
            ..default()
        };
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(205.0, 50.0, -25.0);
        let result = calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );

        let position = Position::new(100.0, 103.0, -25.0);
        let result = calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_exactly_at_boundary() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, _) = boundary_extents(&transform);

        let position = Position::new(50.0, 0.0, 0.0);
        let result = calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(boundary_min.x, position.y, position.z),
        );
    }

    #[test]
    fn test_teleport_asymmetric_boundary() {
        let transform = create_test_transform(Vec3::new(200.0, 50.0, 80.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        let position = Position::new(110.0, 0.0, 0.0);
        let result = calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );

        let position = Position::new(0.0, 30.0, 0.0);
        let result = calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );

        let position = Position::new(0.0, 0.0, -45.0);
        let result = calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                position.x,
                position.y,
                wrap_from_min_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }
}
