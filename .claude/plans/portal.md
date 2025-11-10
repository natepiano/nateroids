# Portal Corner Rendering Fix

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

### Algorithm Overview

```rust
// 1. Get primary face from portal normal
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
    // Determine which axes this face constrains
    let (primary_axis, other_axes) = match face {
        BoundaryFace::Left | BoundaryFace::Right => ('x', ['y', 'z']),
        BoundaryFace::Top | BoundaryFace::Bottom => ('y', ['x', 'z']),
        BoundaryFace::Back | BoundaryFace::Front => ('z', ['x', 'y']),
    };

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
        (Left, Right) | (Right, Left) |
        (Top, Bottom) | (Bottom, Top) |
        (Front, Back) | (Back, Front)
    )
}
```

## Benefits of This Approach

1. **No code duplication**: Single loop handles edges (2-arc) and corners (3-arc)
2. **No branching needed**: Works automatically based on `overextended_faces.len()`
3. **Handles all 8 corners**: Generic solution for any corner
4. **Handles all 12 edges**: Doesn't break existing edge case behavior
5. **Simple logic**: Filter points rather than recalculate intersections
6. **Extensible**: Would work even for theoretical 4-face cases

## Implementation Steps

### 1. Add helper functions to `boundary.rs`
- `constrain_intersection_points()`
- `point_within_boundary_for_face()`
- `faces_share_axis()`

### 2. Refactor `draw_portal()` in `boundary.rs:217-282`
- Remove current `primary_arc_drawn` flag logic
- Replace with unified constraint-based loop
- Calculate intersections for primary + overextended faces
- Apply constraints to each face's intersection points
- Draw all resulting arcs

### 3. Test cases to verify
- Edge cases: Portal extending past one boundary (2-arc split) ✓
- Corner cases: Portal extending past two boundaries (3-arc split)
- All 8 corners of the cuboid boundary
- No arcs extending beyond boundary rectangles
- No missing arcs at corners

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
