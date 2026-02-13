# Plan: Extract Generic PanOrbitCamera Extension Library

## Context

Both nateroids and hana use `bevy_panorbit_camera` for camera control and share several common needs:
- **Camera animation**: Smooth, queued camera movements with easing functions
- **Zoom-to-fit**: Automatically frame objects in the viewport
- **Home camera**: Reset camera to a known position
- **Extension utilities**: Convenience methods for PanOrbitCamera manipulation

Currently, these features exist in nateroids but are coupled to game-specific types (`Boundary`, `GameAction`). We need to extract them into a standalone library (`bevy_panorbit_camera_ext`) that both projects can use.

**Key Design Decision**: Work with `Transform` directly - no traits needed. ZoomToFit accepts any Transform and internally computes the bounding corners. This is ergonomic and Bevy-idiomatic since Transform is ubiquitous, and avoids generic component complexity.

## EXECUTION PROTOCOL

### Instructions

For each step in the implementation sequence:

1. **DESCRIBE**: Present the changes with:
   - Summary of what will change and why
   - Code examples showing before/after
   - List of files to be modified
   - Expected impact on the system

2. **AWAIT APPROVAL**: Stop and wait for user confirmation ("go ahead" or similar)

3. **IMPLEMENT**: Make the changes and stop
   - Use the LSP tool (`findReferences`, `goToDefinition`, `incomingCalls`) to locate all usages before modifying types, functions, or signatures
   - LSP is faster and more accurate than grep/search for Rust codebases

4. **BUILD & VALIDATE**: Execute the build process:
   ```bash
   cargo build && cargo +nightly fmt
   ```

5. **CONFIRM**: Wait for user to confirm the build succeeded

6. **MARK COMPLETE**: Update this document to mark the step as ✅ COMPLETED

7. **PROCEED**: Move to next step only after confirmation

### Execute Implementation

Find the next ⏳ PENDING step in the INTERACTIVE IMPLEMENTATION SEQUENCE below.

For the current step:
1. Follow the Instructions above for executing the step
2. When step is complete, use Edit tool to mark it as ✅ COMPLETED
3. Continue to next PENDING step

If all steps are COMPLETED:
    Display: "✅ Implementation complete! All steps have been executed."

## INTERACTIVE IMPLEMENTATION SEQUENCE

### Step 1: Create Git Branch ⏳ PENDING

**Objective**: Create a dedicated branch for the camera library extraction work

**Changes**:
- Create and checkout a new branch `extract-camera-library`

**Build Commands**:
```bash
git checkout -b extract-camera-library
```

**Status**: ⏳ PENDING
**Change Type**: Safe (branch creation)
**Expected Result**: Working on new branch, ready to begin implementation

---

### Step 2: Create Library Foundation ⏳ PENDING

**Objective**: Set up the new `bevy_panorbit_camera_ext` crate structure

**Changes**:
- Create directory `/Users/natemccoy/rust/bevy_panorbit_camera_ext/`
- Create `Cargo.toml` with dependencies
- Create `src/` directory structure

**Files**:
- `/Users/natemccoy/rust/bevy_panorbit_camera_ext/Cargo.toml` (new)
- `/Users/natemccoy/rust/bevy_panorbit_camera_ext/src/lib.rs` (new, empty for now)

**Build Commands**:
```bash
cd /Users/natemccoy/rust/bevy_panorbit_camera_ext
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: New crate compiles independently

---

### Step 3: Implement Smoothness Module ⏳ PENDING

**Objective**: Add `SmoothnessStash` component with observer-based restore mechanism

**Changes**:
- Create `smoothness.rs` with `SmoothnessStash` component
- Add `restore_smoothness_on_complete` observer
- Handles restoration when CameraMoveList/ZoomToFit/SnapToFit are removed

**Files**:
- `bevy_panorbit_camera_ext/src/smoothness.rs` (new)

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: Library compiles with smoothness module

**Implementation Details**:
```rust
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

