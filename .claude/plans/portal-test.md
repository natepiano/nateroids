# Portal Rendering Test Suite Implementation Plan

## Overview

Create comprehensive unit tests for portal rendering logic covering 4 scenarios:
1. **Too far**: Portal doesn't reach any boundary (no drawing)
2. **Single face**: Portal on one face, doesn't overextend (1 circle)
3. **Edge case**: Portal overextends to 2 faces at boundary edge (2 arcs)
4. **Corner case**: Portal overextends to 3 faces at boundary corner (3 arcs)

Tests will use `cargo nextest run` and require no Bevy runtime (pure geometry testing).

## Objectives

- ✅ Validate current correct behavior (scenarios 1-3)
- ❌ Document corner bug with failing tests (scenario 4)
- ✅ Provide regression suite for portal-upgraded.md implementation
- ✅ Enable fast, deterministic testing without Bevy systems

## Phase 1: Implement Helper Functions (Portal-Upgraded Step 1)

### Context

These helper functions are required for:
1. Portal-upgraded.md Step 2 (the main refactor)
2. Testing constraint behavior in our test suite

By implementing them now, we:
- Complete portal-upgraded.md Step 1 early
- Enable comprehensive constraint validation in tests
- Avoid duplicate work later

### Implementation

Add three helper functions to `src/playfield/boundary.rs` after `intersect_circle_with_line_segment()` (around line 657):

#### Helper 1: `constrain_intersection_points()`

```rust
/// Filters intersection points to only include those within the face's boundary limits.
/// At corners, this prevents arcs from extending into adjacent faces.
///
/// Returns filtered vector containing only points within valid region. May be empty
/// if all points were outside boundaries (e.g., small portal near corner).
fn constrain_intersection_points(
    points: Vec<Vec3>,
    current_face: BoundaryFace,
    overextended_faces: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> Vec<Vec3> {
    points.into_iter().filter(|point| {
        point_within_boundary_for_face(*point, current_face, overextended_faces, min, max)
    }).collect()
}
```

#### Helper 2: `point_within_boundary_for_face()`

```rust
fn point_within_boundary_for_face(
    point: Vec3,
    face: BoundaryFace,
    overextended_faces: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> bool {
    // For each overextended face that's on a different axis, ensure point doesn't
    // extend beyond that boundary
    for overextended in overextended_faces {
        if faces_share_axis(face, *overextended) {
            continue; // Same axis, no constraint needed (optimization)
        }

        // Check if point exceeds the boundary this overextended face represents
        // These are exact comparisons - no epsilon needed for geometric filtering
        match overextended {
            BoundaryFace::Left => if point.x < min.x { return false; },
            BoundaryFace::Right => if point.x > max.x { return false; },
            BoundaryFace::Bottom => if point.y < min.y { return false; },
            BoundaryFace::Top => if point.y > max.y { return false; },
            BoundaryFace::Back => if point.z < min.z { return false; },
            BoundaryFace::Front => if point.z > max.z { return false; },
        }
    }

    true
}
```

#### Helper 3: `faces_share_axis()`

```rust
/// Returns true if two faces are perpendicular to the same axis.
/// Used to optimize constraint checks by skipping geometrically impossible conditions.
///
/// Faces share an axis when they're perpendicular to the same coordinate axis:
/// - Left/Right: both perpendicular to X-axis (points have fixed X, varying Y/Z)
/// - Top/Bottom: both perpendicular to Y-axis (points have fixed Y, varying X/Z)
/// - Front/Back: both perpendicular to Z-axis (points have fixed Z, varying X/Y)
///
/// Example: When drawing on Left face (x = -55) with Right overextended (x = 55),
/// the constraint `point.x > 55` is impossible (point.x is fixed at -55).
/// Skipping this check is a performance optimization.
fn faces_share_axis(face1: BoundaryFace, face2: BoundaryFace) -> bool {
    use BoundaryFace::*;
    matches!(
        (face1, face2),
        // Same face (optimization for redundant self-checks)
        (Left, Left) | (Right, Right) |
        (Top, Top) | (Bottom, Bottom) |
        (Front, Front) | (Back, Back) |
        // Opposite faces on same axis
        (Left, Right) | (Right, Left) |
        (Top, Bottom) | (Bottom, Top) |
        (Front, Back) | (Back, Front)
    )
}
```

**Placement**: After `intersect_circle_with_line_segment()` (currently line ~631-657)

**Note**: These are production functions (no `#[cfg(test)]`) because they'll be used in portal-upgraded.md Step 2.

---

## Phase 2: Add Test Infrastructure

### Test Helper: `calculate_portal_render_data()`

Add test-only method to extract render decisions without drawing (after helper functions, before test module):

