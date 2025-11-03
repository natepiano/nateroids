# Bevy 0.17.2 Migration Plan

**Generated:** 2025-11-02
**Codebase:** /Users/natemccoy/rust/nateroids
**Total Applicable Guides:** 11

---

## Summary

- **REQUIRED changes:** 3 guides (96 total occurrences)
- **HIGH priority:** 2 guides (10 total occurrences)
- **MEDIUM priority:** 0 guides (0 total occurrences)
- **LOW priority:** 6 guides (6 total occurrences)

**Count Anomalies:** 2 guides with >20% variance between Pass 1 and Pass 2
- anchor_is_removed_from_sprite.md: Pass 1=20, Pass 2=1 (±95%)
- state_scoped_entities_by_default.md: Pass 1=6, Pass 2=10 (±67%)

**Estimated effort:** Large (must fix to compile)
- REQUIRED: Large (96 occurrences across import reorganization, HDR changes, and event system changes)
- HIGH: Medium (10 occurrences for reflection and deprecated methods)
- MEDIUM: None
- LOW: Small (informational only - no deprecated methods in use)

---

## 🔍 Anomaly Analysis

During the two-pass analysis, 2 guide(s) showed significant variance (>20%) between initial pattern matching and deep contextual analysis:

### anchor_is_removed_from_sprite.md
- **Pass 1 Count:** 20 occurrences
- **Pass 2 Count:** 1 occurrence
- **Variance:** ±95%
- **Explanation:** This is a 3D game that does not use 2D sprites. Pass 1 incorrectly counted `Sprite` (19) and `Camera3d` (1) patterns, but the guide specifically addresses the removal of `anchor` field from the `Sprite` component for 2D games. The single `Camera3d` occurrence is unrelated to sprite anchoring. This codebase uses `Mesh3d` for 3D rendering and has no sprite usage, making this guide informational only.

### state_scoped_entities_by_default.md
- **Pass 1 Count:** 6 occurrences
- **Pass 2 Count:** 10 occurrences
- **Variance:** ±67%
- **Explanation:** The variance is due to Pass 1 counting only direct `States` trait mentions (4) and state derive macros (2), while Pass 2 also found occurrences of `ComputedStates` trait implementations and `SourceStates` type aliases that contain the word "States". The guide addresses removal of the deprecated `enable_state_scoped_entities()` method and `#[states(scoped_entities)]` attribute, neither of which are present in the codebase. The additional `States` pattern matches are unrelated to the migration.

---

## ⚠️ Dependency Compatibility Review

**Status:** 3 dependencies checked
- ✅ Compatible: 0
- 🔄 Updates available: 2
- ⚠️  Needs verification: 0
- 🚫 Blockers: 1

---

### 🚫 Blockers (Must Resolve Before Migration)

**RESOLUTION AVAILABLE**: Physics engine blocker has a proven solution - migrate to Avian 3D.

#### Current Status Assessment (as of November 2025)
- **bevy_rapier3d**: Latest version 0.31.0 supports Bevy 0.16 only
- **No bevy_rapier3d release exists for Bevy 0.17** (Bevy 0.17.2 released October 4, 2025)
- **Timeline**: Historically, bevy_rapier3d updates 2-8 months after Bevy releases
- **Blocker Status**: RESOLVED via alternative physics engine

#### Solution: Migrate to Avian Physics Engine

**Why Avian:**
- **Proven alternative**: Successor to bevy_xpbd, actively maintained
- **Bevy 0.17 support**: Confirmed compatible (avian3d 0.4.x + Bevy 0.17.x)
- **Latest version**: avian3d 0.4.1 (released October 2025)
- **Performance**: 4-6x improvement in collision-heavy scenes vs bevy_xpbd
- **ECS-native**: Better integration with Bevy's architecture than Rapier
- **Feature parity**: Includes all features used in nateroids (SIMD, debug rendering, parallel)

**Version Compatibility:**
| Bevy Version | Avian Version |
|-------------|---------------|
| 0.17.x      | 0.4.x         |
| 0.16.x      | 0.3.x         |

**Migration Steps:**

1. **Update Cargo.toml dependencies:**
   ```toml
   [dependencies]
   bevy = { version = "0.17.2", features = [...] }
   # Replace bevy_rapier3d with avian3d
   avian3d = { version = "0.4.1", features = [
     "simd",           # equivalent to bevy_rapier3d's "simd-stable"
     "debug-plugin",   # equivalent to "debug-render-3d"
     "parallel",       # same as bevy_rapier3d
   ] }
   ```

2. **Update plugin registration** (in main.rs or app setup):
   ```rust
   // OLD:
   use bevy_rapier3d::prelude::*;
   app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());

   // NEW:
   use avian3d::prelude::*;
   app.add_plugins(PhysicsPlugins::default());
   ```

3. **Update component imports and usage:**
   - `RigidBody` → `RigidBody` (same name, different module)
   - `Collider` → `Collider` (same name, different module)
   - `ExternalForce` → `ExternalForce` (same name)
   - `Velocity` → `LinearVelocity` + `AngularVelocity`
   - Refer to Avian migration guide for complete mapping

4. **Test physics behaviors:**
   - Verify collision detection works as expected
   - Check that forces/impulses apply correctly
   - Validate debug rendering displays properly
   - Test performance in asteroid-heavy scenarios

**Verification Steps:**

Phase 1 - Dependency Resolution:
```bash
cargo search avian3d --limit 1  # Verify 0.4.1 available
cargo info avian3d              # Check features exist
```

Phase 2 - Compilation:
```bash
cargo check                     # Resolve dependencies
cargo tree | grep -E "(bevy|avian)"  # Verify versions
```

Phase 3 - Functionality Testing:
```bash
cargo run --features debug-plugin    # Test debug rendering
cargo build --release               # Test SIMD optimizations
```

Phase 4 - Behavioral Validation:
- Asteroid spawning with physics bodies
- Collisions: ship-asteroid, asteroid-asteroid, bullet-asteroid
- Forces: thrust, rotation, momentum conservation
- Performance: FPS with many asteroids (should improve)
- Debug rendering: collider shapes display correctly

**Alternative: Wait for bevy_rapier3d**
- **NOT RECOMMENDED**: Unknown timeline (could be months)
- Monitor: https://github.com/dimforge/bevy_rapier/issues
- Risk: Delays entire migration indefinitely

**Decision**: Proceed with Avian migration as part of Bevy 0.17 upgrade.

### 🔄 Updates Required

- **`bevy-inspector-egui = "0.33.1"`** → `"0.35.0"`
  - Current version 0.33.1 incompatible. Update to 0.35.0 for Bevy 0.17.2 support
  - Action: Update in Cargo.toml to version 0.35.0
- **`bevy_panorbit_camera = "0.28.0"`** → `"0.32.0"`
  - Current version 0.28.0 incompatible. Update to 0.32.0 for Bevy 0.17.2 support
  - Action: Update in Cargo.toml to version 0.32.0

### Recommended Actions

1. **🚫 Address blockers first** - Migration cannot proceed without resolving these
2. **🔄 Update dependencies** - Bump versions in Cargo.toml
3. **Monitor for updates** - Check dependency issue trackers if blockers exist

---

## REQUIRED Changes

## `bevy_render` reorganization

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/bevy_render_reorganization.md`
**Requirement Level:** REQUIRED
**Occurrences:** 89 locations across 8 files
**Pass 1 Count:** 89 | **Pass 2 Count:** 89 | **Status:** MATCH: ±0.0%

### Migration Guide Summary

Types from `bevy_render` have been reorganized into specialized crates: `bevy_camera` for camera and visibility types, `bevy_post_process` for post-processing effects like Bloom, and visibility-related types like `RenderLayers` now have new import paths. The `bevy::core_pipeline` module has been split, with anti-aliasing and post-processing effects moved to dedicated crates.

### Affected Files Summary

**Files Requiring Import Changes (6 files):**
1. `src/camera/cameras.rs` - Update Bloom, Tonemapping, RenderLayers imports
2. `src/camera/mod.rs` - Update Layer import
3. `src/camera/stars.rs` - Update RenderLayers import
4. `src/actor/actor_spawner.rs` - Update RenderLayers import
5. `src/splash.rs` - Update RenderLayers import
6. `src/actor/aabb.rs` - Update VertexAttributeValues import

**Files Requiring No Changes (types now in prelude):**
- `src/camera/star_twinkling.rs` - MeshMaterial3d now in prelude
- `src/playfield/planes.rs` - Mesh3d, MeshMaterial3d now in prelude
- `src/main.rs` - ImagePlugin now in prelude

### Required Changes

**1. Update Bloom and Tonemapping imports in `src/camera/cameras.rs`**

> **IMPORTANT - CONSOLIDATED FINAL STATE**: This file is also affected by the "Split Hdr from Camera" migration (see section below). After applying BOTH migrations, the final import block should be:

```rust
use bevy::{
    camera::visibility::RenderLayers,
    post_process::bloom::Bloom,
    post_process::tonemapping::Tonemapping,
    prelude::*,
    render::view::Hdr,
};
```

Individual changes for this section only:
```diff
use bevy::{
-    core_pipeline::{
-        bloom::Bloom,
-        tonemapping::Tonemapping,
-    },
+    post_process::bloom::Bloom,
+    post_process::tonemapping::Tonemapping,
    prelude::*,
-    render::view::RenderLayers,
+    camera::visibility::RenderLayers,
};
```

**2. Update RenderLayers import in `src/camera/mod.rs`**
```diff
use bevy::{
    prelude::*,
-    render::view::Layer,
+    camera::visibility::Layer,
};
```

**3. Update RenderLayers import in `src/camera/stars.rs`**
```diff
use bevy::{
    prelude::*,
-    render::view::RenderLayers,
+    camera::visibility::RenderLayers,
};
```

**4. Update RenderLayers import in `src/actor/actor_spawner.rs`**
```diff
use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
-    render::view::RenderLayers,
+    camera::visibility::RenderLayers,
};
```

**5. Update RenderLayers import in `src/splash.rs`**
```diff
use bevy::{
    prelude::*,
-    render::view::RenderLayers,
+    camera::visibility::RenderLayers,
};
```

**6. Update VertexAttributeValues import in `src/actor/aabb.rs`**
```diff
use bevy::{
    color::palettes::tailwind,
    prelude::*,
-    render::mesh::VertexAttributeValues,
+    mesh::VertexAttributeValues,
};
```

**7. Update Mesh usage in `src/camera/cameras.rs`**
No import changes needed - `Camera`, `Projection`, `Camera3d`, `ClearColor`, and `ClearColorConfig` are now re-exported through `bevy::prelude::*` from `bevy_camera` crate. The existing code using these types will continue to work without modification.

**8. Update Mesh3d usage in `src/actor/aabb.rs`**
No import changes needed - `Mesh3d` and `Mesh` are now re-exported through `bevy::prelude::*` from `bevy_mesh` crate. The existing code at lines 65-66 will continue to work:
```rust
if let Some(mesh_handle) = entity.get::<Mesh3d>()
    && let Some(mesh) = meshes.get(mesh_handle)
