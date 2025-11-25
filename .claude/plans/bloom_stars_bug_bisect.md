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
- [ ] Save diff: `git diff > /tmp/stars_fix.patch`
- [ ] Stash changes: `git stash`

### Step 2: Verify Bug Exists
- [ ] Build app
- [ ] Run app and confirm stars flash then disappear

### Step 3: Test Groups (in order of probability)

| Order | Status | Group | File | Change |
|-------|--------|-------|------|--------|
| 1 | [ ] | 4 | cameras.rs | Tuple spawn refactor |
| 2 | [ ] | 1 | stars.rs | `Visibility::Visible` |
| 3 | [ ] | 5 | cameras.rs | `Query` → `Single` |
| 4 | [ ] | 2 | stars.rs | Debug logging |
| 5 | [ ] | 3 | cameras.rs | Rename |

### Step 4: Document Result
- [ ] Identify minimal fix
- [ ] Document root cause understanding

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