```rust
#[cfg(test)]
impl Boundary {
    /// Test helper: Get portal rendering data without drawing
    pub fn calculate_portal_render_data(&self, portal: &Portal) -> PortalRenderData {
        let overextended_faces = self.get_overextended_faces_for(portal);

        if overextended_faces.is_empty() {
            return PortalRenderData::SingleCircle {
                position: portal.position,
                normal: portal.normal,
                radius: portal.radius,
            };
        }

        let intersection_data = self.get_overextended_intersection_points(
            portal,
            overextended_faces.clone()
        );

        PortalRenderData::SplitArcs {
            primary_face: BoundaryFace::from_normal(portal.normal).unwrap(),
            arc_data: intersection_data,
        }
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq)]
pub enum PortalRenderData {
    SingleCircle {
        position: Vec3,
        normal: Dir3,
        radius: f32,
    },
    SplitArcs {
        primary_face: BoundaryFace,
        arc_data: Vec<(BoundaryFace, Vec<Vec3>)>,
    },
}
```

### Test Module Scaffolding

Add at end of `src/playfield/boundary.rs`:

```rust
#[cfg(test)]
mod portal_render_tests {
    use super::*;

    fn create_test_boundary() -> Boundary {
        Boundary {
            cell_count: UVec3::new(1, 1, 1),
            scalar: 110.,
            transform: Transform::from_scale(Vec3::splat(110.)),
            ..default()
        }
        // Boundary extends from -55 to 55 on all axes
    }

    fn create_portal(position: Vec3, radius: f32, normal: Dir3) -> Portal {
        Portal {
            position,
            radius,
            normal,
            ..default()
        }
    }

    // Helper to check if intersection points are properly constrained
    fn verify_points_constrained(
        points: &[Vec3],
        face: BoundaryFace,
        overextended_faces: &[BoundaryFace],
        boundary_min: Vec3,
        boundary_max: Vec3,
    ) {
        let constrained = constrain_intersection_points(
            points.to_vec(),
            face,
            overextended_faces,
            &boundary_min,
            &boundary_max,
        );

        assert_eq!(
            points.len(),
            constrained.len(),
            "Some intersection points extend beyond face boundaries"
        );
    }

    // Tests go here...
}
```

---

## Phase 3: Test Cases

### Category 1: Too Far (1 test)

Portal center far from boundaries, radius doesn't reach any boundary.

```rust
#[test]
fn test_portal_too_far_from_boundary_no_overextension() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::ZERO, 10.0, Dir3::X);

    let render_data = boundary.calculate_portal_render_data(&portal);

    match render_data {
        PortalRenderData::SingleCircle { .. } => {
            // Expected - portal doesn't reach boundaries
        },
        _ => panic!("Expected single circle for portal far from boundaries"),
    }
}
```

**Expected**: ✅ PASS

---

### Category 2: Single Face (6 tests)

Portal on one face, doesn't extend to edges. Test all 6 faces.

```rust
#[test]
fn test_portal_approaching_single_face_right_wall() {
    let boundary = create_test_boundary();
    let portal = create_portal(
        Vec3::new(54.99, 0.0, 0.0),  // Close to right wall, inside boundary
        5.0,                          // Small radius, won't overextend
        Dir3::X,
    );

    let render_data = boundary.calculate_portal_render_data(&portal);

    match render_data {
        PortalRenderData::SingleCircle { normal, .. } => {
            assert_eq!(normal, Dir3::X);
        },
        _ => panic!("Expected single circle on right face"),
    }
}

#[test]
fn test_portal_approaching_single_face_left_wall() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-54.99, 0.0, 0.0), 5.0, Dir3::NEG_X);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SingleCircle { normal, .. } => {
            assert_eq!(normal, Dir3::NEG_X);
        },
        _ => panic!("Expected single circle on left face"),
    }
}

#[test]
fn test_portal_approaching_single_face_top_wall() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, 54.99, 0.0), 5.0, Dir3::Y);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SingleCircle { normal, .. } => {
            assert_eq!(normal, Dir3::Y);
        },
        _ => panic!("Expected single circle on top face"),
    }
}

#[test]
fn test_portal_approaching_single_face_bottom_wall() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, -54.99, 0.0), 5.0, Dir3::NEG_Y);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SingleCircle { normal, .. } => {
            assert_eq!(normal, Dir3::NEG_Y);
        },
        _ => panic!("Expected single circle on bottom face"),
    }
}

#[test]
fn test_portal_approaching_single_face_front_wall() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, 0.0, 54.99), 5.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SingleCircle { normal, .. } => {
            assert_eq!(normal, Dir3::Z);
        },
        _ => panic!("Expected single circle on front face"),
    }
}

#[test]
fn test_portal_approaching_single_face_back_wall() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, 0.0, -54.99), 5.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SingleCircle { normal, .. } => {
            assert_eq!(normal, Dir3::NEG_Z);
        },
        _ => panic!("Expected single circle on back face"),
    }
}
```

**Expected**: ✅ All 6 tests PASS

---

### Category 3: Edge Cases (12 tests)

Portal overextends to 2 faces at a boundary edge. Test all 12 edges.

#### X-axis Edges (4 tests)

```rust
#[test]
fn test_portal_at_top_back_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(
        Vec3::new(0.0, 54.99, -54.99),
        15.0,
        Dir3::NEG_Z,
    );

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2, "Expected 2 faces at edge");

            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Top));

            for (face, points) in &arc_data {
                assert!(points.len() >= 2, "Face {face:?} needs >= 2 points");
            }
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_top_front_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, 54.99, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_bottom_back_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, -54.99, -54.99), 15.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_bottom_front_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(0.0, -54.99, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}
```