```

**9. Update Mesh usage in `src/camera/stars.rs`**
No import changes needed - `Mesh3d` and `MeshMaterial3d` are now re-exported through `bevy::prelude::*` from `bevy_mesh` crate. The existing code will continue to work without modification.

**10. Update Mesh usage in `src/playfield/planes.rs`**
No import changes needed - `Mesh3d` and `MeshMaterial3d` are now re-exported through `bevy::prelude::*` from `bevy_mesh` crate. The existing code will continue to work without modification.

**11. Update Mesh usage in `src/camera/star_twinkling.rs`**
No import changes needed - `MeshMaterial3d` is now re-exported through `bevy::prelude::*` from `bevy_mesh` crate. The existing code will continue to work without modification.

**12. Update ImagePlugin usage in `src/main.rs`**
No import changes needed - `ImagePlugin` is now re-exported through `bevy::prelude::*` from `bevy_image` crate. The existing code at line 50 will continue to work:
```rust
.set(ImagePlugin::default_nearest())
```

### Verification Steps

**Before Migration - Find all affected locations:**
```bash
rg "core_pipeline::(bloom|tonemapping)" --type rust  # Should find src/camera/cameras.rs
rg "render::view::(RenderLayers|Layer)" --type rust  # Should find 5 files (cameras.rs, mod.rs, stars.rs, actor_spawner.rs, splash.rs)
rg "render::mesh::VertexAttributeValues" --type rust # Should find src/actor/aabb.rs
```

**After Migration - Verify changes complete:**
```bash
# These should return NO results (all old imports removed):
rg "core_pipeline::bloom" --type rust
rg "core_pipeline::tonemapping" --type rust
rg "render::view::RenderLayers" --type rust
rg "render::view::Layer" --type rust
rg "render::mesh::VertexAttributeValues" --type rust

# These should find the NEW imports in the 6 affected files:
rg "post_process::bloom::Bloom" --type rust           # cameras.rs
rg "post_process::tonemapping::Tonemapping" --type rust  # cameras.rs
rg "camera::visibility::RenderLayers" --type rust     # cameras.rs, stars.rs, actor_spawner.rs, splash.rs
rg "camera::visibility::Layer" --type rust            # mod.rs
rg "mesh::VertexAttributeValues" --type rust          # aabb.rs
```

**Checklist - Confirm all 6 files updated:**
- [ ] `src/camera/cameras.rs` - Bloom, Tonemapping, RenderLayers
- [ ] `src/camera/mod.rs` - Layer
- [ ] `src/camera/stars.rs` - RenderLayers
- [ ] `src/actor/actor_spawner.rs` - RenderLayers
- [ ] `src/splash.rs` - RenderLayers
- [ ] `src/actor/aabb.rs` - VertexAttributeValues

---

## Split `Hdr` from `Camera`

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/hdr_component.md`
**Requirement Level:** REQUIRED
**Occurrences:** 6 locations across 1 files
**Pass 1 Count:** 5 | **Pass 2 Count:** 6 | **Status:** MATCH: +20%

### Migration Guide Summary

The `Camera.hdr` field has been removed and split into a new marker component `Hdr` found at `bevy::render::view::Hdr`. Instead of setting `hdr: true` in the `Camera` component initialization, you now spawn the `Hdr` component alongside `Camera3d` or `Camera2d`. This change allows rendering effects to use `#[require(Hdr)]` for HDR-only functionality.

### Required Changes

> **IMPORTANT**: Changes in this section require adding `use bevy::render::view::Hdr;` import to `src/camera/cameras.rs`. See the "bevy_render reorganization" section above for the consolidated final import block that combines this change with other import reorganizations.

#### Component Insertion Order

When adding the `Hdr` component to camera spawn chains, follow this order:

1. **First**: Spawn the camera bundle (e.g., `Camera3d`, `PanOrbitCamera`)
2. **Second**: Insert `Hdr` component immediately after spawn
3. **Third**: Insert `Camera` component with configuration
4. **Fourth**: Insert post-processing components (`Tonemapping`, `Bloom`, etc.)

**Rationale**: The `Hdr` marker must be present before the `Camera` component initializes to ensure HDR rendering capabilities are available. Post-processing effects like `Tonemapping` and `Bloom` require `Hdr` to function correctly.

**1. Update HDR camera spawn in stars camera in `src/camera/cameras.rs`**
```diff
  commands
      .spawn(Camera3d::default())
-     .insert(Camera {
-         order: CameraOrder::Stars.order(),
-         hdr: true, // 1. HDR is required for bloom
-         ..default()
-     })
+     .insert(Hdr)
+     .insert(Camera {
+         order: CameraOrder::Stars.order(),
+         ..default()
+     })
      .insert(Tonemapping::BlenderFilmic)
```

**2. Update HDR camera spawn in panorbit camera in `src/camera/cameras.rs`**
```diff
  commands
      .spawn(PanOrbitCamera {
          focus: Vec3::ZERO,
          radius: Some(initial_radius),
          button_orbit: MouseButton::Middle,
          button_pan: MouseButton::Middle,
          modifier_pan: Some(KeyCode::ShiftLeft),
          zoom_sensitivity: 0.1,
          trackpad_behavior: TrackpadBehavior::BlenderLike {
              modifier_pan:  Some(KeyCode::ShiftLeft),
              modifier_zoom: Some(KeyCode::ControlLeft),
          },
          trackpad_pinch_to_zoom_enabled: true,
          ..default()
      })
-     .insert(Camera {
-         hdr: true,
-         order: CameraOrder::Game.order(),
-         clear_color: ClearColorConfig::Custom(
-             camera_config.clear_color.darker(camera_config.darkening_factor),
-         ),
-         ..default()
-     })
+     .insert(Hdr)
+     .insert(Camera {
+         order: CameraOrder::Game.order(),
+         clear_color: ClearColorConfig::Custom(
+             camera_config.clear_color.darker(camera_config.darkening_factor),
+         ),
+         ..default()
+     })
      .insert(Tonemapping::TonyMcMapface)
```

**3. Add import for `Hdr` component in `src/camera/cameras.rs`**
```diff
  use bevy::{
      core_pipeline::{
          bloom::Bloom,
          tonemapping::Tonemapping,
      },
      prelude::*,
-     render::view::RenderLayers,
+     render::view::{Hdr, RenderLayers},
  };
```

### Search Pattern

To find all occurrences:
```bash
rg "hdr: true" --type rust
```

---

## Collision Event System Migration (Avian + Bevy 0.17 Event Split)

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/event_split.md`
**Requirement Level:** REQUIRED
**Occurrences:** 1 collision handler + 7 files using physics components
**Pass 1 Count:** 1 | **Pass 2 Count:** 1 | **Status:** MATCH

### Migration Guide Summary

This migration combines TWO changes:
1. **Bevy 0.17 Event System**: `EventReader` → `MessageReader` (Bevy's buffered events are now called "messages")
2. **Avian Physics Migration**: `bevy_rapier3d::CollisionEvent` → `avian3d::CollisionStart` (different event structure)

**CRITICAL**: Since we're migrating from bevy_rapier3d to Avian 3D, collision events have a completely different API. Avian uses separate `CollisionStart` and `CollisionEnd` events (not an enum), with different field names.

### Avian Collision Event Structure

**Rapier (OLD)**:
```rust
pub enum CollisionEvent {
    Started(Entity, Entity, CollisionEventFlags),
    Stopped(Entity, Entity, CollisionEventFlags),
}
```

**Avian (NEW)**:
```rust
#[derive(EntityEvent, Message, Clone, Copy)]
pub struct CollisionStart {
    pub collider1: Entity,  // First collider entity
    pub collider2: Entity,  // Second collider entity
    pub body1: Option<Entity>,  // Rigid body parent (if any)
    pub body2: Option<Entity>,  // Rigid body parent (if any)
}

#[derive(EntityEvent, Message, Clone, Copy)]
pub struct CollisionEnd {
    pub collider1: Entity,
    pub collider2: Entity,
    pub body1: Option<Entity>,
    pub body2: Option<Entity>,
}
```

**Key Difference**: Avian events are structs with direct field access, not enum variants.

### CollisionEventsEnabled Requirement

**CRITICAL**: Avian requires opt-in for collision events via the `CollisionEventsEnabled` component:

```rust
commands.spawn((
    Collider::sphere(1.0),
    RigidBody::Dynamic,
    CollisionEventsEnabled,  // REQUIRED for collision events!
    // ... other components
));
```

Collision events are **only sent** for contacts where **at least one** of the colliders has `CollisionEventsEnabled`.

### MessageReader vs Observer Decision

Avian supports both patterns:
- **MessageReader**: Bulk process all collisions in one system (RECOMMENDED for nateroids)
- **Observer**: Entity-specific collision handlers

**Recommendation for Nateroids**: Use `MessageReader<CollisionStart>` because:
- Bidirectional damage logic (entity1 → entity2 AND entity2 → entity1)
- Many simultaneous collisions (bullets, asteroids, ship)
- Generic collision processing (ANY entity with CollisionDamage)
- Better performance for bulk collision processing

Reserve observers for future entity-specific triggers (pressure plates, goal zones, pickups).

### Required Changes

**1. Update collision detection system in `src/actor/collision_detection.rs`**

```diff
  use bevy::prelude::*;
- use bevy_rapier3d::prelude::CollisionEvent;
+ use avian3d::prelude::*;

  fn handle_collision_events(
-     mut collision_events: EventReader<CollisionEvent>,
+     mut collision_reader: MessageReader<CollisionStart>,
      mut health_query: Query<&mut Health>,
      name_query: Query<&Name>,
      collision_damage_query: Query<&CollisionDamage>,
  ) {
-     for &collision_event in collision_events.read() {
-         if let CollisionEvent::Started(entity1, entity2, ..) = collision_event
-             && let Ok(name1) = name_query.get(entity1)
-             && let Ok(name2) = name_query.get(entity2)
+     for event in collision_reader.read() {
+         let entity1 = event.collider1;
+         let entity2 = event.collider2;
+
+         if let Ok(name1) = name_query.get(entity1)
+             && let Ok(name2) = name_query.get(entity2)
          {
              apply_collision_damage(
                  &mut health_query,
                  &collision_damage_query,
                  entity1,
                  name1,
                  entity2,
                  name2,
              );
              apply_collision_damage(
                  &mut health_query,
                  &collision_damage_query,
                  entity2,
                  name2,
                  entity1,
                  name1,
              );
          }
      }
  }
```

**2. Add CollisionEventsEnabled when spawning actors in `src/actor/actor_spawner.rs`**

```diff
  commands.spawn((
      ActorBundle::new(config, parent, boundary),
+     CollisionEventsEnabled,  // Required for Avian collision events
  ));
```

**3. Update PhysicsPlugin implementation (src/physics.rs) - CRITICAL**

**IMPORTANT**: This codebase uses a custom `PhysicsPlugin` wrapper (defined in `src/physics.rs`) that internally registers Rapier plugins. You must update the **entire PhysicsPlugin implementation**, not just imports in main.rs.

**Complete file transformation:**

```rust
// BEFORE (Rapier):
use crate::global_input::GlobalAction;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    DebugRenderContext,
    NoUserData,
    RapierDebugRenderPlugin,
    RapierPhysicsPlugin,
};
use leafwing_input_manager::action_state::ActionState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, init_physics_debug_aabb)
            .add_systems(Update, toggle_physics_debug);
    }
}