/// Component that stores camera smoothness values during animations.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct SmoothnessStash {
    pub zoom: f32,
    pub pan: f32,
    pub orbit: f32,
}

// Observer: Restores smoothness when any animation component is removed
fn restore_smoothness_on_complete(
    trigger: On<Remove, (CameraMoveList, ZoomToFit, SnapToFit)>,
    mut commands: Commands,
    query: Query<(&SmoothnessStash, &mut PanOrbitCamera)>,
) {
    let entity = trigger.entity();

    let Ok((stash, mut camera)) = query.get_mut(entity) else {
        return;
    };

    camera.zoom_smoothness = stash.zoom;
    camera.pan_smoothness = stash.pan;
    camera.orbit_smoothness = stash.orbit;

    commands.entity(entity).remove::<SmoothnessStash>();
}
```

---

### Step 4: Implement Animation Module ⏳ PENDING

**Objective**: Extract `CameraMoveList` from nateroids, remove game-specific dependencies

**Changes**:
- Copy from `nateroids/src/camera/move_queue.rs`
- Remove `CameraConfig` dependency
- Remove smoothness storage (handled by `SmoothnessStash`)
- Add `process_camera_move_list` system

**Files**:
- `bevy_panorbit_camera_ext/src/animation.rs` (new)

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: Library compiles with animation module

---

### Step 5: Implement Extension Trait & Events ⏳ PENDING

**Objective**: Add `PanOrbitCameraExt` trait and EntityEvents for camera control

**Changes**:
- Create `PanOrbitCameraExt` trait with interpolation and home position methods
- Add EntityEvents: `SnapToFit`, `ZoomToFit`, `StartAnimation`
- Add observers for each event
- Add `ZoomToFitConfig` component (auto-added to cameras)

**Files**:
- `bevy_panorbit_camera_ext/src/extension.rs` (new)

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: Library compiles with extension module

**Implementation Details**:
```rust
#[derive(EntityEvent)]
pub struct SnapToFit {
    #[event_target]
    pub camera_entity: Entity,
    pub target: Transform,
}

#[derive(EntityEvent)]
pub struct ZoomToFit {
    #[event_target]
    pub camera_entity: Entity,
    pub target: Transform,
}

#[derive(EntityEvent)]
pub struct StartAnimation {
    #[event_target]
    pub camera_entity: Entity,
    pub moves: VecDeque<CameraMove>,
}
```

---

### Step 6: Implement Zoom-to-Fit System ⏳ PENDING

**Objective**: Add Transform-based bounding calculation and zoom convergence system

**Changes**:
- Add `compute_bounding_corners` function (works with any Transform)
- Extract `ScreenSpaceBounds` from nateroids' `ScreenSpaceBoundary`
- Add `ZoomToFit` component
- Add `zoom_to_fit_convergence_system`
- Add `ZoomConfig` resource

**Files**:
- `bevy_panorbit_camera_ext/src/zoom.rs` (new)

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: Library compiles, nateroids still compiles unchanged

---

### Step 7: Complete Library Public API ⏳ PENDING

**Objective**: Finalize `lib.rs` with prelude and plugin

**Changes**:
- Create `lib.rs` with module declarations
- Create `prelude.rs` with convenient re-exports
- Define `CameraExtPlugin` that registers all observers/systems

**Files**:
- `bevy_panorbit_camera_ext/src/lib.rs` (update)
- `bevy_panorbit_camera_ext/src/prelude.rs` (new)

**Build Commands**:
```bash
cargo build
cd /Users/natemccoy/rust/nateroids
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: Both library and nateroids compile

