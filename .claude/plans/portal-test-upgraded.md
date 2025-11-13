# Portal Rendering Test Suite Implementation Plan

## EXECUTION PROTOCOL

<Instructions>
For each step in the implementation sequence:

1. **DESCRIBE**: Present the changes with:
   - Summary of what will change and why
   - Code examples showing before/after
   - List of files to be modified
   - Expected impact on the system

2. **AWAIT APPROVAL**: Stop and wait for user confirmation ("go ahead" or similar)

3. **IMPLEMENT**: Make the changes and stop

4. **BUILD & VALIDATE**: Execute the build process:
   ```bash
   cargo build && cargo +nightly fmt
   ```
   For test steps, also run:
   ```bash
   cargo nextest run portal_render_tests
   ```

5. **CONFIRM**: Wait for user to confirm the build succeeded

6. **MARK COMPLETE**: Update this document to mark the step as ✅ COMPLETED

7. **PROCEED**: Move to next step only after confirmation
</Instructions>

<ExecuteImplementation>
    Find the next ⏳ PENDING step in the INTERACTIVE IMPLEMENTATION SEQUENCE below.

    For the current step:
    1. Follow the <Instructions/> above for executing the step
    2. When step is complete, use Edit tool to mark it as ✅ COMPLETED
    3. Continue to next PENDING step

    If all steps are COMPLETED:
        Display: "✅ Implementation complete! All steps have been executed."
</ExecuteImplementation>

## INTERACTIVE IMPLEMENTATION SEQUENCE

### Step 1: Add Helper Functions ⏳ PENDING

**Objective**: Implement 3 constraint helper functions for intersection point filtering

**Changes**:
- Add `constrain_intersection_points()` - Filters intersection points to only include those within face boundary limits
- Add `point_within_boundary_for_face()` - Checks if a point extends beyond boundaries
- Add `faces_share_axis()` - Determines if two faces are perpendicular to the same axis

**Files**: `src/playfield/boundary.rs`

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

**Expected Result**:
- ✅ Compiles successfully
- ⚠️ Dead code warnings expected (functions will be used in portal-upgraded.md Step 2)

**See Implementation Details**: Phase 1 section below

---

### Step 2: Add Test Infrastructure ⏳ PENDING

**Objective**: Implement test-only method and scaffolding for portal rendering tests

**Changes**:
- Add `#[cfg(test)]` impl block with `calculate_portal_render_data()` method
- Add `PortalRenderData` enum type for test results
- Add test module with helper functions (`create_test_boundary()`, `create_portal()`)

**Files**: `src/playfield/boundary.rs`

**Dependencies**: Requires Step 1 (uses helper functions)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

**Expected Result**: ✅ Compiles successfully

**See Implementation Details**: Phase 2 section below

---

### Step 3: Add Category 1 & 2 Tests (Too Far + Single Face) ⏳ PENDING

**Objective**: Implement 7 tests for basic portal rendering scenarios