fn init_physics_debug_aabb(mut rapier_debug: ResMut<DebugRenderContext>) {
    rapier_debug.enabled = false;
}

fn toggle_physics_debug(
    user_input: Res<ActionState<GlobalAction>>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    if user_input.just_pressed(&GlobalAction::PhysicsAABB) {
        rapier_debug.enabled = !rapier_debug.enabled;
        println!("Physics debug: {}", rapier_debug.enabled);
    }
}

// AFTER (Avian):
use crate::global_input::GlobalAction;
use bevy::prelude::*;
use avian3d::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .add_plugins(PhysicsDebugPlugin::default())
            .add_systems(Startup, init_physics_debug_aabb)
            .add_systems(Update, toggle_physics_debug);
    }
}

fn init_physics_debug_aabb(mut debug_config: ResMut<PhysicsDebugConfig>) {
    debug_config.enabled = false;
}

fn toggle_physics_debug(
    user_input: Res<ActionState<GlobalAction>>,
    mut debug_config: ResMut<PhysicsDebugConfig>,
) {
    if user_input.just_pressed(&GlobalAction::PhysicsAABB) {
        debug_config.enabled = !debug_config.enabled;
        println!("Physics debug: {}", debug_config.enabled);
    }
}
```

**Key changes:**
1. **Import changes:**
   - Remove: `bevy_rapier3d::prelude::{DebugRenderContext, NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin}`
   - Add: `avian3d::prelude::*`

2. **Plugin registration:**
   - `RapierPhysicsPlugin::<NoUserData>::default()` → `PhysicsPlugins::default()`
   - `RapierDebugRenderPlugin::default()` → `PhysicsDebugPlugin::default()`
   - Note: `PhysicsPlugins` is a plugin group that includes all core Avian physics plugins

3. **Resource type changes:**
   - `DebugRenderContext` → `PhysicsDebugConfig`
   - Both resources have `.enabled` field with same behavior

4. **No changes needed for:**
   - System registration (Startup, Update schedules remain the same)
   - System logic (toggle behavior identical)

**Verification:**
After updating, main.rs requires NO changes - it already uses the custom `PhysicsPlugin` wrapper:
```rust
// main.rs - no changes needed, already correct:
app.add_plugins((
    // ...
    PhysicsPlugin,  // This wrapper now internally uses Avian
    // ...
))
```

### Additional Files Requiring Physics Component Updates

The following files use bevy_rapier3d components and need migration to Avian equivalents:

1. **`src/actor/spaceship_control.rs`**: Uses `Velocity` → Update to `LinearVelocity` + `AngularVelocity`
2. **`src/actor/missile.rs`**: Uses rapier prelude → Update imports to `avian3d::prelude::*`
3. **`src/actor/actor_template.rs`**: Uses rapier types → Update to Avian equivalents
4. **`src/state.rs`**: Uses `RapierConfiguration` for pause/unpause → Update to `Time<Physics>` resource (see detailed section below)
5. **`src/playfield/portals.rs`**: Uses `Velocity` → Update to `LinearVelocity`

### Physics Pause/Unpause Migration (CRITICAL)

**File**: `src/state.rs`

Rapier and Avian use fundamentally different approaches for pausing physics simulation.

**Architectural Difference:**
- **Rapier**: Uses a component (`RapierConfiguration`) attached to an entity with `physics_pipeline_active` boolean field
- **Avian**: Uses Bevy's native time system with the `Time<Physics>` resource and `.pause()`/`.unpause()` methods

**Key Advantages of Avian's Approach:**
1. **Simpler API**: No query error handling needed
2. **Resource-based**: More idiomatic Bevy pattern for global configuration
3. **Better integration**: Uses Bevy's standard time system
4. **More features**: Built-in speed control, manual stepping, pause state queries

**Complete Migration:**

```rust
// BEFORE (Rapier):
use bevy_rapier3d::plugin::RapierConfiguration;

fn pause_rapier(mut rapier_config_query: Query<&mut RapierConfiguration>) {
    if let Ok(mut rapier_config) = rapier_config_query.single_mut() {
        println!("pausing game and physics");
        rapier_config.physics_pipeline_active = false;
    } else {
        error!("Error: Unable to find the RapierConfiguration component.");
    }
}

fn unpause_rapier(mut rapier_config_query: Query<&mut RapierConfiguration>) {
    if let Ok(mut rapier_config) = rapier_config_query.single_mut() {
        println!("unpausing game and physics");
        rapier_config.physics_pipeline_active = true;
    } else {
        error!("Error: Unable to find the RapierConfiguration component.");
    }
}

// System registration:
.add_systems(OnEnter(IsPaused::Paused), pause_rapier)
.add_systems(OnEnter(IsPaused::NotPaused), unpause_rapier)

// AFTER (Avian):
use avian3d::prelude::*;

fn pause_physics(mut time: ResMut<Time<Physics>>) {
    println!("pausing game and physics");
    time.pause();
}

fn unpause_physics(mut time: ResMut<Time<Physics>>) {
    println!("unpausing game and physics");
    time.unpause();
}

// System registration (update function names):
.add_systems(OnEnter(IsPaused::Paused), pause_physics)
.add_systems(OnEnter(IsPaused::NotPaused), unpause_physics)
```

**Import Changes:**
```rust
// REMOVE:
use bevy_rapier3d::plugin::RapierConfiguration;

// ADD (if not already present):
use avian3d::prelude::*;
```

**Comparison Table:**

| Aspect | Rapier | Avian |
|--------|--------|-------|
| **Type** | Component (`Query<&mut RapierConfiguration>`) | Resource (`ResMut<Time<Physics>>`) |
| **Access Pattern** | Query with error handling | Direct resource access |
| **Pause API** | `config.physics_pipeline_active = false` | `time.pause()` |
| **Unpause API** | `config.physics_pipeline_active = true` | `time.unpause()` |
| **Error Handling** | Must handle query failure | Resource guaranteed to exist |
| **Additional Features** | Limited to on/off | Speed control, manual stepping, pause checking |

**Setup Requirements:**

**None!** The `Time<Physics>` resource is automatically inserted by `PhysicsPlugins`:
```rust
app.add_plugins(PhysicsPlugins::default())
```

Unlike Rapier's component-based approach, you don't need to spawn an entity with configuration. The resource is global and always available.

**Optional: Check Pause State**

If you need to query whether physics is paused elsewhere in your code:
```rust
fn check_physics_state(time: Res<Time<Physics>>) {
    if time.is_paused() {
        println!("Physics is paused");
    }
}
```

**Optional: Advanced Time Control**

Avian provides additional time control features:
```rust
// Slow motion (50% speed)
time.set_relative_speed(0.5);

// Manual stepping (advance one physics tick)
time.advance_by(time.delta());

// Check elapsed physics time
let physics_elapsed = time.elapsed_secs();
```

**Verification:**
```bash
# Compile check
cargo build

# Runtime test:
# 1. Launch game
# 2. Press pause key (bound to GlobalAction::Pause)
# 3. Verify physics stops (objects freeze)
# 4. Press pause again
# 5. Verify physics resumes (objects continue moving)
```

### ActorBundle Structure Migration (CRITICAL)

**File**: `src/actor/actor_spawner.rs`

The `ActorBundle` struct contains multiple Rapier-specific components that must be migrated to Avian equivalents. This is a CRITICAL change as every actor spawn will fail to compile until all fields are updated.

**Current ActorBundle structure (Rapier):**
```rust
use bevy_rapier3d::prelude::*;

pub struct ActorBundle {
    pub active_events: ActiveEvents,
    pub collision_groups: CollisionGroups,
    pub gravity_scale: GravityScale,
    pub mass_properties: ColliderMassProperties,
    pub damping: Damping,
    pub locked_axes: LockedAxes,
    pub restitution: Restitution,
    pub friction: Friction,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    // ... other non-physics fields
}
```

**Updated ActorBundle structure (Avian):**
```rust
use avian3d::prelude::*;

