# Portal Drawing Code Simplification Plan

## Overview
Analysis of `src/playfield/boundary.rs` portal drawing logic, identifying duplication and opportunities for self-documenting enums.

## Phase Status

### ✅ Phase 1: Named Constants (COMPLETED)
Added constants for magic numbers and colors:
- `MIN_POINTS_FOR_ARC = 2`
- `MIN_FACES_FOR_CORNER = 3`
- `DEADEROID_APPROACHING_COLOR`, `CORNER_COLOR_LEFT_RIGHT_YZ`, `CORNER_COLOR_TOP_BOTTOM_XZ`, `CORNER_COLOR_FRONT_BACK_XY`

Replaced at lines: 244, 306, 312, 319-321, 325

### ✅ Phase 2: PortalGeometry Enum (COMPLETED)
Added self-documenting enum (lines 31-46):
- `PortalGeometry::SingleFace` - portal within one face
- `PortalGeometry::Edge { primary, overextended }` - portal spans 2 faces
- `PortalGeometry::Corner { primary, overextended }` - portal spans 3+ faces

Added `classify_portal_geometry()` method (lines 122-140) to `Boundary` impl

### ✅ Phase 3: Refactor draw_portal() (COMPLETED)
Refactored `draw_portal()` to use `PortalGeometry` pattern matching:
- Replaced `get_overextended_faces_for()` + `is_empty()` check with enum matching
- Each geometry type handled explicitly through match arms
- Eliminated `is_corner` boolean flag - replaced with pattern matching on geometry
- Eliminated need for `unreachable!()` - all cases handled through type system
- Extracted `draw_portal_arcs()` helper method to handle Edge/Corner rendering
- Color selection now pattern matches on `PortalGeometry` instead of boolean conditionals

Benefits: Code is self-documenting, rendering paths are explicit, impossible states eliminated

### ✅ Phase 4: Nested Enum Design (COMPLETED)
Redesigned `PortalGeometry` with nested `MultiFaceGeometry` enum to eliminate panic!/unreachable! branches:

**New structure:**
```rust
enum PortalGeometry {
    SingleFace,
    MultiFace(MultiFaceGeometry),
}

enum MultiFaceGeometry {
    Edge { primary, overextended },
    Corner { primary, overextended },
}
```

**Changes:**
- `draw_portal_arcs()` now takes `&MultiFaceGeometry` instead of `&PortalGeometry`
- Type system prevents `SingleFace` from being passed to multi-face rendering logic
- Eliminated all panic! and unreachable! branches - no dead code
- `render_portal_by_geometry()` cleanly dispatches: SingleFace → inline, MultiFace → helper

**Benefits:**
- Type-safe by design - compiler prevents invalid states
- No runtime checks for "impossible" cases
- Clear separation: single-face vs multi-face rendering
- Each function only handles cases it actually receives

### ✅ Phase 5: Eliminate `.unwrap()` and Remove Redundant Portal Fields (COMPLETED)

**Problem:** Portal struct had redundant data - both `face: BoundaryFace` and `normal: Dir3` representing the same information. This required `.unwrap()` calls when converting between them, violating cargo.toml lints (`unwrap_used = "deny"`).

**Changes:**
- Added infallible `to_dir3()` method to `BoundaryFace` (src/playfield/boundary_face.rs:28-38)
- Removed redundant `normal: Dir3` field from Portal struct
- Added `normal()` helper method to Portal impl that derives normal from face
- Updated `classify_portal_geometry()` to use `portal.face` directly (line 132) - no conversion needed!
- Fixed all Portal construction sites in portals.rs (lines 269, 305, 333)
- Changed `portal.normal` field access to `portal.normal()` method call (line 378)
- Refactored `snap_and_get_normal()` → `snap_and_get_face()` to only return needed values
- Refactored `calculate_portal_face_count()` to use `classify_portal_geometry()` and enum matching
- Extracted `count_faces_with_valid_arcs()` helper method

**Benefits:**
- Zero `.unwrap()` or `.expect()` calls - complies with cargo.toml lints
- No elided variables - function signatures match actual usage patterns
- Portal struct has single source of truth for face/normal information
- Eliminated all duplication between `draw_portal()` and `calculate_portal_face_count()` (~40 lines)
- More maintainable through shared `classify_portal_geometry()` method

## Issues Identified

### 1. Major Code Duplication (~80 lines)

