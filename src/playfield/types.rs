use bevy::math::Vec3;
use bevy::prelude::*;

use super::boundary_face::BoundaryFace;

/// Distinguishes normal actors from deaderoids in portal rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PortalActorKind {
    Nateroid,
    Deaderoid,
}

/// Describes the geometric configuration of a portal relative to boundary faces
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

#[derive(Default, Reflect, GizmoConfigGroup)]
pub(super) struct GridGizmo {}

/// Trigger event to start a grid flash animation
#[derive(Event)]
pub struct GridFlash;

/// Active grid flash animation timer
#[derive(Resource)]
pub(super) struct GridFlashAnimation {
    pub timer: Timer,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub(super) struct BoundaryGizmo {}

pub(super) enum Intersection {
    NoneFound,
    One(Vec3),
    Two(Vec3, Vec3),
}

/// Flattens an array of `Intersection` results into a `Vec<Vec3>`.
///
/// We need to be aware that as long as our portals are smaller than faces we could only ever
/// have no, one or two intersections in total with the line segments comprising a boundary
/// face.
///
/// The `debug_assert` makes sure that if we ever change the invariant and allow portals to be
/// larger than faces, it will catch it and remind us that we need to update our portal drawing
/// logic accordingly.
pub(super) fn flatten_intersections(intersections: [Intersection; 4]) -> Vec<Vec3> {
    let result: Vec<Vec3> = intersections
        .into_iter()
        .flat_map(|intersection| match intersection {
            Intersection::NoneFound => vec![],
            Intersection::One(p) => vec![p],
            Intersection::Two(p1, p2) => vec![p1, p2],
        })
        .collect();

    // Debug assertion: A circle can intersect a rectangle's 4 edges at most 4 times.
    // This occurs when the portal is positioned near a corner of the face -
    // the circle can intersect both adjacent edges (2 points each = 4 total).
    // Portals positioned in the center typically produce 2 intersection points.
    debug_assert!(
        result.len() <= 4,
        "Circle-rectangle intersection exceeded maximum: {} intersection points (expected ≤4). \
         This indicates a geometric error in the intersection calculation.",
        result.len()
    );

    result
}