pub struct ActorBundle {
    // REMOVED: active_events (Avian uses CollisionEventsEnabled component instead)
    pub collision_layers: CollisionLayers,  // was: collision_groups
    pub gravity_scale: GravityScale,        // same name, different module
    pub mass: Mass,                         // was: mass_properties
    pub damping: LinearDamping,             // was: Damping (split into Linear/Angular)
    pub angular_damping: AngularDamping,    // NEW: separate component
    pub locked_axes: LockedAxes,            // same name, verify API
    pub restitution: Restitution,           // same name, different structure
    pub friction: Friction,                 // same name, different structure
    pub rigid_body: RigidBody,              // same name, different module
    pub collider: Collider,                 // same name, different module
    // ... other non-physics fields
}
```

**Field-by-field migration:**

1. **`active_events: ActiveEvents` → REMOVED**
   ```rust
   // OLD (Rapier):
   active_events: ActiveEvents::COLLISION_EVENTS,

   // NEW (Avian):
   // Remove this field entirely. Instead, add CollisionEventsEnabled when spawning:
   commands.spawn((
       ActorBundle::new(config, parent, boundary),
       CollisionEventsEnabled,  // Add as separate component
   ));
   ```

2. **`collision_groups: CollisionGroups` → `collision_layers: CollisionLayers`**
   ```rust
   // OLD (Rapier):
   collision_groups: config.collision_groups,  // Type: CollisionGroups

   // NEW (Avian):
   collision_layers: config.collision_layers,  // Type: CollisionLayers

   // Note: ActorConfig struct also needs updating:
   // pub collision_groups: CollisionGroups,  // OLD
   // pub collision_layers: CollisionLayers,  // NEW
   ```

3. **`gravity_scale: GravityScale` → `gravity_scale: GravityScale`**
   ```rust
   // OLD (Rapier):
   gravity_scale: GravityScale(config.gravity_scale),

   // NEW (Avian):
   gravity_scale: GravityScale(config.gravity_scale),  // Same wrapper syntax
   ```

4. **`mass_properties: ColliderMassProperties` → `mass: Mass`**
   ```rust
   // OLD (Rapier):
   mass_properties: ColliderMassProperties::Mass(config.mass),

   // NEW (Avian):
   mass: Mass(config.mass),  // Simplified to direct Mass component
   ```

5. **`damping: Damping` → `damping: LinearDamping` + `angular_damping: AngularDamping`**
   ```rust
   // OLD (Rapier - combined linear and angular):
   damping: Damping {
       linear_damping: config.linear_damping,
       angular_damping: config.angular_damping,
   },

   // NEW (Avian - separate components):
   damping: LinearDamping(config.linear_damping),
   angular_damping: AngularDamping(config.angular_damping),
   ```

6. **`restitution: Restitution` → `restitution: Restitution`**
   ```rust
   // OLD (Rapier):
   restitution: Restitution {
       coefficient: config.restitution,
       combine_rule: config.restitution_combine_rule,
   },

   // NEW (Avian):
   restitution: Restitution::new(config.restitution)
       .with_combine_rule(config.restitution_combine_rule),
   ```

7. **`friction: Friction` → `friction: Friction`**
   ```rust
   // OLD (Rapier):
   friction: Friction {
       coefficient: config.friction,
       combine_rule: config.friction_combine_rule,
   },

   // NEW (Avian):
   friction: Friction::new(config.friction)
       .with_combine_rule(config.friction_combine_rule),
   ```

**Complete ActorBundle::new() before/after:**

```rust
// BEFORE (Rapier):
impl ActorBundle {
    pub fn new(config: &ActorConfig, parent: Entity, boundary: &Boundary) -> Self {
        Self {
            active_events: ActiveEvents::COLLISION_EVENTS,
            collision_groups: config.collision_groups,
            gravity_scale: GravityScale(config.gravity_scale),
            mass_properties: ColliderMassProperties::Mass(config.mass),
            damping: Damping {
                linear_damping: config.linear_damping,
                angular_damping: config.angular_damping,
            },
            locked_axes: config.locked_axes,
            restitution: Restitution {
                coefficient: config.restitution,
                combine_rule: config.restitution_combine_rule,
            },
            friction: Friction {
                coefficient: config.friction,
                combine_rule: config.friction_combine_rule,
            },
            rigid_body: RigidBody::Dynamic,
            collider: create_collider(config),
            // ... other fields
        }
    }
}

// AFTER (Avian):
impl ActorBundle {
    pub fn new(config: &ActorConfig, parent: Entity, boundary: &Boundary) -> Self {
        Self {
            // active_events REMOVED - use CollisionEventsEnabled component instead
            collision_layers: config.collision_layers,
            gravity_scale: GravityScale(config.gravity_scale),
            mass: Mass(config.mass),
            damping: LinearDamping(config.linear_damping),
            angular_damping: AngularDamping(config.angular_damping),
            locked_axes: config.locked_axes,
            restitution: Restitution::new(config.restitution)
                .with_combine_rule(config.restitution_combine_rule),
            friction: Friction::new(config.friction)
                .with_combine_rule(config.friction_combine_rule),
            rigid_body: RigidBody::Dynamic,
            collider: create_collider(config),
            // ... other fields
        }
    }
}
```

**ActorConfig struct changes:**

The configuration struct must also be updated to match:

```rust
// Update field types in ActorConfig:
pub struct ActorConfig {
    // pub collision_groups: CollisionGroups,  // REMOVE
    pub collision_layers: CollisionLayers,     // ADD
    // ... other fields remain the same except types
}
```

### VelocityBehavior::calculate_velocity() API Migration (CRITICAL)

**File**: `src/actor/actor_spawner.rs`

The `VelocityBehavior::calculate_velocity()` method generates velocity values for spawned actors. This method must be updated to work with Avian's split velocity components.

**Key Changes:**
1. Return type: `Velocity` → `(LinearVelocity, AngularVelocity)` tuple
2. Parameter type: `Option<&Velocity>` → `Option<&LinearVelocity>`
3. Constructor calls: `Velocity::linear(vec)` → `LinearVelocity(vec)`
4. Zero values: `Velocity::zero()` → `(LinearVelocity::ZERO, AngularVelocity::ZERO)`

**Migration Steps:**

**1. Update calculate_velocity method signature and implementation:**

```rust
// BEFORE (Rapier):
fn calculate_velocity(
    &self,
    parent_velocity: Option<&Velocity>,
    parent_transform: Option<&Transform>,
) -> Velocity {
    match self {
        VelocityBehavior::Fixed(velocity) => Velocity::linear(*velocity),
        VelocityBehavior::Random { linvel, angvel } => Velocity {
            linvel: random_vec3(-*linvel..*linvel, -*linvel..*linvel, 0.0..0.0),
            angvel: random_vec3(-*angvel..*angvel, -*angvel..*angvel, -*angvel..*angvel),
        },
        VelocityBehavior::RelativeToParent { base_velocity, inherit_parent_velocity } => {
            if let (Some(parent_velocity), Some(parent_transform)) = (parent_velocity, parent_transform) {
                let forward = -parent_transform.forward();
                let mut velocity = forward * *base_velocity;
                if *inherit_parent_velocity {
                    velocity += parent_velocity.linvel;
                }
                Velocity::linear(velocity)
            } else {
                Velocity::zero()
            }
        },
    }
}

// AFTER (Avian):
fn calculate_velocity(
    &self,
    parent_linear_velocity: Option<&LinearVelocity>,
    parent_transform: Option<&Transform>,
) -> (LinearVelocity, AngularVelocity) {
    match self {
        VelocityBehavior::Fixed(velocity) => (
            LinearVelocity(*velocity),
            AngularVelocity::ZERO,
        ),
        VelocityBehavior::Random { linvel, angvel } => (
            LinearVelocity(random_vec3(-*linvel..*linvel, -*linvel..*linvel, 0.0..0.0)),
            AngularVelocity(random_vec3(-*angvel..*angvel, -*angvel..*angvel, -*angvel..*angvel)),
        ),
        VelocityBehavior::RelativeToParent { base_velocity, inherit_parent_velocity } => {
            if let (Some(parent_linear), Some(parent_transform)) = (parent_linear_velocity, parent_transform) {
                let forward = -parent_transform.forward();
                let mut velocity = forward * *base_velocity;
                if *inherit_parent_velocity {
                    velocity += parent_linear.0;  // Access inner Vec3
                }
                (LinearVelocity(velocity), AngularVelocity::ZERO)
            } else {
                (LinearVelocity::ZERO, AngularVelocity::ZERO)
            }
        },
    }
}
```

**2. Update ActorBundle fields:**

```rust
// BEFORE:
pub struct ActorBundle {
    pub velocity: Velocity,
    // ... other fields
}

// AFTER:
pub struct ActorBundle {
    pub linear_velocity: LinearVelocity,
    pub angular_velocity: AngularVelocity,
    // ... other fields
}
```

**3. Update ActorBundle::new() to destructure tuple:**

```rust
// BEFORE:
let parent_velocity = parent.map(|(_, v, _)| v);
let parent_transform = parent.map(|(t, _, _)| t);

let velocity = config
    .velocity_behavior
    .calculate_velocity(parent_velocity, parent_transform);

Self {
    velocity,
    // ... other fields
}

// AFTER:
let parent_linear_velocity = parent.map(|(_, v, _)| v);
let parent_transform = parent.map(|(t, _, _)| t);

let (linear_velocity, angular_velocity) = config
    .velocity_behavior
    .calculate_velocity(parent_linear_velocity, parent_transform);

Self {
    linear_velocity,
    angular_velocity,
    // ... other fields
}
```

**4. Update spawn_actor signature and all callsites:**

**File**: `src/actor/actor_spawner.rs`
```rust
// BEFORE:
pub fn spawn_actor<'a>(
    commands: &'a mut Commands,
    config: &ActorConfig,
    boundary: Option<Res<Boundary>>,
    parent: Option<(&Transform, &Velocity, &Aabb)>,
) -> EntityCommands<'a> { ... }

// AFTER:
pub fn spawn_actor<'a>(
    commands: &'a mut Commands,
    config: &ActorConfig,
    boundary: Option<Res<Boundary>>,
    parent: Option<(&Transform, &LinearVelocity, &Aabb)>,
) -> EntityCommands<'a> { ... }
```

**5. Update missile fire query (src/actor/missile.rs):**

```rust
// BEFORE:
fn fire_missile(
    q_spaceship: Query<(&Transform, &Velocity, &Aabb, Option<&ContinuousFire>), With<Spaceship>>,
    // ...
) {
    let Ok((spaceship_transform, spaceship_velocity, aabb, continuous_fire)) = q_spaceship.single() else {
        return;
    };

    spawn_actor(
        &mut commands,
        &missile_config.0,
        None,
        Some((spaceship_transform, spaceship_velocity, aabb)),
    )
}

// AFTER:
fn fire_missile(
    q_spaceship: Query<(&Transform, &LinearVelocity, &Aabb, Option<&ContinuousFire>), With<Spaceship>>,
    // ...
) {
    let Ok((spaceship_transform, spaceship_linear_velocity, aabb, continuous_fire)) = q_spaceship.single() else {
        return;
    };

    spawn_actor(
        &mut commands,
        &missile_config.0,
        None,
        Some((spaceship_transform, spaceship_linear_velocity, aabb)),
    )
}
```

**6. Update spaceship control system (src/actor/spaceship_control.rs):**

```rust
// BEFORE:
fn spaceship_movement_controls(
    mut q_spaceship: Query<(&mut Transform, &mut Velocity), With<Spaceship>>,
    // ...
) {
    if let Ok((mut spaceship_transform, mut velocity)) = q_spaceship.single_mut() {
        if controls.pressed(&SpaceshipControl::TurnRight) {
            velocity.angvel.z = 0.0;
            rotation = rotation_speed * delta_seconds;
        } else if controls.pressed(&SpaceshipControl::TurnLeft) {
            velocity.angvel.z = 0.0;
            rotation = -rotation_speed * delta_seconds;
        }

        if controls.pressed(&SpaceshipControl::Accelerate) {
            apply_acceleration(&mut velocity, ...);
        }
    }
}

fn apply_acceleration(
    velocity: &mut Velocity,
    direction: Vec3,
    // ...
) {
    let proposed_velocity = velocity.linvel + direction * (acceleration * delta_seconds);
    if proposed_speed > max_speed {
        velocity.linvel = proposed_velocity.normalize() * max_speed;
    } else {
        velocity.linvel = proposed_velocity;
    }
    velocity.linvel.z = 0.0;
}

