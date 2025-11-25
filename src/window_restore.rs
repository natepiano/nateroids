//! Window state persistence - saves and restores window position and size
// These casts are intentional: monitor dimensions fit in i32, position precision loss is acceptable
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]

use std::fs;
use std::path::PathBuf;

use bevy::ecs::message::MessageReader;
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use bevy::window::Monitor;
use bevy::window::MonitorSelection;
use bevy::window::PrimaryWindow;
use bevy::window::VideoModeSelection;
use bevy::window::WindowCreated;
use bevy::window::WindowMode;
use bevy::window::WindowMoved;
use bevy::window::WindowPosition;
use bevy::window::WindowResized;
use bevy::winit::WINIT_WINDOWS;
use dirs::config_dir;
use serde::Deserialize;
use serde::Serialize;
use winit::window::Fullscreen;

/// The filename for the window state configuration
const WINDOW_STATE_FILENAME: &str = "windows.ron";
/// The filename for monitor configuration
const MONITORS_FILENAME: &str = "monitors.ron";

/// Plugin that handles window state persistence (saving on move/resize/mode change)
pub struct WindowRestorePlugin;

impl Plugin for WindowRestorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindowStateTracker>().add_systems(
            PostStartup,
            (log_actual_window_position, save_monitors_on_startup),
        );

        app.add_systems(Update, on_window_created);
        app.add_systems(Update, on_window_moved);
        app.add_systems(Update, log_window_resized);
        app.add_systems(Last, save_on_window_events);
    }
}

/// Get the application name from the executable for config directory naming
fn get_app_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "bevy_app".to_string())
}

/// Create the primary `Window` with restored position/size/mode applied (if available)
/// Set the title and other app-specific settings on the returned window
pub fn primary_window() -> Window {
    let mut window = Window::default();
    if let Some(state) = load_window_state() {
        apply_window_state(&mut window, &state);
    }
    window
}

/// Marker component: window is settling after creation (OS positioning it)
#[derive(Component)]
enum WindowSettling {
    Created,
    Moved,
}

/// Resource to track last saved window state
#[derive(Resource, Default)]
struct WindowStateTracker {
    position: Option<IVec2>,
    size:     Option<(f32, f32)>,
    mode:     Option<WindowMode>,
}

/// Insert `WindowSettling` component on primary window when created
fn on_window_created(
    mut reader: MessageReader<WindowCreated>,
    mut commands: Commands,
    primary: Query<Entity, With<PrimaryWindow>>,
) {
    for event in reader.read() {
        if primary.get(event.window).is_ok() {
            info!(
                "[WindowCreated] {:?} -> inserting WindowSettling::Created",
                event.window
            );
            // commands
            //     .entity(event.window)
            //     .insert(WindowSettling::Created);
        }
    }
}

/// Track `WindowMoved` and transition `WindowSettling` to `Moved` state
fn on_window_moved(
    mut reader: MessageReader<WindowMoved>,
    mut commands: Commands,
    primary: Query<Entity, (With<PrimaryWindow>, With<WindowSettling>)>,
) {
    for event in reader.read() {
        info!("[WindowMoved] {:?} to {:?}", event.window, event.position);
        if primary.get(event.window).is_ok() {
            commands.entity(event.window).insert(WindowSettling::Moved);
        }
    }
}

/// Test logging: track when `WindowResized` fires
fn log_window_resized(mut reader: MessageReader<WindowResized>) {
    for event in reader.read() {
        info!(
            "[WindowResized] {:?} to {}x{}",
            event.window, event.width, event.height
        );
    }
}

/// Save monitors state at startup (only if changed)
fn save_monitors_on_startup(monitors: Query<&Monitor>) {
    // Load existing monitors state and compare
    let current_monitors: Vec<MonitorInfo> = monitors
        .iter()
        .enumerate()
        .map(|(index, m)| MonitorInfo {
            name: m.name.clone(),
            index,
            physical_position: m.physical_position,
            physical_width: m.physical_width,
            physical_height: m.physical_height,
            scale_factor: m.scale_factor as f32,
        })
        .collect();

    let should_save = load_monitors_state().is_none_or(|saved| saved.monitors != current_monitors);

    if should_save {
        save_monitors_state(&monitors);
    }
}

