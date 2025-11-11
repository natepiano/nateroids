# Portal Corner Rendering Fix

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

### STEP 1: Add Helper Functions ⏳ PENDING

**Objective**: Add three new constraint filtering helper functions to boundary.rs

**Changes**:
- Add `constrain_intersection_points()` - filters intersection points to face boundaries
- Add `point_within_boundary_for_face()` - checks if point belongs to specific face
- Add `faces_share_axis()` - determines if two faces share perpendicular axis

**Files**: `src/playfield/boundary.rs`

**Location**: Add at end of file after `intersect_circle_with_line_segment()` (line ~631)

**Type**: Additive (new free functions, no existing code modified)

**Expected Build Status**: ✅ Compiles successfully (dead code warnings expected until Step 2)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

---

### STEP 2: Refactor draw_portal() Method ⏳ PENDING

**Objective**: Replace broken portal rendering loop with unified constraint-based algorithm

**Changes**:
- Remove call to `get_overextended_intersection_points()`
- Update early return check from `over_extended_intersection_points.is_empty()` to `overextended_faces.is_empty()`
- Replace old loop (with `primary_arc_drawn` flag) with unified constraint-based algorithm
- New algorithm: iterate over all faces (primary + overextended), apply constraints, draw arcs

**Files**: `src/playfield/boundary.rs` (lines ~221-282)

**Type**: Breaking (modifies existing method body)

**Dependencies**: Requires Step 1 (uses new helper functions)

**Expected Build Status**: ✅ Compiles successfully

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

---

### STEP 3: Delete Obsolete Method ⏳ PENDING

**Objective**: Remove `get_overextended_intersection_points()` method (now dead code)

**Changes**:
- Delete `get_overextended_intersection_points()` method

**Files**: `src/playfield/boundary.rs` (lines ~96-116)

**Type**: Cleanup (removes dead code after Step 2 refactor)

**Dependencies**: Requires Step 2 (ensures method no longer called)

**Expected Build Status**: ✅ Compiles successfully (removes dead code warnings)

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

---

### STEP 4: Complete Validation ⏳ PENDING

**Objective**: Verify corner portal rendering fix works correctly

**Validation Steps**:
1. Run application with dying nateroids
2. Verify corner portal rendering (3-arc splits at all 8 corners)
3. Verify edge portal rendering (2-arc splits still work)
4. Visual inspection: no overlaps, no missing arcs, clean arc boundaries
5. Check all success criteria from plan

**Build Command**:
```bash
cargo build && cargo +nightly fmt
```

---

## STEP 1 DETAILS: Add Helper Functions

### Placement
Add as free functions at the end of the file, after `intersect_circle_with_line_segment()` (currently line ~631).

**Rationale**: These are geometric helper functions that work with intersection points and boundaries, similar to the existing `intersect_circle_with_line_segment()` free function. Placing them together maintains logical code organization.

### Implementation

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

### Design Notes
- `constrain_intersection_points()` returns `Vec<Vec3>` directly (not `Option` or `Result`)
- Empty vec is a valid result when all points filtered out - not an error condition
- Caller handles `< 2 points` case with simple `len() >= 2` check
- This matches existing `intersect_circle_with_rectangle()` pattern in boundary.rs

---

## STEP 2 DETAILS: Refactor draw_portal() Method

### Algorithm Overview

The unified constraint-based algorithm:

```rust
// 1. Get primary face from portal normal
// Safe to unwrap: portals are always created with axis-aligned normals via snap_position_to_boundary_face(),
// so from_normal() will always return Some(). This invariant holds until boundary system is redesigned.
let primary_face = BoundaryFace::from_normal(portal.normal).unwrap();

// 2. Calculate intersections for primary + all overextended faces
let all_faces_to_draw = [primary_face].iter()
    .chain(overextended_faces.iter())
    .copied();

let mut face_arcs = Vec::new();

for face in all_faces_to_draw {
    let face_points = face.get_face_points(&min, &max);
    let raw_intersections = intersect_circle_with_rectangle(portal, &face_points);

    // 3. CONSTRAIN points to this face only
    let constrained_points = constrain_intersection_points(
        raw_intersections,
        face,
        &overextended_faces,
        &min,
        &max
    );

    if constrained_points.len() >= 2 {
        face_arcs.push((face, constrained_points));
    }
}

// 4. Draw all arcs
for (face, points) in face_arcs {
    if face == primary_face {
        self.draw_primary_arc(gizmos, portal, color, resolution, points[0], points[1]);
    } else {
        let rotated_center = self.rotate_portal_center_to_target_face(
            portal.position, portal.normal, face
        );
        gizmos.short_arc_3d_between(rotated_center, points[0], points[1], color);
    }
}
```

