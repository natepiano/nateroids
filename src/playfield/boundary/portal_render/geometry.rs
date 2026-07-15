use bevy::prelude::*;

use super::intersection;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::BOUNDARY_OVEREXTENSION_EPSILON;
use crate::playfield::constants::DEFAULT_PORTAL_FACE_COUNT;
use crate::playfield::constants::VALID_PORTAL_ARC_INTERSECTION_COUNT;
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
pub(crate) fn calculate_portal_face_count(portal: &Portal, transform: &Transform) -> usize {
    let portal_geometry = classify_portal_geometry(portal, transform);

    match portal_geometry {
        PortalGeometry::SingleFace => DEFAULT_PORTAL_FACE_COUNT,
        PortalGeometry::MultiFace(multi_face_geometry) => {
            count_faces_with_valid_arcs(portal, &multi_face_geometry, transform)
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
    multi_face_geometry: &MultiFaceGeometry,
    transform: &Transform,
) -> usize {
    let half_size = transform.scale / 2.0;
    let min = transform.translation - half_size;
    let max = transform.translation + half_size;

    let all_faces_in_corner = match multi_face_geometry {
        MultiFaceGeometry::Edge { overextended } => vec![portal.boundary_face, *overextended],
        MultiFaceGeometry::Corner { overextended } => {
            let mut faces = vec![portal.boundary_face];
            faces.extend(overextended);
            faces
        },
    };

    let mut face_count = 0;

    for &face in &all_faces_in_corner {
        let face_points = face.get_face_points(&min, &max);
        let intersections = intersection::flatten_intersections(
            intersection::intersect_portal_with_rectangle(portal, &face_points),
        );

        // Only count faces when `intersections` has
        // `VALID_PORTAL_ARC_INTERSECTION_COUNT` points.
        if intersections.len() == VALID_PORTAL_ARC_INTERSECTION_COUNT {
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

    // `BOUNDARY_OVEREXTENSION_EPSILON` prevents snapped portal centers from
    // producing false overextension and invalid corner arcs.
    let epsilon = BOUNDARY_OVEREXTENSION_EPSILON;

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
    overextended_faces.retain(|&face| face != portal.boundary_face);
    overextended_faces
}