/// Log actual window position after it loads
fn log_actual_window_position(window_query: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_query.single() else {
        warn!("Failed to get primary window for position logging");
        return;
    };

    debug!(
        "[PostStartup] pos={:?} size={}x{} physical={}x{} scale={}",
        window.position,
        window.width(),
        window.height(),
        window.physical_width(),
        window.physical_height(),
        window.scale_factor()
    );
}

/// Serializable window state that persists between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    /// Window position relative to its monitor in logical pixels (None means automatic placement)
    pub position:         Option<IVec2>,
    /// Window width in logical pixels
    pub width:            f32,
    /// Window height in logical pixels
    pub height:           f32,
    /// Window mode (windowed, fullscreen, etc.)
    pub mode:             WindowModeState,
    /// Monitor name (for restoring to the same monitor)
    pub monitor_name:     Option<String>,
    /// Monitor position in absolute screen space in logical pixels
    pub monitor_position: Option<IVec2>,
    /// Monitor index (for fullscreen mode selection)
    pub monitor_index:    Option<usize>,
}

/// Saved information about a single monitor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitorInfo {
    /// Monitor name from the OS
    pub name:              Option<String>,
    /// Monitor index in Bevy's enumeration
    pub index:             usize,
    /// Physical position in screen space
    pub physical_position: IVec2,
    /// Physical width in pixels
    pub physical_width:    u32,
    /// Physical height in pixels
    pub physical_height:   u32,
    /// Scale factor (e.g., 2.0 for Retina)
    pub scale_factor:      f32,
}

/// Saved state of all monitors
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorsState {
    /// All known monitors
    pub monitors: Vec<MonitorInfo>,
}

/// Serializable version of Bevy's `WindowMode`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowModeState {
    /// Normal windowed mode
    Windowed,
    /// Borderless fullscreen
    BorderlessFullscreen,
    /// True fullscreen
    Fullscreen,
}

impl From<WindowMode> for WindowModeState {
    fn from(mode: WindowMode) -> Self {
        match mode {
            WindowMode::Windowed => Self::Windowed,
            WindowMode::BorderlessFullscreen(_) => Self::BorderlessFullscreen,
            WindowMode::Fullscreen(_, _) => Self::Fullscreen,
        }
    }
}

impl WindowModeState {
    /// Convert to Bevy's `WindowMode` with optional monitor selection
    pub fn to_window_mode(&self, monitor_index: Option<usize>) -> WindowMode {
        let monitor_selection =
            monitor_index.map_or(MonitorSelection::Current, MonitorSelection::Index);

        match self {
            Self::Windowed => WindowMode::Windowed,
            Self::BorderlessFullscreen => WindowMode::BorderlessFullscreen(monitor_selection),
            Self::Fullscreen => {
                WindowMode::Fullscreen(monitor_selection, VideoModeSelection::Current)
            },
        }
    }
}

/// Get the path to the window state file
fn get_window_state_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join(get_app_name()).join(WINDOW_STATE_FILENAME))
}

/// Get the path to the monitors file
fn get_monitors_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join(get_app_name()).join(MONITORS_FILENAME))
}

/// Load monitors state from disk
pub fn load_monitors_state() -> Option<MonitorsState> {
    let path = get_monitors_path()?;
    let contents = fs::read_to_string(&path).ok()?;
    ron::from_str(&contents).ok()
}

/// Save monitors state to disk
fn save_monitors_state(monitors: &Query<&Monitor>) {
    let monitors_state = MonitorsState {
        monitors: monitors
            .iter()
            .enumerate()
            .map(|(index, m)| MonitorInfo {
                name: m.name.clone(),
                index,
                physical_position: m.physical_position,
                physical_width: m.physical_width,
                physical_height: m.physical_height,
                scale_factor: m.scale_factor as f32,
            })
            .collect(),
    };

    let Some(path) = get_monitors_path() else {
        warn!("Failed to get monitors path");
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        warn!("Failed to create config directory: {e}");
        return;
    }

    match ron::ser::to_string_pretty(&monitors_state, ron::ser::PrettyConfig::default()) {
        Ok(contents) => {
            if let Err(e) = fs::write(&path, contents) {
                warn!("Failed to write monitors state: {e}");
            } else {
                info!("Monitors state saved to {}", path.display());
            }
        },
        Err(e) => {
            warn!("Failed to serialize monitors state: {e}");
        },
    }
}