#### Duplicated Setup (Lines 201-217 vs 252-278)
Both `calculate_portal_face_count()` and `draw_portal()` have identical initialization:
- Call `get_overextended_faces_for(portal)`
- Early return/action if empty
- Calculate boundary extents (`half_size`, `min`, `max`)
- Get primary face from normal
- Build `all_faces_in_corner` vector

#### Duplicated Face Processing Loop (Lines 222-238 vs 283-300)
Both methods iterate with identical logic:
```rust
for &face in &all_faces_in_corner {
    let face_points = face.get_face_points(&min, &max);
    let raw_intersections = intersect_circle_with_rectangle(portal, &face_points);
    let constrained_points = constrain_intersection_points(...);

    if constrained_points.len() >= 2 {
        // One counts, one collects
    }
}
```

### 2. Boolean Flags That Should Be Enums

#### `is_corner = face_arcs.len() >= 3` (Line 303)
- Magic number encoding geometric knowledge
- Used to switch between corner and edge rendering
- Obscures that there are 3+ distinct portal configurations

#### `overextended_faces.is_empty()` (Lines 204, 255)
- Represents 3 distinct portal states as binary check
- Should be explicit enum: SingleFace/Edge/Corner

#### `is_deaderoid` nested conditionals (Lines 306-320)
- Legitimate domain flag, but nested conditions obscure color logic
- Should extract to helper method with enum-based dispatch

### 3. Hidden Rendering Modes (Lines 303-349)

Current deeply nested structure obscures **4 distinct rendering paths**:

1. **Single Face (full circle)** - Lines 255-263
   - Early return with full circle draw

2. **Edge Primary Face** - Lines 324-335
   - `face == primary_face && !is_corner`
   - Uses complex `draw_arc_with_center_and_normal` with TAU inversion

3. **Edge Overextended Faces** - Lines 336-348 (else branch)
   - `!is_corner` but not primary face
   - Uses `rotate_portal_center_to_target_face`

4. **Corner (all faces)** - Lines 336-348 (same else branch)
   - `is_corner` condition
   - Uses portal position directly as center

### 4. Magic Numbers

- `face_arcs.len() >= 3` → should be `MIN_FACES_FOR_CORNER`
- `constrained_points.len() >= 2` → should be `MIN_POINTS_FOR_ARC`
- `Color::srgb(1.0, 0.0, 0.0)` etc → should be named diagnostic color constants

## Proposed Solution

### New Self-Documenting Enums

```rust
/// Describes the geometric configuration of a portal relative to boundary faces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalGeometry {
    /// Portal completely within a single boundary face
    SingleFace,
    /// Portal extends across an edge (2 faces)
    Edge {
        primary: BoundaryFace,
        overextended: BoundaryFace
    },
    /// Portal extends into a corner (3+ faces)
    Corner {
        primary: BoundaryFace,
        overextended: Vec<BoundaryFace>
    },
}

/// Rendering mode for portal arcs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArcRenderMode {
    /// Full circle on single face
    FullCircle,
    /// Primary face arc at edge using complex angle inversion
    EdgePrimary,
    /// Overextended face arc at edge using rotated center
    EdgeOverextended,
    /// Corner face arc using portal center directly
    CornerFace,
}

/// Color scheme for diagnostic portal rendering
#[derive(Debug, Clone, Copy)]
enum PortalColorScheme {
    Normal(Color),
    DiagnosticEdge,
    DiagnosticCorner { face: BoundaryFace },
}
```

### Shared Data Structures

```rust
/// Intermediate calculation results shared between face counting and drawing
struct PortalFaceAnalysis {
    geometry: PortalGeometry,
    boundary_extents: BoundaryExtents,
    face_arcs: Vec<FaceArc>,
}

struct BoundaryExtents {
    half_size: Vec3,
    min: Vec3,
    max: Vec3,
}

struct FaceArc {
    face: BoundaryFace,
    points: Vec<Vec3>,
    render_mode: ArcRenderMode,
}
```

### Named Constants

```rust
const MIN_POINTS_FOR_ARC: usize = 2;
const MIN_FACES_FOR_CORNER: usize = 3;

// Diagnostic colors
const DIAGNOSTIC_COLOR_X_AXIS: Color = Color::srgb(1.0, 0.0, 0.0);  // Red
const DIAGNOSTIC_COLOR_Y_AXIS: Color = Color::srgb(0.0, 1.0, 0.0);  // Green
const DIAGNOSTIC_COLOR_Z_AXIS: Color = Color::srgb(1.0, 1.0, 0.0);  // Yellow
const DIAGNOSTIC_COLOR_EDGE: Color = Color::srgb(1.0, 0.0, 0.0);    // Red
```