### Code to DELETE
- `get_overextended_intersection_points()` method (lines ~96-116) - replaced by inline intersection calculation
- Old loop with `primary_arc_drawn` flag (lines ~245-264) - replaced by unified constraint-based loop
- Method call to `get_overextended_intersection_points()` (lines ~229-232)

### Code to PRESERVE
**CRITICAL**: Keep the early return for no-overextension case, but update the check:

**After refactoring, use this pattern**:
```rust
// Get overextended faces first
let overextended_faces = self.get_overextended_faces_for(portal);

// Early return if no overextension - draw full circle
if overextended_faces.is_empty() {
    let rotation = Quat::from_rotation_arc(
        orientation.config.axis_profundus,
        portal.normal.as_vec3()
    );
    let isometry = Isometry3d::new(portal.position, rotation);
    gizmos.circle(isometry, portal.radius, color).resolution(resolution);
    return;
}

// Continue with unified constraint-based algorithm for split arcs...
```

**Key point**: Check `overextended_faces.is_empty()` IMMEDIATELY after computing it, BEFORE any intersection calculations. This preserves the performance optimization.

### Existing Functions - No Changes Required

The following existing functions work correctly with the new unified algorithm and require **NO modifications**:

1. **`rotate_portal_center_to_target_face()`** (lines ~271-311)
   - Purpose: Calculates wrapped arc center for non-primary faces
   - Why no changes: Already handles rotation from primary face to any target face
   - Usage: Called for wrapped arcs on overextended faces (unchanged from current implementation)

2. **`draw_primary_arc()`** (lines ~358-405)
   - Purpose: Draws the main arc on the portal's primary face
   - Why no changes: Works with any two intersection points, doesn't depend on how they're calculated
   - Usage: Called once for primary face with constrained points

3. **`get_overextended_faces_for()`** (lines ~414-436)
   - Purpose: Determines which boundaries the portal extends past
   - Why no changes: Detection logic remains the same, only the rendering changes
   - Usage: Called at start of `draw_portal()` (unchanged)

4. **`intersect_circle_with_rectangle()`** (lines ~476-530)
   - Purpose: Calculates raw intersection points between circle and rectangle
   - Why no changes: Returns unconstrained points as before, constraint filtering happens separately
   - Usage: Called for each face in the unified loop (same pattern as before, just different calling context)

**Scope of Changes**: Only `draw_portal()` (lines ~217-282) and the three new helper functions need modification. All other portal rendering infrastructure remains unchanged.

### Edge Case: Insufficient Points After Constraint Filtering

After applying constraints, some faces may have fewer than 2 valid intersection points:

- **When this occurs**: Portal positioned very close to a corner with small radius - constraint filtering may eliminate all points for some faces
- **Why this is correct**: The constraint filter correctly identified that no portion of the circle belongs to that face within valid boundaries
- **Implementation**: The `if constrained_points.len() >= 2` check handles this by skipping arc drawing (no arc on that face)
- **Testing**: Verify at extreme corners with small portals that no crashes occur and visual appearance is correct

---

## STEP 3 DETAILS: Delete Obsolete Method

### Method to Remove

Delete the `get_overextended_intersection_points()` method (lines ~96-116):

```rust
fn get_overextended_intersection_points(
    &self,
    portal: &Portal,
    overextended_faces: Vec<BoundaryFace>,
) -> Vec<(BoundaryFace, Vec<Vec3>)> {
    let mut intersections = Vec::new();
    let half_size = self.transform.scale / 2.0;
    let min = self.transform.translation - half_size;
    let max = self.transform.translation + half_size;

    for face in overextended_faces {
        let face_points = face.get_face_points(&min, &max);
        let face_intersections = intersect_circle_with_rectangle(portal, &face_points);

        if !face_intersections.is_empty() {
            intersections.push((face, face_intersections));
        }
    }

    intersections
}
```

