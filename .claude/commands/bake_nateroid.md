---
description: Prepare and bake nateroid PBR textures from Blender source
---

# Bake Nateroid Textures

This command runs the complete texture baking workflow for the nateroid model.

## Prerequisites Check

First, check if the Blender MCP server is available by looking for `mcp__blender__*` tools in your available tools.

**If Blender MCP tools are NOT available:**

Enable the Blender MCP server:
```bash
~/.claude/scripts/mcp --add blender
```

Then **restart Claude** to pick up the MCP server, and run `/bake_nateroid` again.

---

**If Blender MCP tools ARE available**, proceed with the baking workflow:

## Step 1: Prepare Source File

Run the preparation script to convert the procedural source into a bake-ready file:

```bash
cd ~/rust/nateroids/assets/blend
/opt/homebrew/bin/blender --background nateroid.blend --python prepare_for_bake.py
```

This will:
- Extract ColorRamp data from the Sprinkles material
- Apply geometry nodes to realize sprinkle instances
- Convert random values to vertex colors
- Update materials to use vertex colors
- UV unwrap and pack
- Save as `nateroid_bake.blend`

## Step 2: Bake Textures

Run the baking script to generate all PBR texture maps:

```bash
cd ~/rust/nateroids/assets/blend
/opt/homebrew/bin/blender --background nateroid_bake.blend \
  --python ~/.claude/scripts/bake_textures.py -- nateroid_bake_config.json
```

This will generate:
- Albedo textures (using EMIT baking for correct colors)
- Normal maps
- Metallic maps
- Roughness maps
- Ambient occlusion maps
- Metallic-roughness packed textures (glTF/Bevy format)
- nateroid.glb export

## Step 3: Verify Output

Check that all textures were generated in `~/rust/nateroids/assets/nateroid/textures/`:
- nateroid_donut_albedo.png
- nateroid_donut_normal.png
- nateroid_donut_metallic_roughness.png
- nateroid_donut_ao.png
- nateroid_icing_albedo.png
- nateroid_icing_normal.png
- nateroid_icing_metallic_roughness.png
- nateroid_icing_ao.png

The donut albedo should show light tan color (not dark brown), and the icing albedo should show multi-colored sprinkles.

## Next Steps

After baking completes successfully:
1. Rebuild the Bevy game: `cargo build`
2. Run the game to see the updated textures
3. Verify that:
   - Donut appears light tan/beige (not dark brown)
   - Sprinkles appear metallic (not plastic)
   - Colors match the Blender source file

---

**Note:** This baking process takes approximately 1-2 minutes. You can monitor progress in the terminal output.