**Implementation Details**:
```rust
// lib.rs
pub mod animation;
pub mod extension;
pub mod prelude;
pub mod smoothness;
pub mod zoom;

pub use prelude::*;

// prelude.rs
pub use crate::animation::{CameraMove, CameraMoveList};
pub use crate::extension::{
    PanOrbitCameraExt, SnapToFit, StartAnimation, ZoomToFit, ZoomToFitConfig,
};
pub use crate::smoothness::SmoothnessStash;
pub use crate::zoom::{Edge, ScreenSpaceBounds, ZoomConfig};

// Single plugin for the entire library
pub struct CameraExtPlugin;
impl Plugin for CameraExtPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(restore_smoothness_on_complete)
            .add_observer(auto_add_zoom_config)
            .add_observer(on_snap_to_fit)
            .add_observer(on_zoom_to_fit)
            .add_observer(on_start_animation)
            .add_systems(Update, (
                process_camera_move_list,
                zoom_to_fit_convergence_system,
            ))
            .init_resource::<ZoomConfig>();
    }
}
```

---

### Step 8: Add Library Dependency to Nateroids ⏳ PENDING

**Objective**: Make library available to nateroids

**Changes**:
- Add `bevy_panorbit_camera_ext` to nateroids' workspace dependencies

**Files**:
- `nateroids/Cargo.toml`

**Build Commands**:
```bash
cd /Users/natemccoy/rust/nateroids
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Additive (safe)
**Expected Result**: Nateroids compiles with library available

**Implementation**:
Add to `nateroids/Cargo.toml`:
```toml
[dependencies]
bevy_panorbit_camera_ext = { path = "../bevy_panorbit_camera_ext" }
```

---

### Step 9: Migrate Splash Animation ⏳ PENDING

**Objective**: Update splash screen to use `StartAnimation` EntityEvent

**Changes**:
- Update `start_splash_camera_animation` to trigger `StartAnimation` event
- Remove function signature change (no longer needs `&mut PanOrbitCamera`)
- Update imports

**Files**:
- `nateroids/src/splash.rs`

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Breaking (atomic group)
**Expected Result**: Compiles successfully

**Before**:
```rust
commands.entity(entity).insert(CameraMoveList::new(moves.into()));
```

**After**:
```rust
use bevy_panorbit_camera_ext::prelude::StartAnimation;

commands.trigger(StartAnimation::new(camera_entity, moves));
```

---

### Step 10: Migrate Extension Trait Usage ⏳ PENDING

**Objective**: Update `enable_interpolation` calls to use explicit parameters

**Changes**:
- Find all calls to `enable_interpolation(&camera_config)`
- Replace with explicit zoom/pan/orbit parameters

**Files**:
- Various files in `nateroids/src/camera/`

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Breaking (atomic group)
**Expected Result**: Compiles successfully

**Before**:
```rust
pan_orbit.enable_interpolation(&camera_config);
```

**After**:
```rust
pan_orbit.enable_interpolation(
    camera_config.zoom_smoothness,
    camera_config.pan_smoothness,
    camera_config.orbit_smoothness,
);
```

---

### Step 11: Migrate Zoom-to-Fit ⏳ PENDING

**Objective**: Replace `start_zoom_to_fit` with `ZoomToFit` EntityEvent

**Changes**:
- Update `start_zoom_to_fit` function to trigger event
- Remove convergence system (now in library)

**Files**:
- `nateroids/src/camera/zoom.rs`

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Breaking (atomic group)
**Expected Result**: Compiles successfully

**Implementation**:
```rust
use bevy_panorbit_camera_ext::ZoomToFit;

fn start_zoom_to_fit(
    mut commands: Commands,
    boundary: Res<Boundary>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let camera_entity = camera_query.single();

    commands.trigger(ZoomToFit {
        camera_entity,
        target: boundary.transform,
    });
}
```

---

### Step 12: Migrate Home Camera ⏳ PENDING

**Objective**: Replace `home_camera()` with `SnapToFit` event

**Changes**:
- Update `home_camera` function to trigger `SnapToFit` event
- Delete `calculate_home_radius()` (library handles this internally)

**Files**:
- `nateroids/src/camera/cameras.rs`

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Breaking (atomic group)
**Expected Result**: Compiles successfully

**Implementation**:
```rust
use bevy_panorbit_camera_ext::SnapToFit;

