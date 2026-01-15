# Bevy 0.18.0 Migration Plan

**Generated:** 2026-01-15
**Codebase:** /Users/natemccoy/rust/nateroids
**Total Applicable Guides:** 7

---

## Summary

- **REQUIRED changes:** 3 guides (6 total occurrences)
- **HIGH priority:** 0 guides (0 total occurrences)
- **MEDIUM priority:** 1 guide (8 occurrences - no code changes needed)
- **LOW priority:** 3 guides (0 code changes needed - informational only)

**Count Anomalies:** 1 guide with >20% variance between Pass 1 and Pass 2
- Automatic_Aabb_updates_for_sprites_and_meshes.md: Pass 1=38, Pass 2=0 (-100%)

**Estimated effort:**
- REQUIRED: Small (6 specific locations to update)
- HIGH: None
- MEDIUM: None (behavioral change, review only)
- LOW: None (informational)

---

## Anomaly Analysis

During the two-pass analysis, 1 guide showed significant variance (>20%) between initial pattern matching and deep contextual analysis:

### Automatic_Aabb_updates_for_sprites_and_meshes.md
- **Pass 1 Count:** 38 occurrences
- **Pass 2 Count:** 0 occurrences
- **Variance:** -100%
- **Explanation:** Pass 1 found 38 matches for the pattern "Aabb", but deep analysis revealed these are all references to a **custom `Aabb` struct** defined in `src/actor/aabb.rs` for game-specific bounding box calculations. This is NOT Bevy's `bevy_render::primitives::Aabb` component that the migration guide addresses. The custom type has nothing to do with the automatic AABB update behavior change in Bevy 0.18.

---

## Dependency Compatibility Review

**Status:** 4 dependencies checked
- Compatible: 0
- Updates available: 4
- Needs verification: 0
- Blockers: 0

---

### Updates Required

- **`bevy-inspector-egui = "0.35.0"`** -> `"0.36.0"`
  - Current version 0.35.0 incompatible. Update to 0.36.0 for Bevy 0.18.0 support
  - Action: Update in Cargo.toml to version 0.36.0
- **`bevy_brp_extras = "0.17.2"`** -> `"0.18.0"`
  - Current version 0.17.2 incompatible. Update to 0.18.0 for Bevy 0.18.0 support
  - Action: Update in Cargo.toml to version 0.18.0
- **`bevy_panorbit_camera = "0.33.0"`** -> `"0.34.0"`
  - Current version 0.33.0 incompatible. Update to 0.34.0 for Bevy 0.18.0 support
  - Action: Update in Cargo.toml to version 0.34.0
- **`bevy_window_manager = "0.17.0"`** -> `"0.18.0"`
  - Current version 0.17.0 incompatible. Update to 0.18.0 for Bevy 0.18.0 support
  - Action: Update in Cargo.toml to version 0.18.0

### Recommended Actions

1. **Update dependencies** - Bump versions in Cargo.toml
2. **Monitor for updates** - Check dependency issue trackers if blockers exist

---

## REQUIRED Changes