**Why safe to delete**: After Step 2, this method is no longer called. The unified algorithm in `draw_portal()` calculates intersections inline.

---

## Problem Statement

Portal rendering at boundary corners is broken. When a portal extends into a corner (2+ dimensions), it needs to be split into 3 coplanar arcs but currently draws incorrect/overlapping arcs.

### Visual Issues
See `/tmp/portal_corner_issue.png` and `/tmp/upper_left_corner_case.png` for examples.

- Full circles protruding from corner faces instead of constrained arcs
- Missing arcs on some corner faces
- Overlapping portal arcs at corners
- Edge cases (1-dimension wrapping) work correctly

### Specific Example: Upper-Left Back Corner
From `/tmp/upper_left_corner_case.png`:
- ✅ Arc visible on back wall (x/y plane) - partially correct
- ❌ Full circle on left wall (y/z plane) - should be just an arc
- ❌ Missing arc on top wall (x/z plane) - should have an arc there
- Portal positioned on back wall extending into upper-left corner

## Root Cause Analysis

### Current Implementation
`draw_portal()` in `src/playfield/boundary.rs:217-282`:
```rust
for (face, points) in over_extended_intersection_points {
    if points.len() >= 2 {
        // Draw wrapped arc
        gizmos.short_arc_3d_between(rotated_position, points[0], points[1], color);

        // Draw primary arc using first face's points
        if !primary_arc_drawn {
            self.draw_primary_arc(gizmos, portal, color, resolution, points[0], points[1]);
            primary_arc_drawn = true;
        }
    }
}
```

**Issues:**
1. Uses first overextended face's intersection points for primary arc (wrong for corners)
2. `intersect_circle_with_rectangle()` returns unconstrained points
3. At corners, intersection points include portions that belong on different faces
4. No distinction between edge case (2-arc) vs corner case (3-arc)

### Example: Upper-Left Back Corner
Portal on Back wall extending past Left + Top boundaries:
- **Need**: 3 arcs (Back, Left, Top faces)
- **Get**: Full circle on Left wall, partial arc on Back wall, missing Top wall arc
- **Why**: Left wall intersection includes points above top boundary that should be on Top wall

## Solution: Unified Constraint-Based Algorithm

### Key Insight
Instead of recalculating intersections or branching on edge vs corner, **constrain** the existing intersection points based on which boundaries are exceeded.

### Implementation Details for Algorithm

**Variable Naming and Types:**

```rust
// Type: BoundaryFace
// Purpose: Identifies which face the portal is primarily on (based on its normal)
let primary_face: BoundaryFace = BoundaryFace::from_normal(portal.normal).unwrap();

// Type: impl Iterator<Item = BoundaryFace>
// Purpose: Combines primary face + overextended faces into single iteration
// Details: [primary_face].iter() creates iterator over single-element array,
//          .chain() appends overextended_faces iterator,
//          .copied() converts from &BoundaryFace to BoundaryFace (Copy trait)
let all_faces_to_draw = [primary_face].iter()
    .chain(overextended_faces.iter())
    .copied();

// Type: Vec<(BoundaryFace, Vec<Vec3>)>
// Purpose: Collects all faces that have valid arcs (>= 2 constrained points)
// Structure: Each tuple contains (face to draw on, intersection points for that face)
let mut face_arcs: Vec<(BoundaryFace, Vec<Vec3>)> = Vec::new();

// Inside loop - Type: Vec<Vec3>
// Purpose: Raw intersection points before constraint filtering (may extend into adjacent faces)
let raw_intersections: Vec<Vec3> = intersect_circle_with_rectangle(portal, &face_points);

// Inside loop - Type: Vec<Vec3>
// Purpose: Filtered points that belong only to current face (after applying constraints)
// Note: May have < 2 points after filtering (portal too small/close to corner) - handled by len() check
let constrained_points: Vec<Vec3> = constrain_intersection_points(
    raw_intersections,
    face,
    &overextended_faces,
    &min,
    &max
);
```

**Iterator Chain Explanation:**

The expression `[primary_face].iter().chain(overextended_faces.iter()).copied()` works as follows:

1. `[primary_face]` - Creates single-element array on stack (cheap)
2. `.iter()` - Borrows array, yields `&BoundaryFace`
3. `.chain(overextended_faces.iter())` - Appends second iterator, still yields `&BoundaryFace`
4. `.copied()` - Dereferences each `&BoundaryFace` to `BoundaryFace` (valid because `BoundaryFace` implements `Copy`)

