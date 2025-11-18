"""
Prepare nateroid.blend for baking by converting sprinkle attributes to vertex colors.

This script:
1. Opens nateroid.blend
2. Applies geometry nodes with attribute preservation
3. Converts sprinkle_random attribute to vertex colors
4. Updates Sprinkles material to use vertex colors
5. Saves as nateroid_bake.blend

Usage:
    blender --background --python prepare_for_bake.py
"""

from __future__ import annotations

import sys
from pathlib import Path

import bpy  # pyright: ignore[reportMissingImports]

# Paths
script_dir = Path(__file__).parent
source_blend = script_dir / "nateroid.blend"
output_blend = script_dir / "nateroid_bake.blend"

print(f"Opening source file: {source_blend}")
bpy.ops.wm.open_mainfile(filepath=str(source_blend))  # pyright: ignore[reportUnknownMemberType]

# Verify donut texture is loaded
donut = bpy.data.objects.get("donut")  # pyright: ignore[reportUnknownMemberType]
if donut and donut.material_slots:  # pyright: ignore[reportUnknownMemberType]
    mat = donut.material_slots[0].material  # pyright: ignore[reportUnknownMemberType]
    print(f"Donut material: {mat.name if mat else 'None'}")  # pyright: ignore[reportUnknownMemberType]

# Get icing object
icing = bpy.data.objects.get("icing")  # pyright: ignore[reportUnknownMemberType]
if not icing:
    print("ERROR: icing object not found")
    sys.exit(1)

print(f"Found icing object: {icing.name}")  # pyright: ignore[reportUnknownMemberType]

# Extract actual ColorRamp colors from Sprinkles material on Sphere template
print("Extracting ColorRamp colors from Sprinkles material...")
sphere = bpy.data.objects.get("Sphere")  # pyright: ignore[reportUnknownMemberType]
if not sphere:
    print("ERROR: Sphere template object not found")
    sys.exit(1)

sprinkles_mat = None
for mat_slot in sphere.material_slots:  # pyright: ignore[reportUnknownMemberType]
    mat = mat_slot.material  # pyright: ignore[reportUnknownMemberType]
    if mat and "Sprinkles" in mat.name:  # pyright: ignore[reportUnknownMemberType]
        sprinkles_mat = mat
        break

if not sprinkles_mat:
    print("ERROR: Sprinkles material not found on Sphere")
    sys.exit(1)

# Find the ColorRamp node in the material
color_ramp_node = None
for node in sprinkles_mat.node_tree.nodes:  # pyright: ignore[reportUnknownMemberType]
    if node.type == 'VALTORGB':  # pyright: ignore[reportUnknownMemberType]
        color_ramp_node = node
        break

if not color_ramp_node:
    print("ERROR: ColorRamp node not found in Sprinkles material")
    sys.exit(1)

# Extract color stops from the ColorRamp
color_stops = []
for element in color_ramp_node.color_ramp.elements:  # pyright: ignore[reportUnknownMemberType]
    pos = element.position  # pyright: ignore[reportUnknownMemberType]
    color = element.color  # pyright: ignore[reportUnknownMemberType]
    color_stops.append((pos, (color[0], color[1], color[2])))  # pyright: ignore[reportUnknownMemberType]

print(f"  Found {len(color_stops)} color stops in ColorRamp")
for i, (pos, color) in enumerate(color_stops):
    print(f"    Stop {i}: position={pos:.3f}, color=({color[0]:.3f}, {color[1]:.3f}, {color[2]:.3f})")

# Modify geometry nodes to preserve random values
print("Setting up geometry nodes to preserve sprinkle colors...")
bpy.context.view_layer.objects.active = icing  # pyright: ignore[reportUnknownMemberType]

geonodes_modifier = None
for modifier in icing.modifiers:  # pyright: ignore[reportUnknownMemberType]
    if modifier.type == 'NODES':  # pyright: ignore[reportUnknownMemberType]
        geonodes_modifier = modifier
        break

if not geonodes_modifier:
    print("ERROR: No geometry nodes modifier found on icing")
    sys.exit(1)

node_group = geonodes_modifier.node_group  # pyright: ignore[reportUnknownMemberType]
if not node_group:
    print("ERROR: Geometry nodes modifier has no node group")
    sys.exit(1)

