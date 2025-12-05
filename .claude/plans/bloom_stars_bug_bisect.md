# Followup when you can:
Probably some fun render target reuse issues. Please run it through renderdoc and see what passes write to what
render targets

i asked i claude and claude said: 

⏺ RenderDoc is a free GPU debugger that lets you capture a frame and inspect every render pass - what it draws,
  what textures it reads/writes.

  Render target reuse - Bevy reuses GPU textures between passes to save memory. The dev suspects the bloom pass's
  render target is being incorrectly overwritten or shared with another pass when the archetype timing changes.

  What they're asking: Capture the buggy frame in RenderDoc, then look at the render passes to see:
  - Which pass should be writing the bloom objects
  - Whether that pass's output texture is getting clobbered by another pass

  To use RenderDoc:
  1. Install from https://renderdoc.org
  2. Launch your example through RenderDoc (File → Launch Application)
  3. Press F12 or PrintScreen to capture a frame
  4. Inspect the pass list to see what's writing where

  Want me to help interpret results once you capture it?




# Plan: Identify the Fix for Bloom/Stars Rendering Bug

## Goal
Systematically replay each change from the current uncommitted diff to identify which specific change fixes the stars rendering bug.

## Background
- Stars flash on frame 1 then disappear when bloom is enabled
- The bug is sensitive to archetype changes - any component add/remove can toggle it
- Current uncommitted changes appear to have fixed the bug
- When changes are stashed, the bug is reproducible

## Progress Checklist

### Step 1: Save Current State
- [x] Save diff: `git diff > /tmp/stars_fix.patch` (227 lines)
- [x] Stash changes: `git stash`

### Step 2: Verify Bug Exists
- [x] Build app
- [x] Run app and confirm stars flash then disappear - **REPRODUCED**

### Step 3: Test Groups (least likely → most likely)

| Order | Status | Group | File | Change |
|-------|--------|-------|------|--------|
| 1 | [x] | 3 | cameras.rs | Rename `StarsCamera` → `StarCamera` | **NOT THE FIX** |
| 2 | [x] | 2 | stars.rs | Debug logging additions | **NOT THE FIX** |
| 3 | [x] | 5 | cameras.rs | `Query` → `Single` | **NOT THE FIX** |
| 4 | [x] | 1 | stars.rs | `Visibility::Visible` | **NOT THE FIX** |
| 5 | [x] | 4 | cameras.rs | Tuple spawn refactor | **THE FIX** ✓ |

### Step 4: Document Result
- [x] Identify minimal fix
- [x] Document root cause understanding

---

## Root Cause Analysis

**The Fix**: Converting `spawn_panorbit_camera` from sequential `.insert()` calls to a single tuple spawn.

**Why it works**: Sequential `.insert()` calls create intermediate archetypes as each component is added. The bloom post-processing system's render world extraction appears sensitive to archetype creation order/timing. Spawning all components atomically in a tuple creates the final archetype in one step, avoiding the intermediate states that triggered the bug.

**Affected code pattern**:
```rust
// BUG: Sequential inserts create intermediate archetypes
commands.spawn(A).insert(B).insert(C);

// FIX: Tuple spawn creates archetype atomically
commands.spawn((A, B, C));
```

This is likely a Bevy rendering pipeline edge case where the bloom `ExtractComponent` system runs during an intermediate archetype state.

---

## Group Details

### Group 1: `stars.rs` - Visibility Change
- Added `Visibility::Visible` to star spawn
- **Hypothesis**: Forces stars to be explicitly visible, affecting material extraction timing

### Group 2: `stars.rs` - Debug Logging
- Added imports: `TypeId`, `VisibleEntities`, `FrameCount`, `Mesh3d`
- Enhanced `debug_stars` with frame count and VisibleEntities query

### Group 3: `cameras.rs` - StarCamera Rename
- `StarsCamera` → `StarCamera`

### Group 4: `cameras.rs` - Tuple Spawn Refactor
- Changed from multiple `.insert()` calls to single tuple spawn for panorbit camera
- **Hypothesis**: HIGH LIKELIHOOD - Changes archetype creation pattern

### Group 5: `cameras.rs` - Single Query Change
- Changed `Query<Entity, With<StarCamera>>` to `Single<Entity, With<StarCamera>>`

### Group 6: `window_restore.rs` - WindowSettling Insert
- Uncommented `WindowSettling::Created` insert
- **Note**: User confirmed this can be toggled without affecting the fix - SKIP