fn home_camera(
    mut commands: Commands,
    boundary: Res<Boundary>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let camera_entity = camera_query.single();

    commands.trigger(SnapToFit {
        camera_entity,
        target: boundary.transform,
    });
}
```

---

### Step 13: Update Plugin Registration ⏳ PENDING

**Objective**: Add library plugin, remove old plugin registrations

**Changes**:
- Add `CameraExtPlugin` from library
- Remove references to moved modules
- Update imports

**Files**:
- `nateroids/src/camera/mod.rs`

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Breaking
**Expected Result**: Compiles successfully

**Before**:
```rust
app.add_plugins(MoveQueuePlugin)
   .add_plugins(ZoomPlugin)
```

**After**:
```rust
use bevy_panorbit_camera_ext::CameraExtPlugin;

app.add_plugins(CameraExtPlugin)
```

---

### Step 14: Cleanup Nateroids - Remove Old Code ⏳ PENDING

**Objective**: Delete old code that's been moved to the library

**Changes**:
- DELETE `src/camera/move_queue.rs`
- DELETE `src/camera/pan_orbit_ext.rs`
- DELETE from `src/camera/zoom.rs`: `update_zoom_to_fit()`, `ZoomToFit` component
- DELETE from `src/camera/cameras.rs`: `ScreenSpaceBoundary`, `reset_camera_after_moves()`, `Edge` enum
- DELETE from `src/camera/config.rs`: `ZoomConfig`
- UPDATE `src/camera/mod.rs`: Remove deleted module references

**Files**:
- `nateroids/src/camera/move_queue.rs` (delete)
- `nateroids/src/camera/pan_orbit_ext.rs` (delete)
- `nateroids/src/camera/zoom.rs` (partial cleanup)
- `nateroids/src/camera/cameras.rs` (partial cleanup)
- `nateroids/src/camera/config.rs` (partial cleanup)
- `nateroids/src/camera/mod.rs` (update imports)

**Build Commands**:
```bash
cargo build
```

**Status**: ⏳ PENDING
**Change Type**: Removal (safe - code already unused)
**Expected Result**: Compiles successfully

---

### Step 15: Final Validation ⏳ PENDING

**Objective**: Verify everything works correctly

**Validation Steps**:
1. Run all tests: `cargo nextest run`
2. Launch nateroids and test splash animation
3. Test zoom-to-fit action (verify camera frames playfield)
4. Test home camera action
5. Verify camera animations work smoothly

**Build Commands**:
```bash
cargo nextest run
cargo run
```

**Status**: ⏳ PENDING
**Expected Result**: All tests pass, game functions correctly

---

## Library Structure

Create new crate at `/Users/natemccoy/rust/bevy_panorbit_camera_ext/`:

```
bevy_panorbit_camera_ext/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Public API
│   ├── smoothness.rs             # SmoothnessStash, automatic restore
│   ├── animation.rs              # CameraMoveList, move queue system
│   ├── extension.rs              # PanOrbitCameraExt trait, EntityEvents, observers
│   ├── zoom.rs                   # Zoom-to-fit convergence
│   └── prelude.rs                # Convenient re-exports
```

## Dependencies

Library `Cargo.toml`:
```toml
[package]
name = "bevy_panorbit_camera_ext"
version = "0.1.0"
edition = "2024"

[dependencies]
bevy = "0.18.0"
bevy_panorbit_camera = "0.34.0"
```

No game-specific dependencies - clean separation achieved.

## Success Criteria

- ✅ Library compiles independently
- ✅ Nateroids compiles with library dependency
- ✅ All nateroids tests pass
- ✅ Splash screen animation works
- ✅ Zoom-to-fit action works
- ✅ Home camera action works
- ✅ No Boundary or GameAction references in library
- ✅ Hana can use library for camera control (future)