nodes = node_group.nodes  # pyright: ignore[reportUnknownMemberType]
links = node_group.links  # pyright: ignore[reportUnknownMemberType]

# Find Instance on Points node
instance_node = None
for node in nodes:  # pyright: ignore[reportUnknownMemberType]
    if node.type == 'INSTANCE_ON_POINTS':  # pyright: ignore[reportUnknownMemberType]
        instance_node = node
        break

if not instance_node:
    print("WARNING: No Instance on Points node found, trying to apply as-is...")
else:
    print(f"  Found Instance on Points node: {instance_node.name}")  # pyright: ignore[reportUnknownMemberType]

    # Create nodes to preserve random values
    # 1. Random Value node
    random_node = nodes.new(type='FunctionNodeRandomValue')  # pyright: ignore[reportUnknownMemberType]
    random_node.data_type = 'FLOAT'  # pyright: ignore[reportUnknownMemberType]
    random_node.inputs['Min'].default_value = 0.0  # pyright: ignore[reportUnknownMemberType]
    random_node.inputs['Max'].default_value = 1.0  # pyright: ignore[reportUnknownMemberType]
    random_node.location = (instance_node.location[0] + 250, instance_node.location[1] - 100)  # pyright: ignore[reportUnknownMemberType]

    # 2. Store Named Attribute node
    store_node = nodes.new(type='GeometryNodeStoreNamedAttribute')  # pyright: ignore[reportUnknownMemberType]
    store_node.domain = 'INSTANCE'  # pyright: ignore[reportUnknownMemberType]
    store_node.data_type = 'FLOAT'  # pyright: ignore[reportUnknownMemberType]
    store_node.inputs['Name'].default_value = "sprinkle_random"  # pyright: ignore[reportUnknownMemberType]
    store_node.location = (instance_node.location[0] + 450, instance_node.location[1])  # pyright: ignore[reportUnknownMemberType]

    # 3. Realize Instances node
    realize_node = nodes.new(type='GeometryNodeRealizeInstances')  # pyright: ignore[reportUnknownMemberType]
    realize_node.location = (instance_node.location[0] + 650, instance_node.location[1])  # pyright: ignore[reportUnknownMemberType]

    # Find what the Instance on Points was connected to
    output_socket = instance_node.outputs['Instances']  # pyright: ignore[reportUnknownMemberType]
    connected_nodes = []
    for link in output_socket.links:  # pyright: ignore[reportUnknownMemberType]
        connected_nodes.append((link.to_node, link.to_socket))  # pyright: ignore[reportUnknownMemberType]
        links.remove(link)  # pyright: ignore[reportUnknownMemberType]

    # Create new connections
    links.new(instance_node.outputs['Instances'], store_node.inputs['Geometry'])  # pyright: ignore[reportUnknownMemberType]
    links.new(random_node.outputs['Value'], store_node.inputs['Value'])  # pyright: ignore[reportUnknownMemberType]
    links.new(store_node.outputs['Geometry'], realize_node.inputs['Geometry'])  # pyright: ignore[reportUnknownMemberType]

    # Reconnect to original targets
    for to_node, to_socket in connected_nodes:
        links.new(realize_node.outputs['Geometry'], to_socket)  # pyright: ignore[reportUnknownMemberType]

    print("  Added Random Value -> Store Named Attribute -> Realize Instances nodes")

# Apply geometry nodes modifier
print("Applying geometry nodes modifier...")
bpy.ops.object.modifier_apply(modifier=geonodes_modifier.name)  # pyright: ignore[reportUnknownMemberType]

print(f"Icing mesh now has {len(icing.data.vertices)} vertices")  # pyright: ignore[reportUnknownMemberType]

# Check if sprinkle_random attribute exists
mesh = icing.data  # pyright: ignore[reportUnknownMemberType]
if "sprinkle_random" not in mesh.attributes:  # pyright: ignore[reportUnknownMemberType]
    print("WARNING: sprinkle_random attribute not found on mesh")
    print("Available attributes:", [attr.name for attr in mesh.attributes])  # pyright: ignore[reportUnknownMemberType]