// AFTER:
fn spaceship_movement_controls(
    mut q_spaceship: Query<(&mut Transform, &mut LinearVelocity, &mut AngularVelocity), With<Spaceship>>,
    // ...
) {
    if let Ok((mut spaceship_transform, mut linear_velocity, mut angular_velocity)) = q_spaceship.single_mut() {
        if controls.pressed(&SpaceshipControl::TurnRight) {
            angular_velocity.z = 0.0;
            rotation = rotation_speed * delta_seconds;
        } else if controls.pressed(&SpaceshipControl::TurnLeft) {
            angular_velocity.z = 0.0;
            rotation = -rotation_speed * delta_seconds;
        }

        if controls.pressed(&SpaceshipControl::Accelerate) {
            apply_acceleration(&mut linear_velocity, ...);
        }
    }
}

fn apply_acceleration(
    linear_velocity: &mut LinearVelocity,
    direction: Vec3,
    // ...
) {
    let proposed_velocity = linear_velocity.0 + direction * (acceleration * delta_seconds);
    if proposed_speed > max_speed {
        linear_velocity.0 = proposed_velocity.normalize() * max_speed;
    } else {
        linear_velocity.0 = proposed_velocity;
    }
    linear_velocity.z = 0.0;
}
```

**7. Update portals system (src/playfield/portals.rs):**

```rust
// BEFORE:
fn init_portals(
    mut q_actor: Query<(&Aabb, &Transform, &Velocity, &Teleporter, &mut ActorPortals)>,
    // ...
) {
    for (aabb, transform, velocity, teleporter, mut visual) in q_actor.iter_mut() {
        let actor_direction = velocity.linvel.normalize_or_zero();
        // ...
    }
}

// AFTER:
fn init_portals(
    mut q_actor: Query<(&Aabb, &Transform, &LinearVelocity, &Teleporter, &mut ActorPortals)>,
    // ...
) {
    for (aabb, transform, linear_velocity, teleporter, mut visual) in q_actor.iter_mut() {
        let actor_direction = linear_velocity.0.normalize_or_zero();
        // ...
    }
}
```

**Impact Summary:**
- 1 method signature change (`calculate_velocity`)
- 2 ActorBundle fields (was 1 field `velocity`, now 2 fields: `linear_velocity` + `angular_velocity`)
- 1 spawn_actor signature change
- 3 spawn callsites (missile, nateroid, spaceship spawning)
- 5 field access sites in `spaceship_control.rs` (`.angvel.z` and `.linvel` accesses)
- 1 field access site in `portals.rs` (`.linvel` access)

**Verification:**
```bash
# Find all remaining Velocity references
rg "Velocity" --type rust src/

# Find all field access patterns that need updating
rg "\.linvel|\.angvel" --type rust src/

# After migration, only imports from avian3d should remain
```

### Collider API Changes (HIGH PRIORITY)

**File**: `src/actor/actor_spawner.rs`

Avian introduces breaking changes to collider construction APIs that require code updates.

#### 1. ColliderType Enum Variant Rename

**Change:** `ColliderType::Ball` → `ColliderType::Sphere`

**Location:** Enum definition at `/Users/natemccoy/rust/nateroids/src/actor/actor_spawner.rs:79-83`

```rust
// BEFORE:
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum ColliderType {
    Ball,
    Cuboid,
}

// AFTER:
#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum ColliderType {
    Sphere,  // Renamed from Ball
    Cuboid,
}
```

#### 2. Pattern Match Update

**Location:** `/Users/natemccoy/rust/nateroids/src/actor/actor_spawner.rs:445-451`

```rust
// BEFORE:
let collider = match config.collider_type {
    ColliderType::Ball => {
        let radius = size.length() / 3.;
        Collider::ball(radius)
    },
    ColliderType::Cuboid => Collider::cuboid(half_extents.x, half_extents.y, half_extents.z),
};

// AFTER:
let collider = match config.collider_type {
    ColliderType::Sphere => {  // Renamed from Ball
        let radius = size.length() / 3.;
        Collider::sphere(radius)  // Renamed from ball
    },
    ColliderType::Cuboid => {
        // CRITICAL: Avian uses full extents, not half extents!
        // Must double the values from half_extents
        Collider::cuboid(
            half_extents.x * 2.0,
            half_extents.y * 2.0,
            half_extents.z * 2.0
        )
    },
};
```

#### 3. CRITICAL: Cuboid Half-Extents → Full Extents

**⚠️ BREAKING CHANGE:** Avian's `Collider::cuboid()` uses **full extents** instead of **half-extents**!

**API Difference:**
- **Rapier:** `Collider::cuboid(hx, hy, hz)` where parameters are **half** the width/height/depth
- **Avian:** `Collider::cuboid(x_length, y_length, z_length)` where parameters are **full** width/height/depth

**Why this matters:** If you pass the same values to both APIs, Avian colliders will be **half the size** they should be, breaking all collision detection!

**Affected locations:**

**Location 1:** Default value at `/Users/natemccoy/rust/nateroids/src/actor/actor_spawner.rs:178`
```rust
// BEFORE:
collider: Collider::cuboid(0.5, 0.5, 0.5),  // half extents

// AFTER:
collider: Collider::cuboid(1.0, 1.0, 1.0),  // full extents (doubled)
```

**Location 2:** Runtime construction (already shown in pattern match above)

#### 4. Inspector/Reflection Notes

- `ColliderType` derives `Reflect` and is visible in inspector UI via `ActorConfig`
- Inspector will display "Sphere" instead of "Ball" after migration
- The enum is used in configuration for all actor types (Missile, Nateroid, Spaceship)
- If you have serialized config files (`.ron`, `.json`), they may need manual updates if they reference `Ball`

#### 5. Verification Steps

**Visual testing (CRITICAL):**
```bash
# After migration, launch the game and verify:
cargo run

# 1. Spawn each actor type (Missile, Nateroid, Spaceship)
# 2. Enable physics debug visualization (GlobalAction::PhysicsAABB)
# 3. Verify collision bounds visually match pre-migration size
# 4. Test collisions between actors to ensure physics behave identically
```

**Search for remaining references:**
```bash
# Should find no results after migration:
rg "ColliderType::Ball" --type rust
rg "Collider::ball" --type rust

# Verify all cuboid calls are updated:
rg "Collider::cuboid" --type rust
```

### CollisionGroups → CollisionLayers Migration (CRITICAL)

**Files**: `src/actor/actor_template.rs`, `src/actor/actor_spawner.rs`

Avian replaces Rapier's `CollisionGroups` with a type-safe `CollisionLayers` API using enums instead of raw bit constants.

#### Key API Differences

| Aspect | Rapier | Avian |
|--------|--------|-------|
| **Type name** | `CollisionGroups` | `CollisionLayers` |
| **Constants** | `Group::GROUP_1`, `Group::GROUP_2` | Custom enum with `#[derive(PhysicsLayer)]` |
| **Constructor** | `CollisionGroups::new(membership, filter)` | `CollisionLayers::new(memberships, filters)` |
| **Type safety** | Raw bitmask, error-prone | Enum-based, compile-time checked |

#### Step 1: Replace Group Constants with PhysicsLayer Enum

**File**: `src/actor/actor_template.rs`

```rust
// BEFORE (Rapier):
use bevy_rapier3d::{
    dynamics::LockedAxes,
    geometry::Group,
    prelude::CollisionGroups,
};

pub const GROUP_SPACESHIP: Group = Group::GROUP_1;
pub const GROUP_ASTEROID: Group = Group::GROUP_2;
pub const GROUP_MISSILE: Group = Group::GROUP_3;

// AFTER (Avian):
use avian3d::{
    dynamics::locked_axes::LockedAxes,
    prelude::{CollisionLayers, PhysicsLayer},
};

#[derive(PhysicsLayer, Clone, Copy, Debug, Default)]
pub enum GameLayer {
    #[default]
    Default,    // Layer 0 - reserved for default
    Spaceship,  // Layer 1 (was GROUP_1)
    Asteroid,   // Layer 2 (was GROUP_2)
    Missile,    // Layer 3 (was GROUP_3)
}
```

**Benefits:**
- **Type-safe**: Can't accidentally use wrong bit values
- **Self-documenting**: Clear what each layer represents
- **Auto-assigned bits**: No manual bit management

#### Step 2: Update Actor Configurations

**File**: `src/actor/actor_template.rs`

**MissileConfig:**
```rust
// BEFORE (Rapier):
impl Default for MissileConfig {
    fn default() -> Self {
        Self(ActorConfig {
            collision_groups: CollisionGroups::new(GROUP_MISSILE, GROUP_ASTEROID),
            // ...
        })
    }
}

// AFTER (Avian):
impl Default for MissileConfig {
    fn default() -> Self {
        Self(ActorConfig {
            collision_layers: CollisionLayers::new(
                GameLayer::Missile,    // This entity belongs to Missile layer
                GameLayer::Asteroid,   // This entity collides with Asteroid layer
            ),
            // ...
        })
    }
}
```

**SpaceshipConfig:**
```rust
// BEFORE (Rapier):
impl Default for SpaceshipConfig {
    fn default() -> Self {
        Self(ActorConfig {
            collision_groups: CollisionGroups::new(GROUP_SPACESHIP, GROUP_ASTEROID),
            // ...
        })
    }
}

// AFTER (Avian):
impl Default for SpaceshipConfig {
    fn default() -> Self {
        Self(ActorConfig {
            collision_layers: CollisionLayers::new(
                GameLayer::Spaceship,  // This entity belongs to Spaceship layer
                GameLayer::Asteroid,   // This entity collides with Asteroid layer
            ),
            // ...
        })
    }
}
```

**NateroidConfig (multiple collision targets):**
```rust
// BEFORE (Rapier) - Uses default (collides with everything):
impl Default for NateroidConfig {
    fn default() -> Self {
        Self(ActorConfig {
            // collision_groups: CollisionGroups::default(),  // Implicit
            // ...
        })
    }
}

// AFTER (Avian) - Explicit collision targets:
impl Default for NateroidConfig {
    fn default() -> Self {
        Self(ActorConfig {
            collision_layers: CollisionLayers::new(
                GameLayer::Asteroid,                         // Belongs to Asteroid layer
                [GameLayer::Spaceship, GameLayer::Missile],  // Collides with both
            ),
            // ...
        })
    }
}
```

#### Step 3: Update ActorConfig Structure

**File**: `src/actor/actor_spawner.rs`

```rust
// BEFORE (Rapier):
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct ActorConfig {
    // ...
    #[reflect(ignore)]
    pub collision_groups: CollisionGroups,
    // ...
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            // ...
            collision_groups: CollisionGroups::default(),
            // ...
        }
    }
}

// AFTER (Avian):
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct ActorConfig {
    // ...
    #[reflect(ignore)]
    pub collision_layers: CollisionLayers,
    // ...
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            // ...
            collision_layers: CollisionLayers::default(),  // All layers, all filters
            // ...
        }
    }
}
```

#### Step 4: Update ActorBundle

**File**: `src/actor/actor_spawner.rs`

