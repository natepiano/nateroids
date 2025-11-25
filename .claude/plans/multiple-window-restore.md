# Multi-Window Restore Feature

## Overview

Extend `window_restore.rs` to support restoring positions for multiple windows, not just the primary window.

## Design

### Component

```rust
/// Marker component for windows that should have their position/size persisted
#[derive(Component)]
pub struct RestoreWindow(pub String);

impl RestoreWindow {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}
```

**Usage:**
```rust
commands.spawn((
    Window { title: "Inspector".into(), ..default() },
    RestoreWindow::new("inspector"),
));
```

### Storage Format

New `WindowsState` struct with separate primary and named windows:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowsState {
    pub primary: Option<WindowState>,
    pub windows: HashMap<String, WindowState>,
}
```

**RON format:**
```ron
(
    primary: (
        position: Some((100, 50)),
        width: 1280.0,
        height: 720.0,
        mode: Windowed,
        monitor_name: Some("Built-in Retina Display"),
        monitor_position: Some((0, 0)),
        monitor_index: Some(0),
    ),
    windows: {
        "inspector": (
            position: Some((50, 100)),
            width: 300.0,
            height: 400.0,
            mode: Windowed,
            monitor_name: Some("Built-in Retina Display"),
            monitor_position: Some((0, 0)),
            monitor_index: Some(0),
        ),
    },
)
```

### Flow

1. **Primary window at startup**:
   - App calls `window_restore::primary_window()` when creating the primary window
   - Function loads RON, reads `primary` field, returns configured `Window`
   - Primary window does NOT need `RestoreWindow` component

2. **Named windows spawned later**:
   - App spawns window with `RestoreWindow::new("name")` component
   - Observer fires on `OnAdd<RestoreWindow>`
   - Observer looks up name in `windows` HashMap
   - If found, applies saved position/size to the `Window` component

3. **Saving**:
   - Existing `save_on_window_events` system handles move/resize events
   - For primary window: save to `primary` field
   - For windows with `RestoreWindow`: save to `windows[name]`
   - Iterate all windows, update their entries, write RON

### Edge Cases

- **Duplicate names**: User's responsibility. Last save wins. Optionally warn on observer.
- **Primary window with RestoreWindow**: Ignore the component (primary is always handled separately)
- **Window closed**: Keep entry in RON (allows restore if window is recreated next session)

## Implementation Steps

1. Add `RestoreWindow` component with `new(impl Into<String>)` constructor
2. Add `WindowsState` struct with `primary` and `windows` HashMap
3. Update `load_window_state()` to `load_windows_state()` returning `WindowsState`
4. Update `primary_window()` to use new format (read `state.primary`)
5. Add observer for `OnAdd<RestoreWindow>` that applies saved state from `windows` map
6. Update `save_on_window_events` to:
   - Save primary window to `state.primary`
   - Iterate windows with `RestoreWindow`, save to `state.windows[name]`
7. Build and format

## Files to Modify

- `src/window_restore.rs` - all changes contained here
