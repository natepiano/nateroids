use bevy::prelude::*;

use super::intersection;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::BOUNDARY_OVEREXTENSION_EPSILON;
use crate::playfield::portals::Portal;

/// Describes the geometric configuration of a portal relative to boundary faces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PortalGeometry {
    /// `Portal` completely within a single boundary face.
    SingleFace,
    /// `Portal` extends across multiple faces (edge or corner).
    MultiFace(MultiFaceGeometry),
}

/// Describes `Portal`s that span multiple boundary faces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum MultiFaceGeometry {
    /// `Portal` extends across an edge between two `BoundaryFace`s.
    Edge { overextended: BoundaryFace },
    /// `Portal` extends into a corner (3+ `BoundaryFace`s).
    Corner { overextended: Vec<BoundaryFace> },
}

/// Calculates how many faces a portal spans at a given position
pub fn calculate_portal_face_count(portal: &Portal, transform: &Transform) -> usize {
    let geometry = classify_portal_geometry(portal, transform);

    match geometry {
        PortalGeometry::SingleFace => 1,
        PortalGeometry::MultiFace(multiface) => {
            count_faces_with_valid_arcs(portal, &multiface, transform)
        },
    }
}

/// Analyzes portal geometry relative to boundary faces
pub(super) fn classify_portal_geometry(portal: &Portal, transform: &Transform) -> PortalGeometry {
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
        let intersections = intersection::flatten_intersections(
            intersection::intersect_portal_with_rectangle(portal, &face_points),
        );

        // Only count faces with exactly 2 intersection points
        if intersections.len() == 2 {
            face_count += 1;
        }
    }

    face_count
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
