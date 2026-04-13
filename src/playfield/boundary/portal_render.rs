use bevy::prelude::*;

use crate::orientation::CameraOrientation;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::BOUNDARY_OVEREXTENSION_EPSILON;
use crate::playfield::constants::CORNER_COLOR_FRONT_BACK_XY;
use crate::playfield::constants::CORNER_COLOR_LEFT_RIGHT_YZ;
use crate::playfield::constants::CORNER_COLOR_TOP_BOTTOM_XZ;
use crate::playfield::constants::DEADEROID_APPROACHING_COLOR;
use crate::playfield::constants::INTERSECTION_DEDUP_EPSILON;
use crate::playfield::portals::Portal;
use crate::playfield::portals::PortalGizmo;
use crate::playfield::types;
use crate::playfield::types::Intersection;
use crate::playfield::types::MultiFaceGeometry;
use crate::playfield::types::PortalActorKind;
use crate::playfield::types::PortalGeometry;

struct PortalRenderContext<'a> {
    color:       Color,
    resolution:  u32,
    orientation: &'a CameraOrientation,
    actor_kind:  PortalActorKind,
    transform:   &'a Transform,
}

struct ArcGeometry {
    center: Vec3,
    radius: f32,
    normal: Vec3,
    from:   Vec3,
    to:     Vec3,
}

/// Calculates how many faces a portal spans at a given position
pub(super) fn calculate_portal_face_count(portal: &Portal, transform: &Transform) -> usize {
    let geometry = classify_portal_geometry(portal, transform);

    match geometry {
        PortalGeometry::SingleFace => 1,
        PortalGeometry::MultiFace(multiface) => {
            count_faces_with_valid_arcs(portal, &multiface, transform)
        },
    }
}

pub(super) fn draw_portal(
    gizmos: &mut Gizmos<PortalGizmo>,
    portal: &Portal,
    color: Color,
    resolution: u32,
    orientation: &CameraOrientation,
    actor_kind: PortalActorKind,
    transform: &Transform,
) {
    let geometry = classify_portal_geometry(portal, transform);
    let context = PortalRenderContext {
        color,
        resolution,
        orientation,
        actor_kind,
        transform,
    };
    render_portal_by_geometry(gizmos, portal, &context, &geometry);
}

/// Analyzes portal geometry relative to boundary faces
fn classify_portal_geometry(portal: &Portal, transform: &Transform) -> PortalGeometry {
    let overextended_faces = get_overextended_faces_for(portal, transform);

    if overextended_faces.is_empty() {
        PortalGeometry::SingleFace
    } else if overextended_faces.len() == 1 {
        PortalGeometry::MultiFace(MultiFaceGeometry::Edge {
            overextended: overextended_faces[0],
        })
    } else {
        PortalGeometry::MultiFace(MultiFaceGeometry::Corner {
            overextended: overextended_faces,
        })
    }
}

/// Counts how many faces have valid arc intersections for a multi-face portal
fn count_faces_with_valid_arcs(
    portal: &Portal,
    multiface: &MultiFaceGeometry,
    transform: &Transform,
) -> usize {
    // Calculate boundary extents for constraint checking
    let half_size = transform.scale / 2.0;
    let min = transform.translation - half_size;
    let max = transform.translation + half_size;

    // Collect all faces from the geometry (primary from portal.face + overextended)
    let all_faces_in_corner = match multiface {
        MultiFaceGeometry::Edge { overextended } => vec![portal.face, *overextended],
        MultiFaceGeometry::Corner { overextended } => {
            let mut faces = vec![portal.face];
            faces.extend(overextended);
            faces
        },
    };

    let mut face_count = 0;

    // Calculate constrained intersections for each face
    for &face in &all_faces_in_corner {
        let face_points = face.get_face_points(&min, &max);
        let intersections =
            types::flatten_intersections(intersect_portal_with_rectangle(portal, &face_points));

        // Only count faces with exactly 2 intersection points
        if intersections.len() == 2 {
            face_count += 1;
        }
    }

    face_count
}

fn render_portal_by_geometry(
    gizmos: &mut Gizmos<PortalGizmo>,
    portal: &Portal,
    context: &PortalRenderContext<'_>,
    geometry: &PortalGeometry,
) {
    match geometry {
        PortalGeometry::SingleFace => {
            // Draw full circle
            let rotation = Quat::from_rotation_arc(
                context.orientation.settings.axis_profundus,
                portal.normal().as_vec3(),
            );
            let isometry = Isometry3d::new(*portal.position, rotation);
            gizmos
                .circle(isometry, portal.radius, context.color)
                .resolution(context.resolution);
        },
        PortalGeometry::MultiFace(multiface) => {
            draw_multiface_portal(
                gizmos,
                portal,
                context.color,
                context.resolution,
                context.actor_kind,
                multiface,
                context.transform,
            );
        },
    }
}

