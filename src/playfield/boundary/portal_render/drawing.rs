use bevy::prelude::*;

use super::geometry;
use super::geometry::MultiFaceGeometry;
use super::geometry::PortalGeometry;
use super::intersection;
use super::intersection::ArcGeometry;
use crate::orientation::CameraOrientation;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::CORNER_COLOR_FRONT_BACK_XY;
use crate::playfield::constants::CORNER_COLOR_LEFT_RIGHT_YZ;
use crate::playfield::constants::CORNER_COLOR_TOP_BOTTOM_XZ;
use crate::playfield::constants::DEADEROID_APPROACHING_COLOR;
use crate::playfield::portals::Portal;
use crate::playfield::portals::PortalGizmo;

/// Distinguishes normal actors from deaderoids in portal rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalActorKind {
    Nateroid,
    Deaderoid,
}

struct PortalRenderContext<'a> {
    color:       Color,
    resolution:  u32,
    orientation: &'a CameraOrientation,
    actor_kind:  PortalActorKind,
    transform:   &'a Transform,
}

pub fn draw_portal(
    gizmos: &mut Gizmos<PortalGizmo>,
    portal: &Portal,
    color: Color,
    resolution: u32,
    orientation: &CameraOrientation,
    actor_kind: PortalActorKind,
    transform: &Transform,
) {
    let geometry = geometry::classify_portal_geometry(portal, transform);
    let context = PortalRenderContext {
        color,
        resolution,
        orientation,
        actor_kind,
        transform,
    };
    render_portal_by_geometry(gizmos, portal, &context, &geometry);
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
                context.orientation.orientation_settings.axis_profundus,
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
        let intersections = intersection::flatten_intersections(
            intersection::intersect_portal_with_rectangle(portal, &face_points),
        );

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
                let center = intersection::rotate_portal_center_to_target_face(
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