/// Load window state from disk, returning `None` if file doesn't exist or is invalid
pub fn load_window_state() -> Option<WindowState> {
    let path = get_window_state_path()?;
    let contents = fs::read_to_string(&path).ok()?;
    let state: WindowState = ron::from_str(&contents).ok()?;
    info!(
        "[load] {path:?} -> pos={:?} size={}x{}",
        state.position, state.width, state.height
    );
    Some(state)
}

/// Find which monitor contains the given window position and return monitor info with index
fn find_monitor_for_position_with_index(
    position: IVec2,
    monitors: &Query<&Monitor>,
) -> Option<(usize, String, IVec2)> {
    for (index, monitor) in monitors.iter().enumerate() {
        let monitor_pos = monitor.physical_position;
        let monitor_size = IVec2::new(
            monitor.physical_width as i32,
            monitor.physical_height as i32,
        );

        // Check if position is within this monitor's bounds
        if position.x >= monitor_pos.x
            && position.x < monitor_pos.x + monitor_size.x
            && position.y >= monitor_pos.y
            && position.y < monitor_pos.y + monitor_size.y
        {
            return monitor.name.clone().map(|name| (index, name, monitor_pos));
        }
    }
    None
}

/// Apply loaded window state to a `Window` configuration
/// Position is relative to monitor, we calculate absolute position if monitor info is available
/// NOTE: State stores logical pixels, which is what `WindowPosition::At()` expects during window
/// creation
pub fn apply_window_state(window: &mut Window, state: &WindowState) {
    // Calculate absolute window position from relative position + monitor position
    if let (Some(relative_pos), Some(monitor_pos)) = (state.position, state.monitor_position) {
        let absolute_pos = relative_pos + monitor_pos;
        window.position = WindowPosition::At(absolute_pos);
        info!("[apply] pos={relative_pos}+{monitor_pos}={absolute_pos}");
    } else if let Some(relative_pos) = state.position {
        window.position = WindowPosition::At(relative_pos);
        info!("[apply] pos={relative_pos}");
    }

    window.resolution.set(state.width, state.height);
    window.mode = state.mode.to_window_mode(state.monitor_index);
    info!(
        "[apply] size={}x{} mode={:?} monitor={:?}",
        state.width, state.height, state.mode, state.monitor_index
    );
}

/// Detect effective window mode by querying winit's actual fullscreen state
/// This is deterministic and works cross-platform, including macOS green button fullscreen
/// Must be called from main thread (system should use `NonSendMarker`)
fn detect_effective_mode(
    window_entity: Entity,
    window: &Window,
    monitors: &Query<&Monitor>,
) -> WindowModeState {
    WINIT_WINDOWS.with_borrow(|winit_windows| {
        let Some(winit_id) = winit_windows.entity_to_winit.get(&window_entity) else {
            warn!("[detect_mode] No winit window for {window_entity:?}");
            return WindowModeState::Windowed;
        };

        let Some(winit_window) = winit_windows.windows.get(winit_id) else {
            warn!("[detect_mode] No winit window for ID {winit_id:?}");
            return WindowModeState::Windowed;
        };

        match winit_window.fullscreen() {
            Some(Fullscreen::Exclusive(_)) => WindowModeState::Fullscreen,
            Some(Fullscreen::Borderless(_)) => {
                // Validate: window must fill to bottom of monitor for true fullscreen
                if let WindowPosition::At(pos) = window.position {
                    let window_bottom = pos.y + window.physical_height() as i32;
                    for monitor in monitors.iter() {
                        let mon_x = monitor.physical_position.x;
                        let mon_width = monitor.physical_width as i32;
                        if pos.x >= mon_x && pos.x < mon_x + mon_width {
                            let monitor_bottom =
                                monitor.physical_position.y + monitor.physical_height as i32;
                            if window_bottom != monitor_bottom {
                                return WindowModeState::Windowed;
                            }
                            break;
                        }
                    }
                }
                WindowModeState::BorderlessFullscreen
            },
            None => WindowModeState::Windowed,
        }
    })
}