#### Y-axis Edges (4 tests)

```rust
#[test]
fn test_portal_at_left_back_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-54.99, 0.0, -54.99), 15.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Left));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_left_front_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-54.99, 0.0, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Left));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_right_back_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(54.99, 0.0, -54.99), 15.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Right));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_right_front_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(54.99, 0.0, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Right));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}
```

#### Z-axis Edges (4 tests)

```rust
#[test]
fn test_portal_at_left_top_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-54.99, 54.99, 0.0), 15.0, Dir3::NEG_X);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Left));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_left_bottom_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-54.99, -54.99, 0.0), 15.0, Dir3::NEG_X);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Left));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_right_top_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(54.99, 54.99, 0.0), 15.0, Dir3::X);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Right));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}

#[test]
fn test_portal_at_right_bottom_edge() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(54.99, -54.99, 0.0), 15.0, Dir3::X);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 2);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Right));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at edge"),
    }
}
```

**Expected**: ✅ All 12 tests PASS

---

### Category 4: Corner Cases (8 tests) - EXPECTED TO FAIL

Portal overextends to 3 faces at a boundary corner. Test all 8 corners.

**Why these fail**: Current code only draws primary arc + arcs for overextended faces found by `get_overextended_intersection_points()`. At corners, the primary face's intersection points include portions that belong on other faces, causing visual artifacts. The bug is documented in portal-upgraded.md.

```rust
#[test]
fn test_portal_at_left_bottom_back_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(
        Vec3::new(-50.0, -50.0, -54.99),
        15.0,
        Dir3::NEG_Z,
    );

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { primary_face, arc_data } => {
            assert_eq!(primary_face, BoundaryFace::Back);
            assert_eq!(arc_data.len(), 3, "Expected 3 faces at corner");

            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Left));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_left_bottom_front_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-50.0, -50.0, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Left));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_left_top_back_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-50.0, 50.0, -54.99), 15.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Left));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_left_top_front_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(-50.0, 50.0, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Left));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_right_bottom_back_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(50.0, -50.0, -54.99), 15.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Right));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_right_bottom_front_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(50.0, -50.0, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Right));
            assert!(faces.contains(&BoundaryFace::Bottom));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_right_top_back_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(50.0, 50.0, -54.99), 15.0, Dir3::NEG_Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Back));
            assert!(faces.contains(&BoundaryFace::Right));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}

#[test]
fn test_portal_at_right_top_front_corner() {
    let boundary = create_test_boundary();
    let portal = create_portal(Vec3::new(50.0, 50.0, 54.99), 15.0, Dir3::Z);

    match boundary.calculate_portal_render_data(&portal) {
        PortalRenderData::SplitArcs { arc_data, .. } => {
            assert_eq!(arc_data.len(), 3);
            let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
            assert!(faces.contains(&BoundaryFace::Front));
            assert!(faces.contains(&BoundaryFace::Right));
            assert!(faces.contains(&BoundaryFace::Top));
        },
        _ => panic!("Expected split arcs at corner"),
    }
}
```

**Expected**: ❌ All 8 tests FAIL (confirms corner rendering bug)

---

## Phase 4: Build, Format, and Run Tests

### Build and Format

```bash
cargo build && cargo +nightly fmt
```

### Run Tests

```bash
cargo nextest run portal_render_tests
```

### Expected Results Summary

| Category | Tests | Expected Result |
|----------|-------|-----------------|
| Too Far  | 1     | ✅ PASS         |
| Single Face | 6  | ✅ PASS         |
| Edge (2 faces) | 12 | ✅ PASS      |
| Corner (3 faces) | 8 | ❌ FAIL       |
| **Total** | **27** | **19 pass, 8 fail** |

The 8 failing corner tests document the bug that will be fixed by portal-upgraded.md Steps 2-4.

---

## Summary

### Files Modified

1. **`src/playfield/boundary.rs`**:
   - Add 3 production helper functions (lines ~658-713)
   - Add test infrastructure with `#[cfg(test)]`
   - Add 27 comprehensive tests

### Test Coverage

- **27 total tests** covering all portal rendering scenarios
- 1 "too far" test
- 6 single-face tests (all 6 boundary faces)
- 12 edge tests (all 12 boundary edges)
- 8 corner tests (all 8 boundary corners)

### Benefits

- ✅ Validates current correct behavior (tests 1-3)
- ✅ Documents corner bug with failing tests (test 4)
- ✅ Completes portal-upgraded.md Step 1 early
- ✅ Provides regression suite for future refactor
- ✅ No Bevy runtime dependencies
- ✅ Fast execution with `cargo nextest`

### Relationship to portal-upgraded.md

This test suite:
1. **Implements Step 1** (helper functions) of portal-upgraded.md
2. **Documents the bug** that Steps 2-4 will fix
3. **Provides regression tests** to validate the fix works
4. **Enables TDD workflow** - tests fail now, will pass after refactor

After completing portal-upgraded.md Steps 2-4, re-run these tests to verify all 27 pass.