**Why this pattern?**
- Ensures primary face is always processed first (important for drawing order)
- Avoids allocating a new `Vec` to combine faces
- Lazy evaluation - iterator items computed on-demand

**Alternative (less efficient):**
```rust
// Don't do this - allocates unnecessary Vec
let mut all_faces = vec![primary_face];
all_faces.extend(overextended_faces.iter());
```

**Collection Structure:**

The `face_arcs` vector stores tuples of `(BoundaryFace, Vec<Vec3>)`:
- Used to separate arc collection phase from arc drawing phase
- Allows filtering out faces with insufficient points before drawing
- Preserves face-to-points association for drawing loop

Example contents after collection:
```rust
face_arcs = vec![
    (BoundaryFace::Back, vec![Vec3::new(-40.0, 55.0, -55.0), Vec3::new(-55.0, 50.0, -55.0)]),
    (BoundaryFace::Left, vec![Vec3::new(-55.0, 50.0, -55.0), Vec3::new(-55.0, 40.0, -48.0)]),
    (BoundaryFace::Top, vec![Vec3::new(-55.0, 55.0, -50.0), Vec3::new(-40.0, 55.0, -55.0)]),
];
```

### The Constraint Function

```rust
/// Filters intersection points to only include those within the face's boundary limits.
/// At corners, this prevents arcs from extending into adjacent faces.
fn constrain_intersection_points(
    points: Vec<Vec3>,
    current_face: BoundaryFace,
    overextended_faces: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> Vec<Vec3> {
    points.into_iter().filter(|point| {
        // Check point is within bounds for all non-primary dimensions
        point_within_boundary_for_face(*point, current_face, overextended_faces, min, max)
    }).collect()
}

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
            continue; // Same axis, no constraint needed
        }

        // Check if point exceeds the boundary this overextended face represents
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

### Concrete Constraint Examples

**Example 1: Back Face with Left Overextension**
- Portal on Back face (z = min.z, e.g., z = -55)
- Left boundary overextended (portal extends past x = min.x)
- Constraint applied to Back face points:
  ```rust
  // For BoundaryFace::Left overextension:
  if point.x < min.x { return false; }
  ```
- **Result**: Points with x < -55 are filtered out (they belong on Left face, not Back face)
- **Visual**: On the Back face (z = -55 plane), only keep points with x >= -55

**Example 2: Left Face with Top Overextension**
- Portal on Left face (x = min.x, e.g., x = -55)
- Top boundary overextended (portal extends past y = max.y)
- Constraint applied to Left face points:
  ```rust
  // For BoundaryFace::Top overextension:
  if point.y > max.y { return false; }
  ```
- **Result**: Points with y > 55 are filtered out (they belong on Top face, not Left face)
- **Visual**: On the Left face (x = -55 plane), only keep points with y <= 55

**Example 3: Skipped Constraint (Optimization)**
- Drawing on Left face (x = min.x = -55)
- Right boundary overextended (portal extends past x = max.x = 55)
- Constraint would be: `if point.x > 55 { return false; }`
- **Why skipped**: All points on Left face have x = -55, geometrically impossible to have x > 55
- **Optimization**: `faces_share_axis(Left, Right)` returns true, skips redundant check

### Complete Constraint Mapping Table

| Overextended Face | Constraint Applied | Filters Points Where | Applied When Drawing On |
|-------------------|-------------------|----------------------|------------------------|
| Left (x = min.x)  | `point.x < min.x` | Point past left edge | Right, Top, Bottom, Front, Back |
| Right (x = max.x) | `point.x > max.x` | Point past right edge | Left, Top, Bottom, Front, Back |
| Bottom (y = min.y)| `point.y < min.y` | Point past bottom edge | Left, Right, Top, Front, Back |
| Top (y = max.y)   | `point.y > max.y` | Point past top edge | Left, Right, Bottom, Front, Back |
| Back (z = min.z)  | `point.z < min.z` | Point past back edge | Left, Right, Top, Bottom, Front |
| Front (z = max.z) | `point.z > max.z` | Point past front edge | Left, Right, Top, Bottom, Back |

**Note**: Constraints are NOT applied when current face shares axis with overextended face (skipped via `faces_share_axis()` check).

### Why Exact Boundaries (No Epsilon)?

Constraint checks use **exact boundary comparisons** while overextension detection uses epsilon tolerance:

**Overextension Detection** (uses epsilon):
```rust
let epsilon = BOUNDARY_OVEREXTENSION_EPSILON; // 0.02
if portal.position.x - radius < min.x - epsilon {
    overextended_faces.push(BoundaryFace::Left);
}
```
- **Purpose**: Determine if portal needs arc splitting
- **Why epsilon**: Portals are snapped 0.01 inside boundary; without margin, exact boundary positions trigger false positives

**Constraint Filtering** (no epsilon):
```rust
BoundaryFace::Left => if point.x < min.x { return false; }
```
- **Purpose**: Determine which face owns each intersection point
- **Why no epsilon**: Intersection points are mathematically exact (calculated via quadratic formula). Adding epsilon would incorrectly filter valid points, creating visual gaps or overlaps at corners.

**Example Demonstrating Why Epsilon Would Break Constraints**:
```rust
// Portal at Back-Left corner: intersection point exactly at corner edge
let corner_point = Vec3::new(-55.0, 42.3, -55.0); // x exactly at min.x