```rust
// BEFORE (Rapier):
#[derive(Bundle)]
pub struct ActorBundle {
    // ...
    pub collision_groups: CollisionGroups,
    // ...
}

impl ActorBundle {
    pub fn new(config: &ActorConfig, parent: ..., boundary: ...) -> Self {
        Self {
            // ...
            collision_groups: config.collision_groups,
            // ...
        }
    }
}

// AFTER (Avian):
#[derive(Bundle)]
pub struct ActorBundle {
    // ...
    pub collision_layers: CollisionLayers,
    // ...
}

impl ActorBundle {
    pub fn new(config: &ActorConfig, parent: ..., boundary: ...) -> Self {
        Self {
            // ...
            collision_layers: config.collision_layers,
            // ...
        }
    }
}
```

#### Behavioral Equivalence

The migration maintains identical collision behavior:

**Current Rapier behavior:**
- **Missile**: Belongs to GROUP_3, collides with GROUP_2 (asteroids only)
- **Spaceship**: Belongs to GROUP_1, collides with GROUP_2 (asteroids only)
- **Asteroid**: Belongs to all groups, collides with all groups

**New Avian behavior (identical):**
- **Missile**: Belongs to Missile layer, collides with Asteroid layer
- **Spaceship**: Belongs to Spaceship layer, collides with Asteroid layer
- **Asteroid**: Belongs to Asteroid layer, collides with [Spaceship, Missile]

**Collision matrix (unchanged):**
- ✓ Missile collides with Asteroid
- ✗ Missile does NOT collide with Spaceship
- ✗ Missile does NOT collide with other Missiles
- ✓ Spaceship collides with Asteroid
- ✗ Spaceship does NOT collide with Missile
- ✗ Spaceship does NOT collide with other Spaceships

#### Verification

```bash
# Find old collision group references
rg "CollisionGroups" --type rust
rg "Group::GROUP_" --type rust

# After migration, should find only new code:
rg "CollisionLayers" --type rust
rg "GameLayer::" --type rust
```

**Runtime testing:**
1. Spawn missiles, spaceships, and asteroids
2. Enable physics debug visualization (GlobalAction::PhysicsAABB)
3. Verify collision behavior matches pre-migration:
   - Missiles hit asteroids ✓
   - Missiles pass through spaceships ✓
   - Spaceships hit asteroids ✓
   - Spaceships pass through missiles ✓

### Component Mapping Table (Rapier → Avian)

| Rapier Component | Avian Equivalent |
|------------------|------------------|
| `RigidBody::Dynamic` | `RigidBody::Dynamic` (same) |
| `RigidBody::Fixed` | `RigidBody::Static` |
| `RigidBody::KinematicPositionBased` | `RigidBody::Kinematic` |
| `RigidBody::KinematicVelocityBased` | `RigidBody::Kinematic` |
| `Collider::ball(r)` | `Collider::sphere(r)` |
| `Collider::cuboid(hx, hy, hz)` | `Collider::cuboid(hx*2, hy*2, hz*2)` ⚠️ **DOUBLE VALUES!** |
| `Velocity` | `LinearVelocity` (separate from `AngularVelocity`) |
| `CollisionGroups` | `CollisionLayers` |
| `RapierConfiguration` | `Time<Physics>` resource with `.pause()`/`.unpause()` |
| `DebugRenderContext` | `PhysicsDebugConfig` |

**Note on Force Application:** This codebase does not use `ExternalForce` components. Instead, it directly modifies velocity components (`Velocity` in Rapier → `LinearVelocity` + `AngularVelocity` in Avian). This approach remains valid and is already covered in the VelocityBehavior and spaceship control system migrations above.

### LockedAxes API Migration (HIGH PRIORITY)

**Files**: `src/actor/actor_template.rs`, `src/actor/actor_spawner.rs`

**⚠️ BREAKING CHANGE:** Avian's `LockedAxes` uses a builder pattern instead of bitflag constants.

#### API Differences

| Aspect | Rapier | Avian |
|--------|--------|-------|
| **Constants** | `ROTATION_LOCKED_X`, `TRANSLATION_LOCKED_Z`, etc. | Only `TRANSLATION_LOCKED`, `ROTATION_LOCKED`, `ALL_LOCKED` |
| **API Style** | Bitflag constants with `\|` operator | Builder pattern with chainable methods |
| **Constructor** | Direct constant usage | `LockedAxes::new().lock_*_*()` |
| **Import** | `bevy_rapier3d::dynamics::LockedAxes` | `avian3d::prelude::LockedAxes` |

#### CRITICAL: Individual Axis Constants Don't Exist

Rapier provides these constants that **Avian does NOT have**:
- ❌ `LockedAxes::TRANSLATION_LOCKED_X`
- ❌ `LockedAxes::TRANSLATION_LOCKED_Y`
- ❌ `LockedAxes::TRANSLATION_LOCKED_Z`
- ❌ `LockedAxes::ROTATION_LOCKED_X`
- ❌ `LockedAxes::ROTATION_LOCKED_Y`
- ❌ `LockedAxes::ROTATION_LOCKED_Z`

Instead, Avian uses **builder methods**:
- ✅ `.lock_translation_x()`
- ✅ `.lock_translation_y()`
- ✅ `.lock_translation_z()`
- ✅ `.lock_rotation_x()`
- ✅ `.lock_rotation_y()`
- ✅ `.lock_rotation_z()`

#### Migration Steps

**1. Update import in `src/actor/actor_template.rs`:**
```rust
// BEFORE (Rapier):
use bevy_rapier3d::{
    dynamics::LockedAxes,
    geometry::Group,
    prelude::CollisionGroups,
};

// AFTER (Avian):
use avian3d::{
    dynamics::rigid_body::LockedAxes,
    prelude::{CollisionLayers, PhysicsLayer},
};
```

**2. Update SpaceshipConfig (multiple locks) in `src/actor/actor_template.rs`:**
```rust
// BEFORE (Rapier) - lines 94-96:
locked_axes: LockedAxes::ROTATION_LOCKED_X
    | LockedAxes::ROTATION_LOCKED_Y
    | LockedAxes::TRANSLATION_LOCKED_Z,

// AFTER (Avian) - builder pattern:
locked_axes: LockedAxes::new()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_translation_z(),
```

**3. Update ActorConfig default (single lock) in `src/actor/actor_spawner.rs`:**
```rust
// BEFORE (Rapier) - line 184:
locked_axes: LockedAxes::TRANSLATION_LOCKED_Z,

// AFTER (Avian):
locked_axes: LockedAxes::new().lock_translation_z(),
```

#### Behavioral Equivalence

The migration maintains identical axis locking behavior:

**Spaceship (2D-style gameplay in 3D):**
- Locks: X rotation, Y rotation, Z translation
- Result: Can only rotate around Z axis (yaw), can only move in XY plane
- Behavior: Identical in both Rapier and Avian

**Default config (most actors):**
- Locks: Z translation only
- Result: Constrained to XY plane
- Behavior: Identical in both Rapier and Avian

#### Verification

```bash
# Find old bitflag usage
rg "LockedAxes::(ROTATION|TRANSLATION)_LOCKED_[XYZ]" --type rust

# After migration, should find no results
rg "ROTATION_LOCKED_X|TRANSLATION_LOCKED_Z" --type rust

# Verify new builder pattern usage
rg "LockedAxes::new\\(\\)" --type rust
```

**Runtime testing:**
1. Spawn spaceship and move it
2. Verify it can only rotate around Z axis (yaw)
3. Verify it stays in XY plane (cannot move in Z direction)
4. Test with other actors to ensure proper constraint behavior

### Restitution and Friction Migration (LOW PRIORITY)

**File**: `src/actor/actor_spawner.rs`

Good news: The `Restitution` and `Friction` structures are nearly identical between Rapier and Avian.

#### Restitution Changes

**Type Rename Only:** `CoefficientCombineRule` → `CoefficientCombine`

```rust
// BEFORE (Rapier):
use bevy_rapier3d::prelude::*;

pub struct ActorConfig {
    pub restitution_combine_rule: CoefficientCombineRule,  // Type name
    // ...
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            restitution_combine_rule: CoefficientCombineRule::Max,  // Type name
            // ...
        }
    }
}

// Instantiation (no changes needed - identical):
restitution: Restitution {
    coefficient: config.restitution,
    combine_rule: config.restitution_combine_rule,
}

// AFTER (Avian):
use avian3d::prelude::*;

pub struct ActorConfig {
    pub restitution_combine_rule: CoefficientCombine,  // Renamed type
    // ...
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            restitution_combine_rule: CoefficientCombine::Max,  // Renamed type
            // ...
        }
    }
}

// Instantiation remains identical:
restitution: Restitution {
    coefficient: config.restitution,
    combine_rule: config.restitution_combine_rule,
}
```

**Variant Compatibility:**
All Rapier variants work identically in Avian:
- `Max` - Selects larger coefficient (unchanged)
- `Min` - Selects smaller coefficient (unchanged)
- `Multiply` - Product of coefficients (unchanged)
- `Average` - Mean of coefficients (unchanged)

**Optional Builder Pattern:**
Avian provides a builder pattern alternative (not required):
```rust
// Alternative (cleaner but optional):
restitution: Restitution::new(config.restitution)
    .with_combine_rule(config.restitution_combine_rule),
```

#### Friction Changes

**Identical to Restitution:** Same pattern applies
- Type rename: `CoefficientCombineRule` → `CoefficientCombine`
- Struct fields remain the same
- Variants unchanged

```rust
// BEFORE (Rapier):
friction: Friction {
    coefficient: config.friction,
    combine_rule: config.friction_combine_rule,
}

// AFTER (Avian) - Option 1 (struct init, minimal change):
friction: Friction {
    coefficient: config.friction,
    combine_rule: config.friction_combine_rule,
}

// AFTER (Avian) - Option 2 (builder pattern, optional):
friction: Friction::new(config.friction)
    .with_combine_rule(config.friction_combine_rule),
```

#### Migration Summary

**Required Changes:**
1. Type rename in ActorConfig: `CoefficientCombineRule` → `CoefficientCombine` (2 locations)
2. Type rename in defaults: `CoefficientCombineRule::Max` → `CoefficientCombine::Max` (2 locations)

**No Changes Needed:**
- Struct initialization patterns (identical)
- Combine rule variants (identical behavior)
- ActorBundle instantiation (identical fields)

**Verification:**
```bash
# Find old type name
rg "CoefficientCombineRule" --type rust

# After migration, should find no results
# Verify new type name
rg "CoefficientCombine" --type rust
```

### Type Registration

**No Action Required**: Analysis confirms no Rapier types were registered in the codebase, and no Avian types require explicit registration.

