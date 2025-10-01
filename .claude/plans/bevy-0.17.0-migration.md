# Bevy 0.17.0 Migration Plan for Nateroids

**Status**: BLOCKED - Awaiting dependency updates
**Date Created**: 2025-09-30
**Target**: Migrate from Bevy 0.16.1 to Bevy 0.17.0

## Executive Summary

The nateroids project cannot migrate to Bevy 0.17.0 until critical dependencies are updated:
- `bevy_rapier3d` (currently 0.31.0 supports only Bevy 0.16)
- `bevy_panorbit_camera` (currently 0.28.0 supports only Bevy 0.16)

Once these dependencies release 0.17-compatible versions, the migration will be straightforward with changes to ~7 files.

---

## Dependency Status

### 🚫 BLOCKING Dependencies (Not Yet Compatible)

| Dependency | Current Version | Status | Notes |
|------------|----------------|--------|-------|
| `bevy_rapier3d` | 0.31.0 | ❌ Bevy 0.16 only | Critical - physics engine |
| `bevy_panorbit_camera` | 0.28.0 | ❌ Bevy 0.16 only | Critical - camera system |

### ✅ READY Dependencies (Already Compatible)

| Dependency | Target Version | Status |
|------------|----------------|--------|
| `leafwing-input-manager` | 0.17.x | ✅ Compatible |
| `bevy-inspector-egui` | 0.33.x | ✅ Compatible |

### ⚠️ UNKNOWN Dependencies

| Dependency | Notes |
|------------|-------|
| `rand` | External to Bevy ecosystem - should be fine |
| `strum` | External to Bevy ecosystem - should be fine |

---

## Breaking Changes from Bevy 0.16 → 0.17

### 1. Event System Overhaul

**Change**: Separation of "Events" and "Messages"
- Traditional events are now called "messages"
- `EventWriter<T>` → `MessageWriter<T>`
- `EventReader<T>` → `MessageReader<T>`
- `Events<T>` → `Messages<T>`

**Impact on Nateroids**:
- `src/actor/collision_detection.rs:24` uses `EventReader<CollisionEvent>`
- **Note**: `CollisionEvent` comes from `bevy_rapier3d`, so the change depends on how that crate adapts to 0.17

### 2. Rendering Module Reorganization

**Change**: Rendering types moved to specialized crates
- Camera types → `bevy_camera`
- `bevy::render::view::Layer` → `bevy_camera::Layer`
- `bevy::render::view::RenderLayers` → `bevy_camera::RenderLayers`

**Impact on Nateroids**:
- `src/camera/mod.rs:9` - imports `Layer`
- `src/camera/cameras.rs:19` - imports `RenderLayers`
- `src/camera/stars.rs:4` - imports `RenderLayers`
- `src/actor/actor_spawner.rs:29` - imports `RenderLayers`
- `src/splash.rs:7` - imports `RenderLayers`

### 3. Entity Iteration API Changes

**Change**: `World::iter_entities()` deprecated
- `world.iter_entities()` → `world.entities().iter()`

**Impact on Nateroids**:
- `src/actor/aabb.rs:64` - `scene.world.iter_entities()`

### 4. Observer System Changes

**Change**: `Trigger` renamed to `On`
- Not currently used in nateroids

### 5. Reflection Improvements

**Change**: Automatic type registration for `#[derive(Reflect)]`
- Types with `#[derive(Reflect)]` no longer need manual `app.register_type::<Type>()`
- May allow cleanup of existing registrations

**Impact on Nateroids**:
- Potential cleanup opportunity (not breaking)

---

## Migration Steps

### Phase 1: Pre-Migration (Current)

- [x] Analyze codebase for breaking changes
- [x] Document required changes
- [ ] Monitor dependency repositories for 0.17 support:
  - Watch: https://github.com/dimforge/bevy_rapier
  - Watch: https://github.com/Plonq/bevy_panorbit_camera
- [ ] Test current functionality before migration

### Phase 2: Dependency Updates (BLOCKED)

- [ ] Wait for `bevy_rapier3d` 0.17-compatible release
- [ ] Wait for `bevy_panorbit_camera` 0.17-compatible release
- [ ] Check CHANGELOG for both dependencies for any breaking changes

### Phase 3: Update Cargo.toml

```toml
[dependencies]
bevy = { version = "0.17.0", features = [
  "tonemapping_luts",
  "bevy_dev_tools",
  "bevy_remote",
] }
bevy_rapier3d = { version = "0.XX.0", features = [  # Update when available
  "simd-stable",
  "debug-render-3d",
  "parallel",
] }
bevy_panorbit_camera = { version = "0.XX.0", features = ["bevy_egui"] }  # Update when available
leafwing-input-manager = "0.17"  # Already compatible
bevy-inspector-egui = "0.33"  # Update to latest
```