/// Shared function to save window state
fn save_window_state(window: &Window, window_entity: Entity, monitors: &Query<&Monitor>) {
    let scale_factor = window.scale_factor();

    // Convert physical position to logical, calculate relative to monitor
    // NOTE: window.position contains PHYSICAL pixels at runtime, but we save LOGICAL pixels
    let (monitor_name, monitor_position, monitor_index, relative_position) =
        if let WindowPosition::At(physical_pos) = window.position {
            let logical_pos = IVec2::new(
                (physical_pos.x as f32 / scale_factor) as i32,
                (physical_pos.y as f32 / scale_factor) as i32,
            );
            if let Some((idx, name, mon_pos_physical)) =
                find_monitor_for_position_with_index(physical_pos, monitors)
            {
                let mon_pos_logical = IVec2::new(
                    (mon_pos_physical.x as f32 / scale_factor) as i32,
                    (mon_pos_physical.y as f32 / scale_factor) as i32,
                );
                let rel_pos_logical = logical_pos - mon_pos_logical;
                (
                    Some(name),
                    Some(mon_pos_logical),
                    Some(idx),
                    Some(rel_pos_logical),
                )
            } else {
                (None, None, None, Some(logical_pos))
            }
        } else {
            (None, None, None, None)
        };

    let effective_mode = detect_effective_mode(window_entity, window, monitors);

    let state = WindowState {
        position: relative_position,
        width: window.width(),
        height: window.height(),
        mode: effective_mode,
        monitor_name,
        monitor_position,
        monitor_index,
    };

    let Some(path) = get_window_state_path() else {
        warn!("[save] Failed to get config directory path");
        return;
    };

    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        warn!("[save] Failed to create config directory: {e}");
        return;
    }

    match ron::ser::to_string_pretty(&state, ron::ser::PrettyConfig::default()) {
        Ok(contents) => {
            if let Err(e) = fs::write(&path, contents) {
                warn!("[save] Failed to write: {e}");
            } else {
                info!(
                    "[save] pos={:?} size={}x{} mode={:?}",
                    state.position, state.width, state.height, state.mode
                );
            }
        },
        Err(e) => warn!("[save] Failed to serialize: {e}"),
    }
}

/// System that saves window state when it changes
/// Uses `NonSendMarker` to force main thread execution (required for `WINIT_WINDOWS` access)
fn save_on_window_events(
    mut commands: Commands,
    mut tracker: ResMut<WindowStateTracker>,
    window_query: Query<(Entity, &Window, Option<&WindowSettling>), With<PrimaryWindow>>,
    monitors: Query<&Monitor>,
    // Forces this system to run on main thread where `WINIT_WINDOWS` thread_local is populated
    _non_send: NonSendMarker,
) {
    let Ok((window_entity, window, settling)) = window_query.single() else {
        return;
    };

    let current_position = match window.position {
        WindowPosition::At(pos) => Some(pos),
        _ => None,
    };
    let current_size = (window.width(), window.height());
    let current_mode = window.mode;

    let position_changed = tracker.position != current_position;
    let size_changed = tracker.size != Some(current_size);
    let mode_changed = tracker.mode != Some(current_mode);

    if position_changed || size_changed || mode_changed {
        match settling {
            Some(WindowSettling::Created) => {
                info!("[WindowSettling::Created] skipping save");
                tracker.position = current_position;
                tracker.size = Some(current_size);
                tracker.mode = Some(current_mode);
                return;
            },
            Some(WindowSettling::Moved) => {
                info!("[WindowSettling::Moved] skipping save, removing component");
                commands.entity(window_entity).remove::<WindowSettling>();
                tracker.position = current_position;
                tracker.size = Some(current_size);
                tracker.mode = Some(current_mode);
                return;
            },
            None => {},
        }
        save_window_state(window, window_entity, &monitors);
    }

    tracker.position = current_position;
    tracker.size = Some(current_size);
    tracker.mode = Some(current_mode);
}