fn draw_multiface_portal(
    gizmos: &mut Gizmos<PortalGizmo>,
    portal: &Portal,
    color: Color,
    resolution: u32,
    actor_kind: PortalActorKind,
    geometry: &MultiFaceGeometry,
    transform: &Transform,
) {
    // Extract overextended faces from geometry (primary is always portal.face)
    let primary_face = portal.face;
    let overextended_faces = match geometry {
        MultiFaceGeometry::Edge { overextended } => vec![*overextended],
        MultiFaceGeometry::Corner { overextended } => overextended.clone(),
    };

    // Calculate boundary extents for constraint checking
    let half_size = transform.scale / 2.0;
    let min = transform.translation - half_size;
    let max = transform.translation + half_size;

    // Collect ALL faces that need arcs (primary + overextended)
    let mut all_faces_for_drawing = vec![primary_face];
    all_faces_for_drawing.extend(overextended_faces.iter());

    let mut face_arcs = Vec::new();

    // Calculate constrained intersections for each face
    for &face in &all_faces_for_drawing {
        let face_points = face.get_face_points(&min, &max);
        let intersections =
            types::flatten_intersections(intersect_portal_with_rectangle(portal, &face_points));

        // Only draw arcs for faces with exactly 2 intersection points
        if intersections.len() == 2 {
            face_arcs.push((face, intersections));
        }
    }

    // Draw all arcs
    for (face, points) in face_arcs {
        let face_color = get_portal_color(actor_kind, geometry, face, color);

        match geometry {
            MultiFaceGeometry::Edge { .. } if face == primary_face => {
                // Primary face (contains actual portal.position) at edge uses complex arc logic
                // with TAU angle inversion
                draw_primary_face_arc(
                    gizmos,
                    &ArcGeometry {
                        center: *portal.position,
                        radius: portal.radius,
                        normal: portal.normal().as_vec3(),
                        from:   points[0],
                        to:     points[1],
                    },
                    face_color,
                    resolution,
                );
            },
            MultiFaceGeometry::Edge { .. } => {
                // The single Edge overextended face
                let center = rotate_portal_center_to_target_face(
                    *portal.position,
                    portal.normal(),
                    face,
                    transform,
                );
                gizmos
                    .short_arc_3d_between(center, points[0], points[1], face_color)
                    .resolution(resolution);
            },
            MultiFaceGeometry::Corner { .. } => {
                // For ALL corner faces (including primary)
                gizmos
                    .short_arc_3d_between(*portal.position, points[0], points[1], face_color)
                    .resolution(resolution);
            },
        }
    }
}

// when we rotate this to the target face we get a new center
// for the arc that is drawn outside the boundary
// wrapped to a point that provide a center that gives
// the illusion of having the circle wrap around the edge
fn rotate_portal_center_to_target_face(
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

// Helper function to draw an arc with explicit center, radius, and normal
// Used for primary face arcs - inverts the angle for proper rendering
fn draw_primary_face_arc(
    gizmos: &mut Gizmos<PortalGizmo>,
    arc: &ArcGeometry,
    color: Color,
    resolution: u32,
) {
    // Calculate vectors from center to intersection points
    let vec_from = (arc.from - arc.center).normalize();
    let vec_to = (arc.to - arc.center).normalize();

    // Calculate the angle and determine direction
    let mut angle = vec_from.angle_between(vec_to);
    let cross_product = vec_from.cross(vec_to);
    let is_clockwise = cross_product.dot(arc.normal) < 0.0;

    // Invert the angle for arc_3d rendering logic
    angle = std::f32::consts::TAU - angle;

    // Calculate the rotation to align the arc with the boundary face
    let face_rotation = Quat::from_rotation_arc(Vec3::Y, arc.normal);

    // Determine the start vector based on clockwise/counterclockwise
    let start_vec = if is_clockwise { vec_from } else { vec_to };
    let start_rotation = Quat::from_rotation_arc(face_rotation * Vec3::X, start_vec);

    // Combine rotations
    let final_rotation = start_rotation * face_rotation;

    // Draw the arc
    gizmos
        .arc_3d(
            angle,
            arc.radius,
            Isometry3d::new(arc.center, final_rotation),
            color,
        )
        .resolution(resolution);
}

fn get_overextended_faces_for(portal: &Portal, transform: &Transform) -> Vec<BoundaryFace> {
    let mut overextended_faces = Vec::new();
    let half_size = transform.scale / 2.0;
    let min = transform.translation - half_size;
    let max = transform.translation + half_size;
    let radius = portal.radius;

    // Portals are snapped 0.01 inside boundary - without this margin, they'd be incorrectly
    // detected as overextended, triggering broken corner wrapping math
    let epsilon = BOUNDARY_OVEREXTENSION_EPSILON;

    // Check all faces - only truly overextended if beyond boundary + epsilon
    if portal.position.x - radius < min.x - epsilon {
        overextended_faces.push(BoundaryFace::Left);
    }
    if portal.position.x + radius > max.x + epsilon {
        overextended_faces.push(BoundaryFace::Right);
    }
    if portal.position.y - radius < min.y - epsilon {
        overextended_faces.push(BoundaryFace::Bottom);
    }
    if portal.position.y + radius > max.y + epsilon {
        overextended_faces.push(BoundaryFace::Top);
    }
    if portal.position.z - radius < min.z - epsilon {
        overextended_faces.push(BoundaryFace::Back);
    }
    if portal.position.z + radius > max.z + epsilon {
        overextended_faces.push(BoundaryFace::Front);
    }

    // Remove the face the portal is on from the overextended faces
    overextended_faces.retain(|&face| face != portal.face);
    overextended_faces
}

const fn get_portal_color(
    actor_kind: PortalActorKind,
    geometry: &MultiFaceGeometry,
    face: BoundaryFace,
    default_color: Color,
) -> Color {
    match actor_kind {
        PortalActorKind::Deaderoid => match geometry {
            MultiFaceGeometry::Corner { .. } => {
                // Corner: use 3-color diagnostic scheme
                match face {
                    BoundaryFace::Left | BoundaryFace::Right => CORNER_COLOR_LEFT_RIGHT_YZ,
                    BoundaryFace::Top | BoundaryFace::Bottom => CORNER_COLOR_TOP_BOTTOM_XZ,
                    BoundaryFace::Front | BoundaryFace::Back => CORNER_COLOR_FRONT_BACK_XY,
                }
            },
            MultiFaceGeometry::Edge { .. } => DEADEROID_APPROACHING_COLOR,
        },
        PortalActorKind::Nateroid => default_color,
    }
}

fn intersect_portal_with_rectangle(
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