else:
    print("Found sprinkle_random attribute")

    # Create vertex color layer
    print("Converting sprinkle_random to vertex colors...")
    if not mesh.vertex_colors:  # pyright: ignore[reportUnknownMemberType]
        color_layer = mesh.vertex_colors.new(name="sprinkle_color")  # pyright: ignore[reportUnknownMemberType]
    else:
        color_layer = mesh.vertex_colors[0]  # pyright: ignore[reportUnknownMemberType]

    # Get the attribute data
    attr = mesh.attributes["sprinkle_random"]  # pyright: ignore[reportUnknownMemberType]

    # Convert attribute to vertex colors using the actual ColorRamp from the material
    def random_to_color(random_val: float) -> tuple[float, float, float]:
        """Convert random value [0,1] to a color using the extracted ColorRamp"""
        # Find the two color stops that bracket this value
        if random_val <= color_stops[0][0]:
            return color_stops[0][1]
        if random_val >= color_stops[-1][0]:
            return color_stops[-1][1]

        # Find the two stops to interpolate between
        for i in range(len(color_stops) - 1):
            pos1, color1 = color_stops[i]
            pos2, color2 = color_stops[i + 1]

            if pos1 <= random_val <= pos2:
                # Linear interpolation
                t = (random_val - pos1) / (pos2 - pos1)
                r = color1[0] + t * (color2[0] - color1[0])
                g = color1[1] + t * (color2[1] - color1[1])
                b = color1[2] + t * (color2[2] - color1[2])
                return (r, g, b)

        # Fallback (shouldn't reach here)
        return color_stops[0][1]

    # Vertex colors are per-loop, so we need to map from vertices
    for poly in mesh.polygons:  # pyright: ignore[reportUnknownMemberType]
        for loop_idx in poly.loop_indices:  # pyright: ignore[reportUnknownMemberType]
            loop = mesh.loops[loop_idx]  # pyright: ignore[reportUnknownMemberType]
            vert_idx = loop.vertex_index  # pyright: ignore[reportUnknownMemberType]

            # Get random value for this vertex
            random_val = attr.data[vert_idx].value  # pyright: ignore[reportUnknownMemberType]

            # Convert to color using color ramp
            r, g, b = random_to_color(random_val)
            color_layer.data[loop_idx].color = (r, g, b, 1.0)  # pyright: ignore[reportUnknownMemberType]

    print(f"Created vertex color layer: {color_layer.name}")  # pyright: ignore[reportUnknownMemberType]

    # Update Sprinkles material to use vertex colors
    print("Updating Sprinkles material...")
    for mat_slot in icing.material_slots:  # pyright: ignore[reportUnknownMemberType]
        mat = mat_slot.material  # pyright: ignore[reportUnknownMemberType]
        if mat and "Sprinkles" in mat.name:  # pyright: ignore[reportUnknownMemberType]
            print(f"  Found material: {mat.name}")  # pyright: ignore[reportUnknownMemberType]
            nodes = mat.node_tree.nodes  # pyright: ignore[reportUnknownMemberType]
            links = mat.node_tree.links  # pyright: ignore[reportUnknownMemberType]

            # Find Principled BSDF
            bsdf = None
            for node in nodes:  # pyright: ignore[reportUnknownMemberType]
                if node.type == 'BSDF_PRINCIPLED':  # pyright: ignore[reportUnknownMemberType]
                    bsdf = node
                    break

            if bsdf and 'Base Color' in bsdf.inputs:  # pyright: ignore[reportUnknownMemberType]
                base_color_input = bsdf.inputs['Base Color']  # pyright: ignore[reportUnknownMemberType]

                # Disconnect any existing connection to Base Color
                if base_color_input.is_linked:  # pyright: ignore[reportUnknownMemberType]
                    for link in base_color_input.links:  # pyright: ignore[reportUnknownMemberType]
                        links.remove(link)  # pyright: ignore[reportUnknownMemberType]

                # Create Vertex Color node
                vc_node = nodes.new(type='ShaderNodeVertexColor')  # pyright: ignore[reportUnknownMemberType]
                vc_node.layer_name = "sprinkle_color"  # pyright: ignore[reportUnknownMemberType]
                vc_node.location = (bsdf.location[0] - 300, bsdf.location[1])  # pyright: ignore[reportUnknownMemberType]

                # Connect Vertex Color to Base Color
                links.new(vc_node.outputs['Color'], base_color_input)  # pyright: ignore[reportUnknownMemberType]

                print(f"    Connected Vertex Color node to Base Color input")

                # Update Metallic and Roughness ColorRamps to use vertex attribute instead of Object Info
                # Find the ColorRamp nodes for Metallic and Roughness
                metallic_ramp = nodes.get("Color Ramp.001")  # pyright: ignore[reportUnknownMemberType]
                roughness_ramp = nodes.get("Color Ramp.002")  # pyright: ignore[reportUnknownMemberType]

                # Create Attribute node for sprinkle_random
                attr_node = nodes.new(type='ShaderNodeAttribute')  # pyright: ignore[reportUnknownMemberType]
                attr_node.attribute_name = "sprinkle_random"  # pyright: ignore[reportUnknownMemberType]
                attr_node.location = (bsdf.location[0] - 600, bsdf.location[1] - 200)  # pyright: ignore[reportUnknownMemberType]

                if metallic_ramp:
                    # Disconnect Object Info and connect Attribute node
                    if metallic_ramp.inputs['Fac'].is_linked:  # pyright: ignore[reportUnknownMemberType]
                        for link in metallic_ramp.inputs['Fac'].links:  # pyright: ignore[reportUnknownMemberType]
                            links.remove(link)  # pyright: ignore[reportUnknownMemberType]
                    links.new(attr_node.outputs['Fac'], metallic_ramp.inputs['Fac'])  # pyright: ignore[reportUnknownMemberType]
                    print(f"    Connected Attribute node to Metallic ColorRamp")

                if roughness_ramp:
                    # Disconnect Object Info and connect Attribute node
                    if roughness_ramp.inputs['Fac'].is_linked:  # pyright: ignore[reportUnknownMemberType]
                        for link in roughness_ramp.inputs['Fac'].links:  # pyright: ignore[reportUnknownMemberType]
                            links.remove(link)  # pyright: ignore[reportUnknownMemberType]
                    links.new(attr_node.outputs['Fac'], roughness_ramp.inputs['Fac'])  # pyright: ignore[reportUnknownMemberType]
                    print(f"    Connected Attribute node to Roughness ColorRamp")