// Correct (no epsilon):
if point.x < min.x { return false; }
// -55.0 < -55.0 = false → point kept on Back face ✓

// Incorrect (with epsilon = 0.02):
if point.x < min.x - epsilon { return false; }
// -55.0 < -55.02 = true → point incorrectly filtered ✗
// Result: Gap in arc rendering at corner!
```

## Benefits of This Approach

1. **No code duplication**: Single loop handles edges (2-arc) and corners (3-arc)
2. **No branching needed**: Works automatically based on `overextended_faces.len()`
3. **Handles all 8 corners**: Generic solution for any corner
4. **Handles all 12 edges**: Doesn't break existing edge case behavior
5. **Simple logic**: Filter points rather than recalculate intersections
6. **Extensible**: Would work even for theoretical 4-face cases

## Example Walkthrough: Upper-Left Back Corner

**Setup:**
- Portal on Back wall (normal = Dir3::NEG_Z)
- Position near upper-left corner
- Extends past Left boundary (x < min.x)
- Extends past Top boundary (y > max.y)

**Step 1: Identify faces**
- Primary face: Back
- Overextended faces: [Left, Top]
- Faces to draw: [Back, Left, Top]

**Step 2: Calculate raw intersections**
- Back wall: Returns 4 points (circle intersects all 4 edges)
- Left wall: Returns 4 points (circle intersects all 4 edges)
- Top wall: Returns 4 points (circle intersects all 4 edges)

**Step 3: Constrain points**
- Back wall points: Keep only those with `x >= min.x AND y <= max.y`
  - Result: 2 points (arc segment on back wall that doesn't extend into corner)

- Left wall points: Keep only those with `z == min.z AND y <= max.y`
  - Result: 2 points (arc segment on left wall, below top boundary)

- Top wall points: Keep only those with `z == min.z AND x >= min.x`
  - Result: 2 points (arc segment on top wall, right of left boundary)

**Step 4: Draw arcs**
- Primary arc on Back wall using constrained Back points
- Wrapped arc on Left wall using constrained Left points
- Wrapped arc on Top wall using constrained Top points

**Result:** 3 clean coplanar arcs, no overlaps, no protrusions ✓

## Files Modified

### `src/playfield/boundary.rs`
- Refactor `draw_portal()` (lines ~217-282)
- Add `constrain_intersection_points()` helper
- Add `point_within_boundary_for_face()` helper
- Add `faces_share_axis()` helper
- Delete `get_overextended_intersection_points()` method (lines ~96-116)

## Testing Strategy

1. Launch app with dying nateroids vectoring to back wall corners
2. Verify all 8 corners show proper 3-arc splits
3. Verify edges (middle of walls) still show proper 2-arc splits
4. Verify no arcs extend beyond boundary planes
5. Verify no missing arcs at any corner

## Additional Enhancement: Red Portals for Deaderoid

Separate smaller task already completed by user:
- Modified `draw_approaching_portals()` in `portals.rs`
- Query for `Option<&Deaderoid>`
- Use red color when entity is dying
