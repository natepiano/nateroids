use bevy::prelude::*;
use bevy_kana::Position;

use super::ActorPortals;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::BOUNDARY_SNAP_EPSILON;
use crate::playfield::constants::PORTAL_PHYSICS_BURST_MULTIPLIER;
use crate::playfield::portals::settings::PortalSettings;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PhysicsBurst {
    Active,
    Inactive,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoundsContainment {
    Inside,
    Outside,
}

pub(super) fn physics_burst(position: Position, boundary_transform: &Transform) -> PhysicsBurst {
    let boundary_half_size = boundary_transform.scale / 2.0;
    let max_distance_from_center = position.distance(boundary_transform.translation);
    let boundary_diagonal = boundary_half_size.length();
    if max_distance_from_center > boundary_diagonal * PORTAL_PHYSICS_BURST_MULTIPLIER {
        PhysicsBurst::Active
    } else {
        PhysicsBurst::Inactive
    }
}

pub(super) fn snap_and_get_face(
    position: Position,
    initial_face: BoundaryFace,
    boundary_transform: &Transform,
) -> (Position, BoundaryFace) {
    let snapped_position =
        snap_position_to_boundary_face(position, initial_face, boundary_transform);
    let final_face = get_face_for_position(snapped_position, boundary_transform);
    (snapped_position, final_face)
}

/// Snaps a position to slightly inside the given boundary face.
/// Offsets by epsilon to prevent false-positive overextension detection that would trigger
/// corner wrapping arcs. Clamps perpendicular axes to handle corner/edge teleportation cases.
fn snap_position_to_boundary_face(
    position: Position,
    face: BoundaryFace,
    transform: &Transform,
) -> Position {
    let boundary_min = transform.translation - transform.scale / 2.0;
    let boundary_max = transform.translation + transform.scale / 2.0;

    // Without this offset, portals on exact boundary would be flagged as overextended
    let epsilon = BOUNDARY_SNAP_EPSILON;

    let mut snapped_position = *position;

    // Set primary axis slightly inside boundary face and clamp perpendicular axes
    match face {
        BoundaryFace::Right => {
            snapped_position.x = boundary_max.x - epsilon;
            snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
            snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
        },
        BoundaryFace::Left => {
            snapped_position.x = boundary_min.x + epsilon;
            snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
            snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
        },
        BoundaryFace::Top => {
            snapped_position.y = boundary_max.y - epsilon;
            snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
            snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
        },
        BoundaryFace::Bottom => {
            snapped_position.y = boundary_min.y + epsilon;
            snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
            snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
        },
        BoundaryFace::Front => {
            snapped_position.z = boundary_max.z - epsilon;
            snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
            snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
        },
        BoundaryFace::Back => {
            snapped_position.z = boundary_min.z + epsilon;
            snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
            snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
        },
    }

    Position(snapped_position)
}

/// Returns the closest boundary face to a position.
/// Uses distance-based matching because teleported positions have offsets (e.g., -54.97 instead
/// of -55.0) that break simple epsilon matching.
pub(super) fn get_face_for_position(position: Position, transform: &Transform) -> BoundaryFace {
    let half_size = transform.scale / 2.0;
    let boundary_min = transform.translation - half_size;
    let boundary_max = transform.translation + half_size;

    // Calculate distance to all 6 faces and return the closest
    let faces = [
        ((position.x - boundary_min.x).abs(), BoundaryFace::Left),
        ((position.x - boundary_max.x).abs(), BoundaryFace::Right),
        ((position.y - boundary_min.y).abs(), BoundaryFace::Bottom),
        ((position.y - boundary_max.y).abs(), BoundaryFace::Top),
        ((position.z - boundary_min.z).abs(), BoundaryFace::Back),
        ((position.z - boundary_max.z).abs(), BoundaryFace::Front),
    ];
    faces[1..]
        .iter()
        .fold(
            faces[0],
            |acc, &current| {
                if current.0 < acc.0 { current } else { acc }
            },
        )
        .1
}

pub(super) fn find_edge_point(
    origin: Position,
    direction: Vec3,
    transform: &Transform,
) -> Option<Position> {
    let boundary_min = transform.translation - transform.scale / 2.0;
    let boundary_max = transform.translation + transform.scale / 2.0;

    let mut closest_hit_time: Option<f32> = None;

    for (start, axis_direction, positive_boundary, negative_boundary) in [
        (origin.x, direction.x, boundary_max.x, boundary_min.x),
        (origin.y, direction.y, boundary_max.y, boundary_min.y),
        (origin.z, direction.z, boundary_max.z, boundary_min.z),
    ] {
        if axis_direction != 0.0 {
            let mut update_closest_hit_time = |boundary: f32| {
                let t = (boundary - start) / axis_direction;
                let point = origin + direction * t;
                if t > 0.0
                    && closest_hit_time.is_none_or(|current| t < current)
                    && bounds_containment(point, start, origin, boundary_min, boundary_max)
                        == BoundsContainment::Inside
                {
                    closest_hit_time = Some(t);
                }
            };

            update_closest_hit_time(positive_boundary);
            update_closest_hit_time(negative_boundary);
        }
    }

    closest_hit_time.map(|t| origin + direction * t)
}

fn bounds_containment(
    point: Position,
    start: f32,
    origin: Position,
    boundary_min: Vec3,
    boundary_max: Vec3,
) -> BoundsContainment {
    if (start - origin.x).abs() < BOUNDARY_SNAP_EPSILON {
        if point.y >= boundary_min.y
            && point.y <= boundary_max.y
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
        {
            BoundsContainment::Inside
        } else {
            BoundsContainment::Outside
        }
    } else if (start - origin.y).abs() < BOUNDARY_SNAP_EPSILON {
        if point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
        {
            BoundsContainment::Inside
        } else {
            BoundsContainment::Outside
        }
    } else if point.x >= boundary_min.x
        && point.x <= boundary_max.x
        && point.y >= boundary_min.y
        && point.y <= boundary_max.y
    {
        BoundsContainment::Inside
    } else {
        BoundsContainment::Outside
    }
}

pub(super) fn smooth_circle_position(
    actor_portals: &Mut<ActorPortals>,
    collision_point: Position,
    current_boundary_wall_face: BoundaryFace,
    portal_settings: &PortalSettings,
) -> Position {
    if let Some(approaching_portal) = &actor_portals.approaching_portal {
        let smoothing_factor = portal_settings.movement_smoothing_factor;

        if approaching_portal
            .normal()
            .dot(current_boundary_wall_face.get_normal())
            > portal_settings.direction_change_factor
        {
            approaching_portal
                .position
                .lerp(collision_point, smoothing_factor)
        } else {
            collision_point
        }
    } else {
        collision_point
    }
}