# Re-unwrap UVs for all icing faces and pack them properly
print("\n=== UV Unwrapping and Packing ===")
bpy.context.view_layer.objects.active = icing  # pyright: ignore[reportUnknownMemberType]
bpy.ops.object.mode_set(mode='EDIT')  # pyright: ignore[reportUnknownMemberType]

# Select all faces
bpy.ops.mesh.select_all(action='SELECT')  # pyright: ignore[reportUnknownMemberType]

# Smart UV unwrap all faces
bpy.ops.uv.smart_project(angle_limit=66.0, island_margin=0.02)  # pyright: ignore[reportUnknownMemberType]
print("UV unwrapping complete")

# Pack UV islands to avoid overlap
bpy.ops.uv.pack_islands(margin=0.01)  # pyright: ignore[reportUnknownMemberType]
print("UV packing complete")

# Switch back to object mode
bpy.ops.object.mode_set(mode='OBJECT')  # pyright: ignore[reportUnknownMemberType]

# Delete Sphere template if it exists
sphere = bpy.data.objects.get("Sphere")  # pyright: ignore[reportUnknownMemberType]
if sphere:
    print("Deleting Sphere template object...")
    bpy.data.objects.remove(sphere, do_unlink=True)  # pyright: ignore[reportUnknownMemberType]

# Verify before saving
print("\n=== Verification ===")
print(f"Icing vertices: {len(icing.data.vertices)}")  # pyright: ignore[reportUnknownMemberType]
print(f"Vertex color layers: {len(icing.data.vertex_colors)}")  # pyright: ignore[reportUnknownMemberType]
if donut:
    print(f"Donut object exists: Yes")
    if donut.material_slots:  # pyright: ignore[reportUnknownMemberType]
        print(f"Donut materials: {[slot.material.name for slot in donut.material_slots if slot.material]}")  # pyright: ignore[reportUnknownMemberType]

# Save as nateroid_bake.blend
print(f"\nSaving to: {output_blend}")
bpy.ops.wm.save_as_mainfile(filepath=str(output_blend))  # pyright: ignore[reportUnknownMemberType]
print("Done!")