**Verification Results:**
- ✅ Searched codebase for `register_type` calls: No Rapier physics types found
- ✅ Checked common physics types: `CollisionEvent`, `Velocity`, `RigidBody`, `Collider` - none registered
- ✅ Bevy 0.17 auto-registration: Physics types from Avian are automatically registered via `reflect_auto_register` feature (enabled by default)

**Rationale:**
1. Rapier types were never explicitly registered in the original codebase
2. Avian's physics types implement `Reflect` and are auto-registered in Bevy 0.17
3. The 9 manual `register_type` removals documented in the "Changes to type registration for reflection" section are all game-specific config types, not physics types

**Note**: If you use bevy_inspector_egui or similar tools that require reflection for Avian types, the auto-registration feature handles this automatically. No manual registration needed.

### Verification Steps

**Phase 1 - Imports**:
```bash
# Find all rapier imports
rg "bevy_rapier3d" --type rust

# After migration, verify none remain
rg "rapier" --type rust
```

**Phase 2 - Compilation**:
```bash
cargo check
# Address any remaining compilation errors
```

**Phase 3 - Runtime Testing**:
- Spawn entities with physics bodies
- Verify collisions trigger damage application
- Test physics debug visualization with PhysicsAABB toggle
- Verify performance with many asteroids

### Search Pattern

To find all event-related code:
```bash
rg "EventReader|EventWriter" --type rust
rg "CollisionEvent" --type rust
rg "bevy_rapier3d" --type rust
```

---

## HIGH Priority Changes

