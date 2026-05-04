use bevy::prelude::*;

use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::INTERSECTION_DEDUP_EPSILON;
use crate::playfield::portals::Portal;

pub(super) enum Intersection {
    NoneFound,
    One(Vec3),
    Two(Vec3, Vec3),
}

pub(super) struct ArcGeometry {
    pub(super) center: Vec3,
    pub(super) radius: f32,
    pub(super) normal: Vec3,
    pub(super) from:   Vec3,
    pub(super) to:     Vec3,
}

// when we rotate this to the target face we get a new center
// for the arc that is drawn outside the boundary
// wrapped to a point that provide a center that gives
// the illusion of having the circle wrap around the edge
pub(super) fn rotate_portal_center_to_target_face(
    position: Vec3,
    normal: Dir3,
    target_face: BoundaryFace,
    transform: &Transform,
) -> Vec3 {
    let current_normal = normal.as_vec3();
    let target_normal = target_face.get_normal();

    // The rotation axis is the cross product of the current and target normals
    let rotation_axis = current_normal.cross(target_normal).normalize();

    // Find the closest point on the rotation axis to the current position
    let rotation_point =
        find_closest_point_on_edge(position, current_normal, target_normal, transform);

    // Create a rotation quaternion (90 degrees around the rotation axis)
    let rotation = Quat::from_axis_angle(rotation_axis, std::f32::consts::FRAC_PI_2);

    // Apply the rotation to the position relative to the rotation point
    let relative_pos = position - rotation_point;
    let rotated_pos = rotation * relative_pos;

    let mut result = rotation_point + rotated_pos;

    // Rotation math at corners can produce off-plane positions - force result onto target
    // face's plane
    let half_extents = transform.scale / 2.0;
    let center = transform.translation;

    match target_face {
        BoundaryFace::Right => result.x = center.x + half_extents.x,
        BoundaryFace::Left => result.x = center.x - half_extents.x,
        BoundaryFace::Top => result.y = center.y + half_extents.y,
        BoundaryFace::Bottom => result.y = center.y - half_extents.y,
        BoundaryFace::Front => result.z = center.z + half_extents.z,
        BoundaryFace::Back => result.z = center.z - half_extents.z,
    }

    result
}

pub(super) fn intersect_portal_with_rectangle(
    portal: &Portal,
    rectangle_points: &[Vec3; 4],
) -> [Intersection; 4] {
    [
        intersect_circle_with_line_segment(portal, rectangle_points[0], rectangle_points[1]),
        intersect_circle_with_line_segment(portal, rectangle_points[1], rectangle_points[2]),
        intersect_circle_with_line_segment(portal, rectangle_points[2], rectangle_points[3]),
        intersect_circle_with_line_segment(portal, rectangle_points[3], rectangle_points[0]),
    ]
}

/// Flattens an array of `Intersection` results into a `Vec<Vec3>`.
///
/// As long as portals are smaller than boundary faces, the four per-edge
/// intersection tests can yield at most two points in total against a given
/// face. The `debug_assert` below guards that invariant: if portals ever grow
/// larger than faces, it will fire and remind us to revisit the portal-drawing
/// logic.
pub(super) fn flatten_intersections(intersections: [Intersection; 4]) -> Vec<Vec3> {
    let result: Vec<Vec3> = intersections
        .into_iter()
        .flat_map(|intersection| match intersection {
            Intersection::NoneFound => vec![],
            Intersection::One(point) => vec![point],
            Intersection::Two(point_one, point_two) => vec![point_one, point_two],
        })
        .collect();

    // A circle can intersect a rectangle's 4 edges at most 4 times. This
    // occurs when the portal is positioned near a corner of the face — the
    // circle can intersect both adjacent edges (2 points each = 4 total).
    // Portals positioned in the center typically produce 2 intersection
    // points.
    debug_assert!(
        result.len() <= 4,
        "Circle-rectangle intersection exceeded maximum: {} intersection points (expected <=4). \
         This indicates a geometric error in the intersection calculation.",
        result.len()
    );

    result
}

fn find_closest_point_on_edge(
    position: Vec3,
    normal1: Vec3,
    normal2: Vec3,
    transform: &Transform,
) -> Vec3 {
    let half = transform.scale / 2.0;
    let center = transform.translation;
    let min = center - half;
    let max = center + half;

    // For axis-aligned cuboid, the edge between two faces runs along one axis
    // with the other two coordinates fixed at the boundary planes.
    // For each axis: if either normal points along it, fix at that boundary;
    // otherwise the edge runs along that axis, so use position's coordinate.

    let x = if normal1.x != 0.0 {
        if normal1.x > 0.0 { max.x } else { min.x }
    } else if normal2.x != 0.0 {
        if normal2.x > 0.0 { max.x } else { min.x }
    } else {
        position.x // Edge runs along X axis
    };

    let y = if normal1.y != 0.0 {
        if normal1.y > 0.0 { max.y } else { min.y }
    } else if normal2.y != 0.0 {
        if normal2.y > 0.0 { max.y } else { min.y }
    } else {
        position.y // Edge runs along Y axis
    };

    let z = if normal1.z != 0.0 {
        if normal1.z > 0.0 { max.z } else { min.z }
    } else if normal2.z != 0.0 {
        if normal2.z > 0.0 { max.z } else { min.z }
    } else {
        position.z // Edge runs along Z axis
    };

    Vec3::new(x, y, z)
}

fn intersect_circle_with_line_segment(portal: &Portal, start: Vec3, end: Vec3) -> Intersection {
    let edge = end - start;
    let center_to_start = start - *portal.position;

    let a = edge.dot(edge);
    let b = 2.0 * center_to_start.dot(edge);
    // FMA optimization (faster + more precise): dot(center_to_start) - radius²
    let c = portal
        .radius
        .mul_add(-portal.radius, center_to_start.dot(center_to_start));

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return Intersection::NoneFound;
    }

    let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

    let t1_valid = (0.0..=1.0).contains(&t1);
    let t2_valid = (0.0..=1.0).contains(&t2) && (t1 - t2).abs() > INTERSECTION_DEDUP_EPSILON;

    match (t1_valid, t2_valid) {
        (false, false) => Intersection::NoneFound,
        (true, false) => Intersection::One(start + t1 * edge),
        (false, true) => Intersection::One(start + t2 * edge),
        (true, true) => Intersection::Two(start + t1 * edge, start + t2 * edge),
    }
}