### New Method Structure

```rust
impl Boundary {
    /// Core analysis method - extracts shared logic
    fn analyze_portal_geometry(&self, portal: &Portal) -> PortalFaceAnalysis {
        let overextended_faces = self.get_overextended_faces_for(portal);

        let geometry = if overextended_faces.is_empty() {
            PortalGeometry::SingleFace
        } else if overextended_faces.len() == 1 {
            PortalGeometry::Edge {
                primary: BoundaryFace::from_normal(portal.normal).unwrap(),
                overextended: overextended_faces[0],
            }
        } else {
            PortalGeometry::Corner {
                primary: BoundaryFace::from_normal(portal.normal).unwrap(),
                overextended: overextended_faces,
            }
        };

        let boundary_extents = BoundaryExtents {
            half_size: self.transform.scale / 2.0,
            min: self.transform.translation - (self.transform.scale / 2.0),
            max: self.transform.translation + (self.transform.scale / 2.0),
        };

        let face_arcs = match geometry {
            PortalGeometry::SingleFace => vec![],
            PortalGeometry::Edge { primary, overextended } |
            PortalGeometry::Corner { primary, overextended } => {
                self.calculate_face_arcs(portal, &geometry, &boundary_extents)
            }
        };

        PortalFaceAnalysis {
            geometry,
            boundary_extents,
            face_arcs,
        }
    }

    /// Simplified face count using analysis
    pub fn calculate_portal_face_count(&self, portal: &Portal) -> usize {
        let analysis = self.analyze_portal_geometry(portal);
        match analysis.geometry {
            PortalGeometry::SingleFace => 1,
            _ => analysis.face_arcs.len(),
        }
    }

    /// Simplified drawing using analysis
    pub fn draw_portal(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        orientation: &CameraOrientation,
        is_deaderoid: bool
    ) {
        let analysis = self.analyze_portal_geometry(portal);

        match analysis.geometry {
            PortalGeometry::SingleFace => {
                self.draw_full_circle(gizmos, portal, color, resolution, orientation);
            }
            _ => {
                for arc in analysis.face_arcs {
                    let arc_color = self.determine_arc_color(
                        &analysis.geometry,
                        arc.face,
                        color,
                        is_deaderoid
                    );
                    self.render_arc(gizmos, portal, &arc, arc_color, resolution);
                }
            }
        }
    }

    /// Extracts shared intersection calculation loop
    fn calculate_face_arcs(
        &self,
        portal: &Portal,
        geometry: &PortalGeometry,
        extents: &BoundaryExtents,
    ) -> Vec<FaceArc> {
        let all_faces = match geometry {
            PortalGeometry::SingleFace => return vec![],
            PortalGeometry::Edge { primary, overextended } => {
                vec![*primary, *overextended]
            }
            PortalGeometry::Corner { primary, overextended } => {
                let mut faces = vec![*primary];
                faces.extend(overextended);
                faces
            }
        };

        let mut face_arcs = Vec::new();
        let primary_face = match geometry {
            PortalGeometry::Edge { primary, .. } |
            PortalGeometry::Corner { primary, .. } => *primary,
            _ => unreachable!(),
        };

        for &face in &all_faces {
            let face_points = face.get_face_points(&extents.min, &extents.max);
            let raw_intersections = intersect_circle_with_rectangle(portal, &face_points);

            let constrained_points = constrain_intersection_points(
                raw_intersections,
                face,
                &all_faces,
                &extents.min,
                &extents.max,
            );

            if constrained_points.len() >= MIN_POINTS_FOR_ARC {
                let render_mode = self.determine_render_mode(geometry, face, primary_face);
                face_arcs.push(FaceArc {
                    face,
                    points: constrained_points,
                    render_mode,
                });
            }
        }

        face_arcs
    }

    /// Determines which rendering technique to use for an arc
    fn determine_render_mode(
        &self,
        geometry: &PortalGeometry,
        face: BoundaryFace,
        primary_face: BoundaryFace,
    ) -> ArcRenderMode {
        match geometry {
            PortalGeometry::SingleFace => ArcRenderMode::FullCircle,
            PortalGeometry::Edge { .. } => {
                if face == primary_face {
                    ArcRenderMode::EdgePrimary
                } else {
                    ArcRenderMode::EdgeOverextended
                }
            }
            PortalGeometry::Corner { .. } => ArcRenderMode::CornerFace,
        }
    }

    /// Extracts color determination logic
    fn determine_arc_color(
        &self,
        geometry: &PortalGeometry,
        face: BoundaryFace,
        base_color: Color,
        is_deaderoid: bool
    ) -> Color {
        if !is_deaderoid {
            return base_color;
        }

        match geometry {
            PortalGeometry::SingleFace => base_color,
            PortalGeometry::Edge { .. } => DIAGNOSTIC_COLOR_EDGE,
            PortalGeometry::Corner { .. } => match face {
                BoundaryFace::Left | BoundaryFace::Right => DIAGNOSTIC_COLOR_X_AXIS,
                BoundaryFace::Top | BoundaryFace::Bottom => DIAGNOSTIC_COLOR_Y_AXIS,
                BoundaryFace::Front | BoundaryFace::Back => DIAGNOSTIC_COLOR_Z_AXIS,
            }
        }
    }

    /// Unified arc rendering dispatch
    fn render_arc(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        arc: &FaceArc,
        color: Color,
        resolution: u32
    ) {
        match arc.render_mode {
            ArcRenderMode::FullCircle => {
                unreachable!("Full circle handled separately in draw_portal")
            }
            ArcRenderMode::EdgePrimary => {
                self.draw_arc_with_center_and_normal(
                    gizmos,
                    portal.position,
                    portal.radius,
                    portal.normal.as_vec3(),
                    color,
                    resolution,
                    arc.points[0],
                    arc.points[1]
                );
            }
            ArcRenderMode::EdgeOverextended => {
                let center = self.rotate_portal_center_to_target_face(
                    portal.position,
                    portal.normal,
                    arc.face
                );
                gizmos
                    .short_arc_3d_between(center, arc.points[0], arc.points[1], color)
                    .resolution(resolution);
            }
            ArcRenderMode::CornerFace => {
                gizmos
                    .short_arc_3d_between(
                        portal.position,
                        arc.points[0],
                        arc.points[1],
                        color
                    )
                    .resolution(resolution);
            }
        }
    }

    /// Extracts full circle drawing logic
    fn draw_full_circle(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        orientation: &CameraOrientation,
    ) {
        let rotation = Quat::from_rotation_arc(
            orientation.config.axis_profundus,
            portal.normal.as_vec3()
        );
        let isometry = Isometry3d::new(portal.position, rotation);
        gizmos
            .circle(isometry, portal.radius, color)
            .resolution(resolution);
    }
}
```