### Phase 4: Code Changes

#### File: `src/actor/collision_detection.rs`

**Current** (line 24):
```rust
mut collision_events: EventReader<CollisionEvent>,
```

**Action**:
- Check `bevy_rapier3d` migration guide when 0.17 version releases
- May need to change to `MessageReader<CollisionEvent>` or similar
- Update event reading method if needed

---

#### File: `src/actor/aabb.rs`

**Current** (line 64):
```rust
for entity in scene.world.iter_entities() {
```

**New** (line 64):
```rust
for entity in scene.world.entities().iter() {
```

---

#### File: `src/camera/mod.rs`

**Current** (line 7-9):
```rust
use bevy::{
    prelude::*,
    render::view::Layer,
};
```

**New** (line 7-10):
```rust
use bevy::{
    prelude::*,
};
use bevy_camera::Layer;
```

---

#### File: `src/camera/cameras.rs`

**Current** (line 13-19):
```rust
use bevy::{
    core_pipeline::{
        bloom::Bloom,
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::view::RenderLayers,
};
```

**New**:
```rust
use bevy::{
    core_pipeline::{
        bloom::Bloom,
        tonemapping::Tonemapping,
    },
    prelude::*,
};
use bevy_camera::RenderLayers;
```

---

#### File: `src/camera/stars.rs`

**Current** (line 1-4):
```rust
use crate::camera::RenderLayer;
use bevy::{
    prelude::*,
    render::view::RenderLayers,
};
```

**New**:
```rust
use crate::camera::RenderLayer;
use bevy::prelude::*;
use bevy_camera::RenderLayers;
```

---

#### File: `src/actor/actor_spawner.rs`

**Current** (line 29):
```rust
    render::view::RenderLayers,
```

**New** (line 29):
```rust
use bevy_camera::RenderLayers;
```

(And remove from the bevy:: import block)

---

#### File: `src/splash.rs`

**Current** (line 7):
```rust
    render::view::RenderLayers,
```

**New**:
```rust
use bevy_camera::RenderLayers;
```

(And remove from the bevy:: import block)

---

### Phase 5: Build and Test

```bash
# Clean build
cargo clean

# Build with new dependencies
cargo build

# Format code
cargo +nightly fmt

# Run tests
cargo nextest run

# Test run the game
cargo run
```

### Phase 6: Validation

- [ ] Physics collisions work correctly
- [ ] Camera controls function properly
- [ ] Rendering layers display correctly
- [ ] Stars with bloom render correctly
- [ ] Input handling works
- [ ] Inspector UI functions
- [ ] No deprecation warnings
- [ ] All tests pass

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| bevy_rapier3d breaking changes beyond event system | HIGH | Thoroughly read migration guide when released |
| bevy_panorbit_camera breaking changes | MEDIUM | Review changelog and test camera controls |
| Rendering layer behavior changes | LOW | Visual testing of all render layers |
| Performance regressions | LOW | Profile before/after migration |

---

## Rollback Plan

If migration fails:
1. `git stash` or commit to feature branch
2. Revert `Cargo.toml` to Bevy 0.16.1
3. `cargo clean && cargo build`
4. Document blocking issues
5. Wait for ecosystem maturity

---

## Resources

- [Bevy 0.17 Release Notes](https://bevy.org/news/bevy-0-17/)
- [Bevy 0.16 to 0.17 Migration Guide](https://bevy.org/learn/migration-guides/0-16-to-0-17/)
- [bevy_rapier GitHub](https://github.com/dimforge/bevy_rapier)
- [bevy_panorbit_camera GitHub](https://github.com/Plonq/bevy_panorbit_camera)
- [Local Bevy 0.17 source](../bevy-0.17.0)

---

## Estimated Effort

- **Preparation**: 1 hour (monitoring dependencies)
- **Code Changes**: 2-3 hours (once dependencies ready)
- **Testing**: 2-3 hours
- **Total**: ~6 hours (excluding wait time for dependencies)

---

## Timeline

- **Now**: Document and wait for dependencies
- **When bevy_rapier3d releases**: Review their migration guide
- **When bevy_panorbit_camera releases**: Begin migration
- **Target completion**: Within 1 week of all dependencies being ready

---

## Notes

- The project is using `bevy_rapier3d` from a git branch (`mnmaita/bevy-0.16`), which may need to switch back to crates.io or to a new git branch for 0.17 support
- Consider creating a `migration/bevy-0.17` branch for the work
- Test thoroughly before merging to main
- The collision detection system is critical - extra testing needed there
- Custom `RenderLayer` enum should continue working, just imports change
