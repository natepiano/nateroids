# Bevy Text Mesh - Three.js Port Plan

Port Three.js `TextGeometry` and `ExtrudeGeometry` to Rust for use in Bevy, enabling true 3D volumetric text meshes from TTF/OTF fonts.

## Background

Three.js provides 3D text via:
- `TextGeometry.js` (~70 lines) - thin wrapper
- `Font.generateShapes()` (~150 lines) - converts glyphs to 2D bezier paths
- `ExtrudeGeometry.js` (~650 lines) - converts 2D shapes to 3D meshes with extrusion/beveling
- `ShapeUtils.js` (~100 lines) - triangulation wrapper around Earcut

## Rust Crate Dependencies

| Purpose | Crate | Notes |
|---------|-------|-------|
| Font parsing | `ttf-parser` | Extracts glyph outlines as bezier curves directly from TTF/OTF |
| Curve flattening | `lyon_geom` or `kurbo` | Converts bezier curves to line segments |
| Triangulation | `earcut` | Rust port of MapBox's Earcut algorithm |
| 3D math | `glam` | Already used by Bevy |

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────────┐     ┌────────────┐
│  TTF/OTF    │────▶│  ttf-parser  │────▶│  lyon/kurbo     │────▶│  earcut    │
│  Font File  │     │  (outlines)  │     │  (flatten)      │     │  (front/   │
└─────────────┘     └──────────────┘     └─────────────────┘     │  back face)│
                                                                  └─────┬──────┘
                                                                        │
                    ┌──────────────┐     ┌─────────────────┐           │
                    │  Bevy Mesh   │◀────│  ExtrudeGeom    │◀──────────┘
                    │  (final)     │     │  (sides/bevels) │
                    └──────────────┘     └─────────────────┘
