use bevy::math::Vec3;
use bevy::prelude::*;

use crate::playfield::boundary_face::BoundaryFace;

/// Describes the geometric configuration of a portal relative to boundary faces
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortalGeometry {
    /// Portal completely within a single boundary face
    SingleFace,
    /// Portal extends across multiple faces (edge or corner)
    MultiFace(MultiFaceGeometry),
}

/// Describes portals that span multiple boundary faces
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiFaceGeometry {
    /// Portal extends across an edge between two faces
    Edge { overextended: BoundaryFace },
    /// Portal extends into a corner (3+ faces)
    Corner { overextended: Vec<BoundaryFace> },
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct GridGizmo {}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmo {}

pub enum Intersection {
    NoIntersections,
    OneIntersection(Vec3),
    TwoIntersections(Vec3, Vec3),
}

/// we added Intersection for readability in our intersection logic
/// however we need to be aware that as long as our portals are smaller than faces
/// we could only ever have no, one or two intersections in total with the line segments
/// comprising a boundary face
///
/// the `to_vec()` takes our code and flattens it into Vec<Vec3> and this can NEVER be > 2
/// so we added the `debug_assert` to make sure that if we ever change the invariant and
/// allow portals to be larger than faces, that this will catch it and remind us that
/// we need to update our portal drawing logic accordingly.
pub trait FlattenIntersections {
    fn to_vec(self) -> Vec<Vec3>;
}

impl FlattenIntersections for [Intersection; 4] {
    fn to_vec(self) -> Vec<Vec3> {
        let result: Vec<Vec3> = self
            .into_iter()
            .flat_map(|intersection| match intersection {
                Intersection::NoIntersections => vec![],
                Intersection::OneIntersection(p) => vec![p],
                Intersection::TwoIntersections(p1, p2) => vec![p1, p2],
            })
            .collect();

        // Debug assertion: portals must never be larger than faces
        // If this fails, it means our simplified intersection logic is broken
        debug_assert!(
            result.len() <= 2,
            "Portal larger than face detected: {} intersection points (expected ≤2). \
             This breaks the assumption that portals never exceed face boundaries.",
            result.len()
        );

        result
    }
}