## Benefits

| Benefit | Impact |
|---------|--------|
| **Eliminates ~80 lines duplication** | Better maintainability |
| **Makes portal states explicit** | `PortalGeometry` enum documents the 3 configurations |
| **Makes rendering paths explicit** | `ArcRenderMode` enum documents the 4 rendering techniques |
| **Type safety** | Impossible states become unrepresentable |
| **Self-documenting** | Enum variants replace boolean conditionals |
| **Easier testing** | Can test each geometry/render mode independently |
| **Easier extension** | Add new portal types by extending enums |

## Summary Table

| Issue Type | Location | Problem | Proposed Solution |
|------------|----------|---------|-------------------|
| **Duplication** | Lines 201-217 & 252-278 | Identical setup code | Extract to `analyze_portal_geometry()` |
| **Duplication** | Lines 222-238 & 283-300 | Identical loop structure | Share `calculate_face_arcs()` helper |
| **Boolean Flag** | Line 303 | `is_corner` obscures 4 render modes | `PortalGeometry` + `ArcRenderMode` enums |
| **Boolean Flag** | Lines 204, 255 | `.is_empty()` represents 3 states | `PortalGeometry::SingleFace/Edge/Corner` |
| **Magic Number** | Line 303 | `>= 3` encodes corner knowledge | `MIN_FACES_FOR_CORNER` constant |
| **Magic Number** | Lines 235, 297 | `>= 2` for arc validity | `MIN_POINTS_FOR_ARC` constant |
| **Magic Colors** | Lines 310-312 | Hardcoded RGB values | Named color constants |
| **Nested Logic** | Lines 303-349 | 4 render paths obscured | `ArcRenderMode` enum + `render_arc()` dispatch |