**Changes**:
- Add 1 "too far" test (portal doesn't reach boundaries)
- Add 6 "single face" tests (one for each boundary face)

**Files**: `src/playfield/boundary.rs`

**Dependencies**: Requires Step 2 (test infrastructure)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

**Test Command**:
```bash
cargo nextest run portal_render_tests
```

**Expected Result**:
- ✅ Compiles successfully
- ✅ All 7 tests PASS

**See Implementation Details**: Phase 3, Category 1 & 2 sections below

---

### Step 4: Add Category 3 Tests (Edge Cases) ⏳ PENDING

**Objective**: Implement 12 tests for portal rendering at boundary edges

**Changes**:
- Add 4 X-axis edge tests (top/bottom + back/front combinations)
- Add 4 Y-axis edge tests (left/right + back/front combinations)
- Add 4 Z-axis edge tests (left/right + top/bottom combinations)

**Files**: `src/playfield/boundary.rs`

**Dependencies**: Requires Step 2 (test infrastructure)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

**Test Command**:
```bash
cargo nextest run portal_render_tests
```

**Expected Result**:
- ✅ Compiles successfully
- ✅ All 12 tests PASS

**See Implementation Details**: Phase 3, Category 3 section below

---

### Step 5: Add Category 4 Tests (Corner Cases) ⏳ PENDING

**Objective**: Implement 8 tests for portal rendering at boundary corners (EXPECTED TO FAIL)

**Changes**:
- Add 8 corner tests covering all boundary corners
- These tests document the known corner rendering bug

**Files**: `src/playfield/boundary.rs`

**Dependencies**: Requires Step 2 (test infrastructure)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

**Test Command**:
```bash
cargo nextest run portal_render_tests
```

**Expected Result**:
- ✅ Compiles successfully
- ❌ All 8 tests FAIL (expected - documents corner rendering bug)

**Note**: These failures are intentional and document the bug that will be fixed by portal-upgraded.md Steps 2-4

**See Implementation Details**: Phase 3, Category 4 section below

---

### Step 6: Final Validation ⏳ PENDING

**Objective**: Run complete test suite and verify expected results

**Tasks**:
- Run all tests
- Verify 19 tests pass (categories 1-3)
- Verify 8 tests fail (category 4 - corner bug)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

**Test Command**:
```bash
cargo nextest run portal_render_tests
```

**Expected Results**:

| Category | Tests | Expected Result |
|----------|-------|-----------------|
| Too Far  | 1     | ✅ PASS         |
| Single Face | 6  | ✅ PASS         |
| Edge (2 faces) | 12 | ✅ PASS      |
| Corner (3 faces) | 8 | ❌ FAIL       |
| **Total** | **27** | **19 pass, 8 fail** |

---

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

---

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

Add three helper functions to `src/playfield/boundary.rs` immediately after line 657 (after the closing brace of `intersect_circle_with_line_segment()` function):

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

**Placement**: Immediately after line 657 (the closing brace of `intersect_circle_with_line_segment()` which spans lines 631-657)

**Note**: These are production functions (no `#[cfg(test)]`) because they'll be used in portal-upgraded.md Step 2. The Rust compiler will generate **dead code warnings** for these functions after Phase 1 completes, since they won't be called until portal-upgraded.md executes. This is expected and can be ignored - these warnings will disappear once portal-upgraded.md Step 2 uses them in the refactored rendering code.

---

## Phase 2: Add Test Infrastructure

### Test Helper: `calculate_portal_render_data()`

Add test-only method to extract render decisions without drawing (after helper functions, before test module):

**IMPORTANT**: This implementation is **self-contained** and does NOT depend on `get_overextended_intersection_points()` (which will be deleted in portal-upgraded.md Step 3). Instead, it directly calculates intersections using the low-level geometry functions and the new constraint helpers from Phase 1.

```rust
#[cfg(test)]
impl Boundary {
    /// Test helper: Get portal rendering data without drawing
    ///
    /// This is a self-contained implementation that doesn't depend on
    /// `get_overextended_intersection_points()` (which will be removed
    /// in the portal-upgraded.md refactor). It directly calculates
    /// intersections and applies constraints using the Phase 1 helpers.
    pub fn calculate_portal_render_data(&self, portal: &Portal) -> PortalRenderData {
        let overextended_faces = self.get_overextended_faces_for(portal);

        if overextended_faces.is_empty() {
            return PortalRenderData::SingleCircle {
                position: portal.position,
                normal: portal.normal,
                radius: portal.radius,
            };
        }

        // Calculate boundary extents for constraint checking
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;

        let primary_face = BoundaryFace::from_normal(portal.normal).unwrap();
        let mut arc_data = Vec::new();

        // Collect ALL faces that need arcs (primary + overextended)
        let mut all_faces = vec![primary_face];
        all_faces.extend(overextended_faces.iter());

        // Calculate intersections for each face
        for &face in &all_faces {
            let face_points = face.get_face_points(&min, &max);
            let mut face_intersections = intersect_circle_with_rectangle(portal, &face_points);

            // Apply constraints: filter out points that extend beyond face boundaries
            face_intersections = constrain_intersection_points(
                face_intersections,
                face,
                &overextended_faces,
                &min,
                &max,
            );

            if !face_intersections.is_empty() {
                arc_data.push((face, face_intersections));
            }
        }

        PortalRenderData::SplitArcs {
            primary_face,
            arc_data,
        }
    }
}

/// Test helper return type that captures complete portal rendering data.
///
/// **Design rationale**: This type intentionally includes ALL data that the real
/// `draw_portal()` function uses, even though current tests only validate subsets:
///
/// 1. **Debugging value**: When tests fail, developers can inspect the complete
///    rendering state (position, radius, normals, intersection points) to understand
///    what went wrong, not just which assertion failed.
///
/// 2. **Future test expansion**: Additional tests may validate position accuracy
///    (e.g., verifying `snap_position_to_boundary_face()` worked correctly) or
///    radius constraints (e.g., portals don't exceed max size).
///
/// 3. **Mirrors production code**: Type structure matches what `draw_portal()`
///    actually uses, making it a true "rendering data snapshot" rather than a
///    minimal test assertion type.
///
/// **Current test usage**: Tests validate rendering strategy (single circle vs
/// split arcs) and face selection, using pattern matching with `..` to ignore
/// geometric parameters that are already known (input values).
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
    use crate::playfield::portals::Portal;

    // Note: The following types are available via `use super::*;`:
    // - Boundary, BoundaryFace (from boundary.rs)
    // - Vec3, Dir3, UVec3, Transform, default() (from bevy prelude)
    // - constrain_intersection_points() and other helper functions (from Phase 1)

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

    // Tests go here...
}
```

**Note on Test Scope:** These tests validate **rendering outcomes** (single circle vs split arcs, correct face counts) rather than intermediate constraint filtering behavior. The `constrain_intersection_points()` helper is used internally by `calculate_portal_render_data()` (lines 173-179 above), but tests don't directly validate constraint filtering. They verify that the entire rendering pipeline produces correct visual results. Constraint filtering is an implementation detail that's tested indirectly through outcome validation.

---

## Phase 3: Test Cases

**Test Validation Scope:** These tests intentionally validate **rendering strategy** (single circle vs split arcs, correct face selection) rather than geometric correctness of intersection points. Specifically, tests do NOT validate:

- **Exact point coordinates**: Tests don't assert specific x/y/z values for intersection points
- **Points staying within boundaries**: Constraint filtering is handled by `constrain_intersection_points()` helper and tested indirectly
- **`primary_face` matching portal normal**: Implicit in test setup (portal normal determines face)
- **Individual point positions**: Tests use `..` pattern to ignore point data, focusing on arc counts

This is intentional because:

1. **Tests focus on high-level rendering decisions**, not low-level geometry calculations
2. **Geometric parameters are test inputs**: Validating output coordinates against input position/radius would be circular testing
3. **Constraint filtering tested indirectly**: If constraints fail, arc counts or face selection would be wrong (caught by assertions)
4. **Maintainability**: Geometric coordinate validation would be brittle and require complex epsilon comparisons for floating-point arithmetic

**What tests DO validate:**
- Correct rendering path chosen (SingleCircle vs SplitArcs)
- Correct number of faces rendered (1 for single face, 2 for edges, 3 for corners)
- Correct face identification (Back, Top, Left, etc.)
- Sufficient intersection points per arc (>= 2 points needed to draw an arc)

If detailed geometric validation is needed in the future, add separate unit tests for the constraint helpers (`constrain_intersection_points()`, `point_within_boundary_for_face()`, etc.) rather than expanding these integration tests.

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