## `Gizmos::cuboid` has been renamed to `Gizmos::cube`

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/Gizmoscuboid_has_been_renamed_to_Gizmoscube.md`
**Requirement Level:** REQUIRED
**Occurrences:** [1] location across [1] file
**Pass 1 Count:** 24 | **Pass 2 Count:** 1 | **Status:** ANOMALY: Pattern overlap - only 1 actual `Gizmos::cuboid` call exists; other matches are `Collider::cuboid` (Avian physics) and `Cuboid` mesh primitive

### Migration Guide Summary

The `Gizmos::cuboid` method has been renamed to `Gizmos::cube` in Bevy 0.18.0. This is a straightforward API rename that will cause a compilation error until updated. The method signature and behavior remain the same.

### Required Changes

**1. Update `gizmos.cuboid()` to `gizmos.cube()` in `src/actor/aabb.rs`**
```diff
fn draw_aabb_system(
    mut gizmos: Gizmos<AabbGizmo>,
    aabbs: Query<(&Transform, &Aabb)>,
    config: Res<AabbConfig>,
) {
    for (transform, aabb) in aabbs.iter() {
        let center = transform.transform_point(aabb.center());

-        gizmos.cuboid(
+        gizmos.cube(
            Transform::from_trs(center, transform.rotation, aabb.size() * transform.scale),
            config.color,
        );
    }
}
```

### Search Pattern

To find all occurrences:
```bash
rg "gizmos\.cuboid" --type rust
```

---

## glTF Coordinate Conversion

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/glTF_Coordinate_Conversion.md`
**Requirement Level:** REQUIRED
**Occurrences:** [3] locations across [1] files
**Pass 1 Count:** 3 | **Pass 2 Count:** 3 | **Status:** MATCH

### Migration Guide Summary

The `use_model_forward_direction` field on `GltfPlugin` and `GltfLoaderSettings` has been renamed to `convert_coordinates` and is now a struct (`GltfConvertCoordinates`) with two separate options: `rotate_scene_entity` and `rotate_meshes`. This change provides more granular control over glTF coordinate conversion behavior and fixes bugs with cameras and lights from Bevy 0.17.

### Required Changes

**1. Update `GltfPlugin` configuration in `src/main.rs`**

The import and configuration must be updated to use the new `GltfConvertCoordinates` struct.

```diff
- use bevy::gltf::GltfPlugin;
+ use bevy::gltf::GltfConvertCoordinates;
+ use bevy::gltf::GltfPlugin;
```

```diff
            .set(GltfPlugin {
-                use_model_forward_direction: true,
+                convert_coordinates: GltfConvertCoordinates {
+                    rotate_scene_entity: true,
+                    rotate_meshes: true,
+                },
                ..default()
            })
```

Note: Both `rotate_scene_entity` and `rotate_meshes` are enabled to match the closest behavior to the 0.17 `use_model_forward_direction: true` setting, as recommended by the migration guide.

### Search Pattern

To find all occurrences:
```bash
rg "use_model_forward_direction|GltfPlugin|GltfLoaderSettings" --type rust
```

---

## `AmbientLight` split into a component and a resource

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/AmbientLight_split_into_a_component_and_a_resource.md`
**Requirement Level:** REQUIRED
**Occurrences:** [2] locations across [1] files
**Pass 1 Count:** 2 | **Pass 2 Count:** 2 | **Status:** MATCH

### Migration Guide Summary

In Bevy 0.18, `AmbientLight` has been split into two separate structs: `AmbientLight` (a component for per-camera override) and `GlobalAmbientLight` (a resource for world-wide ambient lighting). Code that previously used `AmbientLight` as a resource should be renamed to `GlobalAmbientLight`.

### Required Changes

**1. Update resource initialization in `src/camera/lights.rs`**
```diff
-        app.init_resource::<AmbientLight>()
+        app.init_resource::<GlobalAmbientLight>()
```

**2. Update system parameter type in `src/camera/lights.rs`**
```diff
 fn manage_lighting(
     mut commands: Commands,
-    mut ambient_light: ResMut<AmbientLight>,
+    mut ambient_light: ResMut<GlobalAmbientLight>,
     light_config: Res<LightConfig>,
     camera_orientation: Res<CameraOrientation>,
     mut query: Query<(Entity, &mut DirectionalLight, &LightDirection)>,
 ) {
```

### Search Pattern

To find all occurrences:
```bash
rg "AmbientLight" --type rust
```

---

## HIGH Priority Changes

*No HIGH priority changes required.*

---

## MEDIUM Priority Changes

## Same State Transitions

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/Same_State_Transitions.md`
**Requirement Level:** MEDIUM
**Occurrences:** 8 locations across 4 files
**Pass 1 Count:** 16 | **Pass 2 Count:** 16 | **Status:** MATCH

### Migration Guide Summary

In Bevy 0.18, calling `next_state.set()` now triggers state transitions (`OnEnter`, `OnExit`) even when setting the state to the same value it already has. If your code relies on the previous behavior where setting the same state was a no-op, you should use `set_if_neq()` instead to preserve that behavior.

### Required Changes

This change is behavioral rather than breaking at compile time. The existing `next_state.set()` calls will still compile, but they will now trigger `OnEnter`/`OnExit` schedules even when the state doesn't change. Review each usage to determine if you need the old behavior (use `set_if_neq`) or want the new behavior (keep `set`).

**Analysis of all 8 usages:**

1. `spaceship_destroyed` in `src/actor/spaceship.rs` - Transitions from InGame to GameOver. **No change needed.**
2. `run_splash` in `src/splash.rs` - Transitions from Splash to InGame. **No change needed.**
3. `toggle_pause` in `src/state.rs` - Changes paused field within InGame. **No change needed.**
4. `restart_game` in `src/state.rs` - Transitions to GameOver. **No change needed.**
5. `restart_with_splash` in `src/state.rs` - Transitions to Splash. **No change needed.**
6. `transition_to_in_game` in `src/state.rs` - Transitions from GameOver to InGame. **No change needed.**
7. `transition_to_splash_on_startup` in `src/state.rs` - Transitions to Splash on startup. **No change needed.**
8. `check_asset_loading` in `src/asset_loader.rs` - Transitions from Loading to Loaded. **No change needed.**

**No code changes required.** All usages are intentional state transitions between different states.

### Search Pattern

To find all occurrences:
```bash
rg "next_state\.set" --type rust
```

---

## LOW Priority Changes

## System Combinators

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/System_Combinators.md`
**Requirement Level:** LOW
**Occurrences:** [1] locations across [1] files
**Pass 1 Count:** 45 | **Pass 2 Count:** 45 | **Status:** MATCH

### Migration Guide Summary

Bevy 0.18 changes how `CombinatorSystem`s handle errors when combining multiple `SystemCondition`s with logical operators (`and`, `or`, `xor`, `nand`, `nor`, `xnor`). Previously, if one of the combined systems failed validation, the error would propagate and the entire combinator would return an error. Now, failed systems are treated as returning `false`, and the combinator logic continues. This is a behavioral change that does not require code modifications but may affect runtime behavior in edge cases.

### Required Changes

**No code changes required.** The existing `.or()` usage in `src/playfield/boundary.rs` will continue to work correctly. Since `in_state()` is a standard Bevy condition that won't fail validation, this behavioral change has no impact.

### Search Pattern

To find all occurrences of system combinators:
```bash
rg "\.(and|or|xor|nand|nor|xnor)\(" --type rust
```

---

## Automatic `Aabb` updates for sprites and meshes

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/Automatic_Aabb_updates_for_sprites_and_meshes.md`
**Requirement Level:** LOW
**Occurrences:** 0 relevant locations across 0 files
**Pass 1 Count:** 38 | **Pass 2 Count:** 0 | **Status:** ANOMALY: -100% (false positive - custom type)

### Migration Guide Summary

In Bevy 0.17, the `Aabb` component was not automatically updated when meshes or sprites were modified, requiring manual workarounds like calling `entity.remove::<Aabb>()` after mesh modifications. Bevy 0.18 fixes this by automatically updating `Aabb` when the underlying mesh or sprite changes.

### Required Changes

**No changes required.** The 38 occurrences of "Aabb" found in this codebase are a custom `Aabb` struct defined in `src/actor/aabb.rs` - this is NOT Bevy's `bevy_render::primitives::Aabb` component.

### Search Pattern

To verify no Bevy Aabb usage exists:
```bash
rg "bevy_render::primitives::Aabb|bevy::render::primitives::Aabb" --type rust
```

---

## LineHeight is now a separate component

**Guide File:** `/Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides/LineHeight_is_now_a_separate_component.md`
**Requirement Level:** LOW
**Occurrences:** 0 locations across 0 files
**Pass 1 Count:** 4 | **Pass 2 Count:** 0 | **Status:** MATCH (no migration required)

### Migration Guide Summary

The `line_height` field has been removed from `TextFont`. `LineHeight` is now a separate component. This migration only affects code that explicitly used the `line_height` field on `TextFont`.

### Required Changes

**No changes required.** All 4 `TextFont` usages in the codebase use `..default()` and do not explicitly set `line_height`.

### Search Pattern

To find code that would need migration:
```bash
rg "line_height" --type rust
```

---

## Guides Not Applicable to This Codebase

The following 57 guides from Bevy 0.18.0 do not apply to this codebase:

- release-content/migration-guides/API_for_working_with_Relationships_and_RelationshipTargets_in_type-erased_contex.md
- release-content/migration-guides/AnimationEventTriggeranimation_player_has_been_renamed_to_AnimationEventTriggert.md
- release-content/migration-guides/AnimationTarget_replaced_by_separate_components.md
- release-content/migration-guides/ArchetypeQueryData_trait.md
- release-content/migration-guides/AssetPlugin_now_has_a_use_asset_processor_override_field.md
- release-content/migration-guides/AssetSources_now_give_an_async_channelSender_instead_of_a_crossbeam_channelSende.md
- release-content/migration-guides/BevyManifestshared_is_now_a_scope-like_API.md
- release-content/migration-guides/BindGroupLayout_labels_are_no_longer_optional.md
- release-content/migration-guides/BorderRadius_has_been_added_to_Node_and_is_no_longer_a_component.md
- release-content/migration-guides/BorderRect_now_has_Vec2_fields.md
- release-content/migration-guides/Cargo_Feature_Collections.md
- release-content/migration-guides/Change_Bundlecomponent_ids_and_Bundleget_component_ids_to_return_an_iterator.md
- release-content/migration-guides/Changes_to_AssetServer_and_AssetProcessor_creation.md
- release-content/migration-guides/Changes_to_the_Process_trait_in_bevy_asset.md
- release-content/migration-guides/Custom_asset_sources_now_require_a_reader.md
- release-content/migration-guides/Derive_on_Resource_will_fail_when_using_non-static_lifetimes.md
- release-content/migration-guides/DragEnter_now_fires_on_drag_starts.md
- release-content/migration-guides/Entities_APIs.md
- release-content/migration-guides/ExtractedUiNodes_stack_index_has_been_renamed_to_z_order_and_is_now_an_f32.md
- release-content/migration-guides/Feature_cleanup.md
- release-content/migration-guides/FunctionSystem_Generics.md
- release-content/migration-guides/Generalized_Atmospheric_Scattering_Media.md
- release-content/migration-guides/ImageRenderTargets_scale_factor_field_is_now_an_f32.md
- release-content/migration-guides/Image_Loader_Array_Layout.md
- release-content/migration-guides/Imagereinterpret_size_and_Imagereinterpret_stacked_2d_as_array_now_return_a_Resu.md
- release-content/migration-guides/Immutable_Entity_Events.md
- release-content/migration-guides/Implementations_of_Reader_now_must_implement_Readerseekable_and_AsyncSeekForward.md
- release-content/migration-guides/Internal_has_been_removed.md
- release-content/migration-guides/LoadContextpath_now_returns_AssetPath.md
- release-content/migration-guides/Per-RenderPhase_Draw_Functions.md
- release-content/migration-guides/Put_input_sources_for_bevy_input_under_features.md
- release-content/migration-guides/Remove_bevyptrdangling_with_align.md
- release-content/migration-guides/Remove_ron_re-export_from_bevy_scene_and_bevy_asset.md
- release-content/migration-guides/Removed_FontAtlasSets.md
- release-content/migration-guides/Removed_SimpleExecutor.md
- release-content/migration-guides/Removed_dummy_white_gpu_image.md
- release-content/migration-guides/Rename_ThinSlicePtrget_to_ThinSlicePtrget_unchecked.md
- release-content/migration-guides/Renamed_bevy_platformHashMapget_many__to_bevy_platformHashMapget_disjoint_.md
- release-content/migration-guides/Renamed_bevy_reflect_feature_documentation_to_reflect_documentation.md
- release-content/migration-guides/Renamed_clear_children_and_clear_related_methods_to_detach_.md
- release-content/migration-guides/RenderPipelineDescriptor_and_ComputePipelineDescriptor_now_hold_a_BindGroupLayou.md
- release-content/migration-guides/RenderTarget_is_now_a_component.md
- release-content/migration-guides/Replaced_Column_with_ThinColumn.md
- release-content/migration-guides/Schedule_cleanup.md
- release-content/migration-guides/TextLayoutInfos_section_rects_field_has_been_replaced_with_run_geometry.md
- release-content/migration-guides/The_non-text_areas_of_UI_Text_nodes_are_no_longer_pickable.md
- release-content/migration-guides/Tick-related_refactors.md
- release-content/migration-guides/Tilemap_Chunk_Layout.md
- release-content/migration-guides/TrackedRenderPassset_index_buffer_no_longer_takes_buffer_offset.md
- release-content/migration-guides/Traits_AssetLoader_AssetTransformer_AssetSaver_and_Process_all_now_require_TypeP.md
- release-content/migration-guides/Use_Meshtry__mesh_functions_for_AssetsMesh_entries_when_there_can_be_RenderAsset.md
- release-content/migration-guides/Virtual_Geometry.md
- release-content/migration-guides/Winit_user_events_removed.md
- release-content/migration-guides/bevy_gizmos_rendering_split.md
- release-content/migration-guides/enable_prepass_and_enable_shadows_are_now_Material_methods.md
- release-content/migration-guides/get_components_get_components_mut_unchecked_now_return_a_Result.md
- release-content/migration-guides/reflect_now_supports_only_parentheses.md

---

## Next Steps

1. Start with REQUIRED changes (must fix to compile with Bevy 0.18.0)
2. Update dependencies in Cargo.toml (4 packages need version bumps)
3. Address MEDIUM priority changes (review state transition behavior)
4. Consider LOW priority improvements (informational only)
5. Test thoroughly after each category of changes
6. Run `cargo check` and `cargo nextest run` frequently

---

## Reference

- **Migration guides directory:** /Users/natemccoy/rust/bevy-0.18.0/release-content/migration-guides
- **Bevy 0.18.0 release notes:** https://github.com/bevyengine/bevy/releases/tag/v0.18.0
