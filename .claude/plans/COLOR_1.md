# COLOR_1 Vertex Attribute Warning

## Issue
Getting warning from Bevy when loading assets:
```
WARN bevy_gltf::loader: Unknown vertex attribute COLOR_1
```

## Source
**File**: `assets/nateroid/nateroid.glb`
**Mesh**: "icing_mesh" (the icing part of the donut)

The mesh has both `COLOR_0` and `COLOR_1` vertex color attributes:
```json
"attributes": {
  "POSITION": 0,
  "NORMAL": 1,
  "TEXCOORD_0": 2,
  "TANGENT": 3,
  "COLOR_0": 4,
  "COLOR_1": 5
}
```

The donut mesh doesn't have this issue - only the icing.

## Why Multiple Vertex Color Layers Exist

Common reasons in Blender:
1. **Baked lighting data**: Different layers for different bake types
   - `COLOR_0`: Vertex colors or baked diffuse lighting
   - `COLOR_1`: Ambient occlusion (AO), shadow data, or secondary lighting
2. **Shader masks**: Layers to drive different shader effects (roughness, metallic, emission)
3. **Non-destructive workflow**: Multiple versions kept during modeling
4. **Accidental**: Created unintentionally during workflow

Since the nateroid already has baked PBR textures (normal, albedo, metallic_roughness), the vertex colors are likely leftover from the baking process.

## Impact
**None** - This is just a warning. Bevy ignores `COLOR_1` and uses only `COLOR_0`. The game renders correctly.

## How to Fix (If Desired)

### Option 1: Remove in Blender
1. Open `assets/nateroid/nateroid.glb` in Blender
2. Select the icing mesh
3. Go to Mesh properties → Vertex Colors / Color Attributes
4. Delete the second color layer
5. Re-export to GLB

### Option 2: Just Ignore It
The warning is harmless and doesn't affect functionality.

## Other Files
Checked all other GLB files - they don't have this issue:
- ✅ `Planet.glb` - no vertex colors
- ✅ `Bullets Pickup.glb` - no vertex colors
- ✅ `spaceship.glb` - no vertex colors
- ⚠️  `nateroid.glb` - has COLOR_0 and COLOR_1 on icing mesh
