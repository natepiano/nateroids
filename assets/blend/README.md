# Nateroids Blend Files

This directory contains two versions of the nateroids model:

## Files

### `nateroid.blend` - Editable Source File
**Use this file for:** Editing, modeling, adjusting sprinkles

This is the master source file with procedural geometry nodes. The sprinkles on the icing are generated using a Geometry Nodes modifier that instances a sphere template.

**Features:**
- Procedural sprinkle generation via Geometry Nodes
- Easy to adjust sprinkle distribution, size, and density
- Non-destructive workflow
- Includes the Sphere template object used for instancing

**Objects:**
- `sphere sprinkle donut.002` - Donut base mesh
- `sphere sprinkle icing.002` - Icing mesh with Geometry Nodes modifier
- `Sphere` - Template object used by Geometry Nodes to generate sprinkles

### `nateroid_bake.blend` - Bake-Ready Production File
**Use this file for:** PBR texture baking and game asset export

This is a production-ready version where all procedural modifiers have been applied and the geometry is "realized" (converted to actual mesh data). This file is required for texture baking because:
- UV unwrapping and baking require real mesh geometry
- Geometry node instances cannot be directly baked
- The sprinkles are now permanent geometry on the icing mesh

**Features:**
- All modifiers applied/realized
- Ready for UV unwrapping and texture baking
- Optimized for export to game engines
- No procedural elements

**Objects:**
- `sphere sprinkle donut.002` - Donut base mesh
- `sphere sprinkle icing.002` - Icing mesh with realized sprinkles (41,104 vertices)

**Missing:**
- `Sphere` template object (deleted, no longer needed)
- Geometry Nodes modifier (applied and removed)

## Workflow

### Editing Sprinkles
1. Open `nateroid.blend`
2. Select the icing object (`sphere sprinkle icing.002`)
3. Modify the Geometry Nodes in the modifier panel
4. Adjust sprinkle distribution, density, rotation, etc.
5. Save changes

### Regenerating Bake File (After Editing)
If you modify the sprinkles in `nateroid.blend`, you need to regenerate `nateroid_bake.blend`:

#### CRITICAL: Color Preservation Issue
The sprinkles use `Object Info -> Random` in the material to get random colors. When instances are realized into a single mesh, they lose per-object randomness and all become the same color. **You must follow this exact process to preserve colors:**

#### Automated Process (via Claude with Blender MCP)
Ask Claude to regenerate the bake file - it will handle color preservation automatically.

#### Manual Process (Step-by-Step)
**Important:** Do NOT simply apply the geometry nodes modifier, or you will lose the colors!

1. **Open** `nateroid.blend`

2. **Modify Geometry Nodes** to preserve random values:
   - Open Geometry Editor for the icing object's "round sprinkles geometry" node group
   - Find the connection: `Join Geometry -> Group Output`
   - Insert these nodes in this exact order:
     1. **Store Named Attribute** node:
        - Set Domain: `INSTANCE` (critical!)
        - Set Data Type: `FLOAT`
        - Name field: `"sprinkle_random"`
     2. **Random Value** node:
        - Connect `Random Value.Value` -> `Store Named Attribute.Value`
     3. **Realize Instances** node
   - Final flow: `Join Geometry -> Store Named Attribute -> Realize Instances -> Group Output`

3. **Update Sprinkles Material** to read the stored attribute:
   - Open Shader Editor for the "Sprinkles" material
   - Find the `Object Info` node
   - Add an **Attribute** node:
     - Set attribute_name: `"sprinkle_random"`
   - Reconnect: Change all `Object Info.Random` connections to use `Attribute.Fac`
   - The attribute node replaces Object Info.Random

4. **Verify colors** in viewport - sprinkles should be multi-colored

5. **Apply modifier**:
   - Select icing object
   - Apply the Geometry Nodes modifier (Object > Apply > Geometry Nodes)

6. **Delete** the Sphere template object

7. **Save As** `nateroid_bake.blend`

**Why this works:**
- Store Named Attribute captures a random value for each sprinkle INSTANCE before realization
- Realize Instances converts instances to mesh geometry, preserving the stored attribute
- The material reads the attribute instead of Object Info, which only has one value per object

### Baking Textures
Use `nateroid_bake.blend` as the source file in your bake configuration:

```json
{
  "blend_file": "/Users/natemccoy/rust/nateroids/assets/blend/nateroid_bake.blend",
  "objects": ["sphere sprinkle donut.002", "sphere sprinkle icing.002"],
  "output_name": "nateroids",
  ...
}
```

## Technical Details

**Geometry Statistics:**

| Object | Editable Version | Bake Version |
|--------|-----------------|--------------|
| Donut | 1,536 vertices | 1,536 vertices |
| Icing | Variable (procedural) | 41,104 vertices (realized with sprinkles) |
| Sphere Template | 86 vertices | Deleted |

**File Sizes:**
- `nateroid.blend`: ~4.9 MB (with geometry nodes)
- `nateroid_bake.blend`: ~7.3 MB (with realized geometry)

## Important Notes

- **Never use `nateroid_bake.blend` for editing** - It has no procedural controls
- **Always edit `nateroid.blend`** - This is your source of truth
- **Regenerate bake file** whenever you change sprinkles in the editable version
- The bake file is essentially a "compiled" version of your editable file