## Changes to type registration for reflection

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/reflect_registration_changes.md`
**Requirement Level:** HIGH
**Occurrences:** 9 locations across 7 files
**Pass 1 Count:** 9 | **Pass 2 Count:** 9 | **Status:** MATCH

### Migration Guide Summary

In Bevy 0.17, types implementing `Reflect` are now automatically registered when the `reflect_auto_register` feature is enabled (part of default features). This eliminates the need for manual `.register_type()` calls for non-generic types. Generic types still require manual registration, but all the types found in this codebase are non-generic concrete types that can safely have their registration calls removed.

### Required Changes

**1. Remove register_type for SpaceshipControlConfig in `src/actor/spaceship_control.rs`**
```diff
 impl Plugin for SpaceshipControlPlugin {
     fn build(&self, app: &mut App) {
-        app.register_type::<SpaceshipControlConfig>()
-            .add_plugins(
+        app.add_plugins(
                 ResourceInspectorPlugin::<SpaceshipControlConfig>::default()
                     .run_if(toggle_active(false, GlobalAction::SpaceshipControlInspector)),
             )
```

**2. Remove register_type for MissileConfig in `src/actor/actor_spawner.rs`**
```diff
 impl Plugin for ActorSpawner {
     fn build(&self, app: &mut App) {
-        app.register_type::<MissileConfig>()
-            .register_type::<NateroidConfig>()
+        app.register_type::<NateroidConfig>()
             .register_type::<SpaceshipConfig>()
             .add_systems(OnEnter(AssetsState::Loaded), initialize_actor_configs)
```

**3. Remove register_type for NateroidConfig in `src/actor/actor_spawner.rs`**
```diff
 impl Plugin for ActorSpawner {
     fn build(&self, app: &mut App) {
-        app.register_type::<NateroidConfig>()
-            .register_type::<SpaceshipConfig>()
+        app.register_type::<SpaceshipConfig>()
             .add_systems(OnEnter(AssetsState::Loaded), initialize_actor_configs)
```

**4. Remove register_type for SpaceshipConfig in `src/actor/actor_spawner.rs`**
```diff
 impl Plugin for ActorSpawner {
     fn build(&self, app: &mut App) {
-        app.register_type::<SpaceshipConfig>()
-            .add_systems(OnEnter(AssetsState::Loaded), initialize_actor_configs)
+        app.add_systems(OnEnter(AssetsState::Loaded), initialize_actor_configs)
             .add_plugins(
```

**5. Remove register_type for PortalConfig in `src/playfield/portals.rs`**
```diff
 impl Plugin for PortalPlugin {
     fn build(&self, app: &mut App) {
         app.init_gizmo_group::<PortalGizmo>()
             .init_resource::<PortalConfig>()
-            .register_type::<PortalConfig>()
             .add_plugins(
                 ResourceInspectorPlugin::<PortalConfig>::default()
```

**6. Remove register_type for Boundary in `src/playfield/boundary.rs`**
```diff
 impl Plugin for BoundaryPlugin {
     fn build(&self, app: &mut App) {
         app.init_resource::<Boundary>()
             .init_gizmo_group::<BoundaryGizmo>()
-            .register_type::<Boundary>()
             .add_plugins(
                 ResourceInspectorPlugin::<Boundary>::default()
```

**7. Remove register_type for PlaneConfig in `src/playfield/planes.rs`**
```diff
 impl Plugin for PlanesPlugin {
     fn build(&self, app: &mut App) {
         app.add_systems(Update, manage_box_planes)
-            .register_type::<PlaneConfig>()
             .init_resource::<PlaneConfig>()
             .add_plugins(
```

**8. Remove register_type for LightConfig in `src/camera/lights.rs`**
```diff
 impl Plugin for DirectionalLightsPlugin {
     fn build(&self, app: &mut App) {
         app.init_resource::<AmbientLight>()
             .add_plugins(
                 ResourceInspectorPlugin::<LightConfig>::default()
                     .run_if(toggle_active(false, GlobalAction::LightsInspector)),
             )
             .init_resource::<LightConfig>()
-            .register_type::<LightConfig>()
             .add_systems(Update, manage_lighting);
```

**9. Remove register_type for CameraConfig in `src/camera/config.rs`**
```diff
 impl Plugin for CameraConfigPlugin {
     fn build(&self, app: &mut App) {
-        app.register_type::<CameraConfig>()
-            .add_plugins(
+        app.add_plugins(
                 ResourceInspectorPlugin::<CameraConfig>::default()
                     .run_if(toggle_active(false, GlobalAction::CameraConfigInspector)),
```

### Search Pattern

To find all occurrences:
```bash
rg "register_type" --type rust
```

---

## Deprecate `iter_entities` and `iter_entities_mut`

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/deprecate_iter_entities.md`
**Requirement Level:** HIGH
**Occurrences:** 1 locations across 1 files
**Pass 1 Count:** 1 | **Pass 2 Count:** 1 | **Status:** MATCH

### Migration Guide Summary

The methods `world.iter_entities()` and `world.iter_entities_mut()` are **deprecated** (not removed) in Bevy 0.17.0. They still work but generate compiler warnings.

**Why This Changed**: Bevy 0.17 introduced "entity disabling" - a feature that lets you exclude entities from queries by adding a `Disabled` component. The old `iter_entities()` returns ALL entities, while the new `world.query::<EntityRef>().iter(&world)` respects `DefaultQueryFilters` and skips disabled entities by default.

**Behavioral Difference**:
- **Old**: `iter_entities()` → Returns every entity, no exceptions
- **New**: `query::<EntityRef>().iter(&world)` → Skips entities with `Disabled` component (unless explicitly queried)

**Impact on Nateroids**: **Zero impact** - codebase doesn't use entity disabling (verified by searching for `Disabled`, `entity_disabling`, `register_disabling`). The replacement behaves identically.

### Required Changes

**1. Update scene AABB iteration in `src/actor/aabb.rs`**

**Context**: This code calculates bounding boxes during asset loading. Since no entities are disabled in the scene's internal world, the new query-based approach returns the same entities.
```diff
- for entity in scene.world.iter_entities() {
+ for entity in scene.world.query::<EntityRef>().iter(&scene.world) {
```

### Future Considerations

**If you later use entity disabling** and need to include disabled entities in AABB calculations:

**Option 1 - Include disabled entities in query**:
```rust
use bevy::ecs::component::ComponentId;
use bevy::ecs::query::With;

for entity in scene.world.query::<(EntityRef, With<Disabled>)>().iter(&scene.world) {
    // This will include disabled entities
}
```

**Option 2 - Temporarily remove default filters**:
```rust
// Remove DefaultQueryFilters resource before querying
// (Not recommended for most use cases)
```

**Documentation**: See [Bevy Entity Disabling docs](https://docs.rs/bevy/0.17.2/bevy/ecs/entity_disabling/index.html) for details.

### Search Pattern

To find all occurrences:
```bash
rg "iter_entities" --type rust
```

---

## MEDIUM Priority Changes

No MEDIUM priority changes required.

---

## LOW Priority Changes

## `Assets::insert` and `Assets::get_or_insert_with` now return `Result`

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/assets-insert-result.md`
**Requirement Level:** LOW
**Occurrences:** 0 locations across 0 files
**Pass 1 Count:** 28 | **Pass 2 Count:** 28 | **Status:** MATCH

### Migration Guide Summary

In Bevy 0.17, `Assets::insert` and `Assets::get_or_insert_with` now return `Result` instead of panicking when inserting into an `AssetId` whose handle was dropped. This codebase does not use either of these methods, so no migration is required.

### Required Changes

No changes required. The codebase uses `Assets::add()` for inserting assets, which is not affected by this migration.

### Search Pattern

To verify no occurrences exist:
```bash
rg "\.insert\(|\.get_or_insert_with\(" --type rust | rg "Assets"
```

---

## Anchor is now a required component on Sprite

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/anchor_is_removed_from_sprite.md`
**Requirement Level:** LOW
**Occurrences:** 0 locations across 0 files
**Pass 1 Count:** 20 | **Pass 2 Count:** 1 | **Status:** ANOMALY: ±95%

### Migration Guide Summary

The `anchor` field has been removed from `Sprite` and is now a separate required component. Additionally, the `Anchor` enum variants have been changed to associated constants (e.g., `Anchor::BottomLeft` becomes `Anchor::BOTTOM_LEFT`). This guide does not apply to this codebase as it is a 3D game using `Mesh3d` and does not use 2D sprites.

### Required Changes

None. This codebase does not use `Sprite` components. The single occurrence of `Camera3d` in `/Users/natemccoy/rust/nateroids/src/camera/cameras.rs` is unrelated to sprite anchoring and requires no changes for this migration.

### Search Pattern

To verify no sprite usage exists:
```bash
rg "Sprite|Anchor::" --type rust src/
```

### Variance Explanation

The 95% variance between Pass 1 (20 occurrences) and Pass 2 (1 occurrence) is due to this being a 3D game that does not use 2D sprites. The Pass 1 count appears to have incorrectly flagged patterns that don't exist in this codebase. The single occurrence found is `Camera3d` which is unrelated to sprite anchoring. This migration guide is informational only for this project.

---

## State-scoped entities are now always enabled implicitly

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/state_scoped_entities_by_default.md`
**Requirement Level:** LOW
**Occurrences:** 0 locations across 0 files
**Pass 1 Count:** 6 | **Pass 2 Count:** 10 | **Status:** ANOMALY: +67%

### Migration Guide Summary

State-scoped entities are now always enabled implicitly in Bevy 0.17.2. The `app.enable_state_scoped_entities::<State>()` method has been deprecated and does nothing when called. The `#[states(scoped_entities)]` attribute has been removed and can be safely deleted without replacement.

### Required Changes

No changes required. The codebase does not use the deprecated `enable_state_scoped_entities()` method or the `#[states(scoped_entities)]` attribute.

**Note on variance:** The anomaly in pattern counts (Pass 1: 6, Pass 2: 10) is due to the `States` pattern matching additional occurrences including `ComputedStates` trait implementations and `SourceStates` type aliases, which are unrelated to this migration. The actual deprecated features targeted by this guide are not present in the codebase.

### Search Pattern

To verify no deprecated usage exists:
```bash
rg "enable_state_scoped_entities" --type rust
rg "scoped_entities" --type rust
```

---

## Combine now takes an extra parameter

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/combine_soundness_fix.md`
**Requirement Level:** LOW
**Occurrences:** 0 locations across 0 files
**Pass 1 Count:** 4 | **Pass 2 Count:** 0 | **Status:** MATCH

### Migration Guide Summary

The `Combine::combine` method now requires an extra parameter that must be passed mutably to the two closures. This change fixes a soundness issue that could occur when closures were called re-entrantly. The codebase does not use the Bevy `Combine` trait; the 4 pattern matches were `CoefficientCombineRule` (physics) and comment text.

### Required Changes

No changes required - the codebase does not use `Combine::combine`.

### Search Pattern

To find all occurrences:
```bash
rg "Combine::combine" --type rust
```

---

## Window is now split into multiple components

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/split-window.md`
**Requirement Level:** LOW
**Occurrences:** 4 locations across 1 files
**Pass 1 Count:** 4 | **Pass 2 Count:** 4 | **Status:** MATCH

### Migration Guide Summary

The `Window` component has been split into multiple smaller components to improve internal handling and make it more approachable. This migration specifically affects `CursorOptions`, which is now a separate component instead of being nested within `Window`. When querying cursor options, you now query `CursorOptions` directly, and when configuring the primary window cursor options in `WindowPlugin`, you use `primary_cursor_options` instead of nesting `CursorOptions` within `Window`.

### Required Changes

**Note:** This codebase does not use `CursorOptions` functionality. The `Window` component appears only in WASM-specific configuration for window mode and present mode settings, which are not affected by the `CursorOptions` split. No changes are required for this migration.

### Search Pattern

To find all occurrences:
```bash
rg "Window" --type rust
```

---

## Remove `TextFont` Constructor Methods

**Guide File:** `/Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides/remove_text_font_from_constructor_methods.md`
**Requirement Level:** LOW
**Occurrences:** 2 locations across 1 file
**Pass 1 Count:** 2 | **Pass 2 Count:** 2 | **Status:** MATCH

### Migration Guide Summary

The `TextFont::from_font` and `TextFont::from_line_height` constructor methods have been removed in favor of `From` trait implementations. This codebase does not use these constructor methods - it uses struct initialization syntax (`TextFont { font_size: 1.0, ..default() }`) which remains unchanged. The occurrences found are valid uses of `TextFont` that require no migration.

### Required Changes

**No changes required.** The codebase uses `TextFont` with struct initialization syntax (line 36) and in query type signatures (line 54), neither of which are affected by this migration.

### Search Pattern

To find all occurrences:
```bash
rg "TextFont" --type rust
```

---

## Guides Not Applicable to This Codebase

The following 104 guides from Bevy 0.17.2 do not apply to this codebase:

- release-content/migration-guides/LightVisibilityClass_rename.md
- release-content/migration-guides/Newtype_ScrollPosition.md
- release-content/migration-guides/RenderTargetInfo_default.md
- release-content/migration-guides/UI_scroll_position_is_now_logical.md
- release-content/migration-guides/animation_graph_no_more_asset_ids.md
- release-content/migration-guides/check_change_ticks.md
- release-content/migration-guides/chromatic_aberration_option.md
- release-content/migration-guides/clone_behavior_no_longer_eq.md
- release-content/migration-guides/component-lifecycle-module.md
- release-content/migration-guides/component_entry.md
- release-content/migration-guides/components-registrator-derefmut.md
- release-content/migration-guides/composable_specialization.md
- release-content/migration-guides/compressed-image-saver.md
- release-content/migration-guides/cursor-android.md
- release-content/migration-guides/dragenter_includes_dragged_entity.md
- release-content/migration-guides/dynamic-bundle-movingptr.md
- release-content/migration-guides/entities_apis.md
- release-content/migration-guides/entity_cloner_builder_split.md
- release-content/migration-guides/entity_representation.md
- release-content/migration-guides/extract-picking-plugin-members.md
- release-content/migration-guides/extract-pointer-input-plugin-members.md
- release-content/migration-guides/extract_fn_is_mut.md
- release-content/migration-guides/extract_ui_text_colors_per_glyph.md
- release-content/migration-guides/extracted_uinodes_z_order.md
- release-content/migration-guides/fullscreen_shader_resource.md
- release-content/migration-guides/gated_reader.md
- release-content/migration-guides/generic-option-parameter.md
- release-content/migration-guides/glam-rand-upgrades.md
- release-content/migration-guides/gles_optional.md
- release-content/migration-guides/gltf-animation-load-optional.md
- release-content/migration-guides/handle_weak_replaced_with_handle_uuid.md
- release-content/migration-guides/incorrect-type-error-on-run-system-command.md
- release-content/migration-guides/internal_entities.md
- release-content/migration-guides/interned-labels-cleanup.md
- release-content/migration-guides/labeled_asset_scope_errors.md
- release-content/migration-guides/log-diagnostics-hash-set.md
- release-content/migration-guides/map_set_apply.md
- release-content/migration-guides/merge_observerState_observer_single_component.md
- release-content/migration-guides/mesh_compute_smooth_normals.md
- release-content/migration-guides/non-generic-access.md
- release-content/migration-guides/observer_and_event_changes.md
- release-content/migration-guides/observers_may_not_be_exclusive.md
- release-content/migration-guides/overflowclipbox_default_is_now_paddingbox.md
- release-content/migration-guides/parallelism_strategy_changes.md
- release-content/migration-guides/per-world-error-handler.md
- release-content/migration-guides/picking_location_not_component.md
- release-content/migration-guides/pointer_target.md
- release-content/migration-guides/primitives_non_const_generic_meshable.md
- release-content/migration-guides/query_items_borrow_from_query_state.md
- release-content/migration-guides/rangefinder.md
- release-content/migration-guides/reflect_asset_asset_ids.md
- release-content/migration-guides/relationship_set_risky.md
- release-content/migration-guides/relative_cursor_position_is_object_centered.md
- release-content/migration-guides/remove_archetype_component_id.md
- release-content/migration-guides/remove_bundle_register_required_components.md
- release-content/migration-guides/remove_cosmic_text_reexports.md
- release-content/migration-guides/remove_default_extend_from_iter.md
- release-content/migration-guides/remove_deprecated_batch_spawning.md
- release-content/migration-guides/remove_scale_value.md
- release-content/migration-guides/remove_the_add_sub_impls_on_volume.md
- release-content/migration-guides/removed_components_stores_messages.md
- release-content/migration-guides/rename-justifytext.md
- release-content/migration-guides/rename_condition.md
- release-content/migration-guides/rename_pointer_events.md
- release-content/migration-guides/rename_spawn_gltf_material_name.md
- release-content/migration-guides/rename_state_scoped.md
- release-content/migration-guides/rename_timer_paused_and_finished.md
- release-content/migration-guides/rename_transform_compute_matrix.md
- release-content/migration-guides/renamed_BRP_methods.md
- release-content/migration-guides/renamed_computednodetarget.md
- release-content/migration-guides/render_graph_app_to_ext.md
- release-content/migration-guides/render_startup.md
- release-content/migration-guides/render_target_info_error.md
- release-content/migration-guides/replace_non_send_resources.md
- release-content/migration-guides/required_components_rework.md
- release-content/migration-guides/rework_merge_mesh_error.md
- release-content/migration-guides/rot2_matrix_construction.md
- release-content/migration-guides/scalar-field-on-vector-space.md
- release-content/migration-guides/scene_spawner_api.md
- release-content/migration-guides/schedule_cleanup.md
- release-content/migration-guides/send_event_rename.md
- release-content/migration-guides/separate-border-colors.md
- release-content/migration-guides/simple_executor_going_away.md
- release-content/migration-guides/spawnable-list-movingptr.md
- release-content/migration-guides/specialized_ui_transform.md
- release-content/migration-guides/split_up_computeduitargetcamera.md
- release-content/migration-guides/stack_z_offsets_changes.md
- release-content/migration-guides/stop-exposing-minimp3.md
- release-content/migration-guides/stop_storing_system_access.md
- release-content/migration-guides/sync_cell_utils.md
- release-content/migration-guides/system_run_returns_result.md
- release-content/migration-guides/system_set_naming_convention.md
- release-content/migration-guides/taa_non_experimental.md
- release-content/migration-guides/text2d_moved_to_bevy_sprite.md
- release-content/migration-guides/textshadow_is_moved_to_widget_text_module.md
- release-content/migration-guides/texture_format_pixel_size_returns_result.md
- release-content/migration-guides/ui-debug-overlay.md
- release-content/migration-guides/unified_system_state_flag.md
- release-content/migration-guides/view-transformations.md
- release-content/migration-guides/wayland.md
- release-content/migration-guides/wgpu_25.md
- release-content/migration-guides/window_resolution_constructors.md
- release-content/migration-guides/zstd.md

---

## Next Steps

1. **🚫 CRITICAL: Resolve bevy_rapier3d blocker** - Migration cannot proceed until this is resolved
   - Check bevy_rapier3d repository for 0.17.2 support status
   - Monitor for releases or consider alternatives
2. **🔄 Update dependencies** in Cargo.toml:
   - bevy-inspector-egui: 0.33.1 → 0.35.0
   - bevy_panorbit_camera: 0.28.0 → 0.32.0
3. **Start with REQUIRED changes** (must fix to compile with Bevy 0.17.2):
   - Update import paths for bevy_render reorganization (89 occurrences)
   - Convert HDR camera field to component (6 occurrences)
   - Update EventReader to MessageReader (1 occurrence)
4. **Address HIGH priority changes** (deprecated features):
   - Remove manual register_type calls (9 occurrences)
   - Update iter_entities to query pattern (1 occurrence)
5. **Test thoroughly after each category of changes**
6. **Run `cargo check` and `cargo test` frequently**

---

## Reference

- **Migration guides directory:** /Users/natemccoy/rust/bevy-0.17.2/release-content/migration-guides
- **Bevy 0.17.2 release notes:** https://github.com/bevyengine/bevy/releases/tag/v0.17.2