```

## Implementation Phases

### Phase 1: Core Infrastructure (~2-3 days)

1. **Create crate structure**
   ```
   bevy_text_mesh/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── font.rs          # Font loading and glyph extraction
       ├── shape.rs         # 2D shape representation
       ├── extrude.rs       # ExtrudeGeometry port
       └── text_mesh.rs     # TextMesh component and systems
   ```

2. **Define core types**
   ```rust
   // 2D contour (outer boundary or hole)
   struct Contour {
       points: Vec<Vec2>,
       is_hole: bool,
   }

   // 2D shape with optional holes
   struct Shape {
       outer: Contour,
       holes: Vec<Contour>,
   }

   // Extrusion parameters
   struct ExtrudeSettings {
       depth: f32,
       bevel_enabled: bool,
       bevel_thickness: f32,
       bevel_size: f32,
       bevel_segments: u32,
       curve_segments: u32,  // bezier flattening tolerance
   }
   ```

3. **Implement font → shapes conversion**
   - Load TTF with `ttf-parser`
   - Extract glyph outlines (bezier curves)
   - Flatten curves to polygons using `lyon_geom::CubicBezierSegment::for_each_flattened()`
   - Handle glyph advance/kerning for text layout

### Phase 2: ExtrudeGeometry Port (~3-4 days)

Port the core algorithm from Three.js `ExtrudeGeometry.js`:

1. **Basic extrusion (no bevels)**
   - Triangulate front face with `earcut`
   - Duplicate vertices at `z = depth` for back face
   - Generate side walls by connecting front/back contour vertices
   - Compute normals

2. **Bevel support**
   - Port `getBevelVec()` - calculates perpendicular offset vectors
   - Generate intermediate vertex layers for bevel steps
   - Handle acute/obtuse angle cases

3. **Key functions to port:**
   ```
   buildLidFaces()      → triangulate front/back with earcut
   buildSideFaces()     → connect contour vertices across depth
   sidewalls()          → generate side wall triangles
   getBevelVec()        → compute bevel offset direction
   scalePt2()           → apply bevel offset to vertex
   ```

### Phase 3: Bevy Integration (~2 days)

1. **Asset loader for fonts**
   ```rust
   #[derive(Asset, TypePath)]
   struct TextMeshFont {
       face: ttf_parser::Face<'static>,
   }
   ```

2. **Component API**
   ```rust
   #[derive(Component)]
   struct TextMesh {
       text: String,
       font: Handle<TextMeshFont>,
       size: f32,
       depth: f32,
       bevel: Option<BevelSettings>,
   }
   ```

3. **Mesh generation system**
   - React to `TextMesh` component changes
   - Generate/update `Mesh` asset
   - Cache generated meshes by (text, font, settings) key

### Phase 4: Polish (~2-3 days)

1. **Text layout**
   - Multi-line support
   - Alignment (left, center, right)
   - Line height control

2. **Performance**
   - Mesh caching with LRU eviction
   - Async mesh generation for large text
   - LOD support (reduce segments at distance)

3. **Additional features**
   - Per-character transforms (for animation)
   - Outline-only mode (no front/back faces)
   - Path extrusion (text along curve)

## ExtrudeGeometry Algorithm Details

### Side Wall Generation

```rust
fn build_side_walls(
    front_contour: &[Vec2],
    depth: f32,
    vertices: &mut Vec<Vec3>,
    indices: &mut Vec<u32>,
) {
    let n = front_contour.len();
    let base = vertices.len() as u32;

    // Add front and back contour vertices
    for p in front_contour {
        vertices.push(Vec3::new(p.x, p.y, 0.0));
    }
    for p in front_contour {
        vertices.push(Vec3::new(p.x, p.y, depth));
    }

    // Create quads (as 2 triangles) connecting front to back
    for i in 0..n {
        let next = (i + 1) % n;
        let f0 = base + i as u32;           // front current
        let f1 = base + next as u32;        // front next
        let b0 = base + n as u32 + i as u32;     // back current
        let b1 = base + n as u32 + next as u32;  // back next

        // Two triangles per quad
        indices.extend_from_slice(&[f0, b0, f1]);
        indices.extend_from_slice(&[f1, b0, b1]);
    }
}
```

### Bevel Vector Calculation

```rust
/// Calculate the direction to offset a vertex for beveling.
/// This finds the bisector of the angle between two edges.
fn get_bevel_vec(
    prev: Vec2,
    curr: Vec2,
    next: Vec2,
) -> Vec2 {
    // Edge vectors
    let v1 = (curr - prev).normalize();
    let v2 = (next - curr).normalize();

    // Perpendiculars (rotate 90°)
    let n1 = Vec2::new(-v1.y, v1.x);
    let n2 = Vec2::new(-v2.y, v2.x);

    // Bisector direction
    let bisector = (n1 + n2).normalize();

    // Scale factor to maintain consistent bevel width at corners
    let dot = n1.dot(bisector);
    if dot.abs() > 0.001 {
        bisector / dot
    } else {
        n1 // Fallback for ~180° angles
    }
}
```

## Reference Materials

- Three.js ExtrudeGeometry: https://github.com/mrdoob/three.js/blob/dev/src/geometries/ExtrudeGeometry.js
- Three.js FontLoader/Font: https://github.com/mrdoob/three.js/blob/dev/examples/jsm/loaders/FontLoader.js
- ttf-parser docs: https://docs.rs/ttf-parser
- earcut docs: https://docs.rs/earcut
- lyon_geom docs: https://docs.rs/lyon_geom

## Existing Bevy Crates (Reference)

- `bevy_text_mesh` - Uses C-based ttf2mesh, stale at Bevy 0.12
- `bevy_rich_text3d` - 2D quads only, not volumetric 3D

## Success Criteria

- [ ] Load TTF/OTF fonts directly (no conversion step)
- [ ] Generate volumetric 3D meshes from text
- [ ] Support depth extrusion
- [ ] Support beveled edges
- [ ] Proper normals for lighting
- [ ] Mesh caching for performance
- [ ] Clean Bevy component API
- [ ] No C/FFI dependencies (pure Rust)
