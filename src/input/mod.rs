//! Input action definitions and context setup for keyboard/game controls.
//!
//! Command flow pattern used by the app:
//! 1. Keyboard input fires enhanced-input action events (`Start<...>`).
//! 2. Input observers adapt those events into app-level command events (for example
//!    `RestartGameEvent`, `EscapeEvent`, `ZoomToFitEvent`).
//! 3. Command-event observers run reusable command systems via `run_system_cached(...)`.
//!
//! Why this split:
//! - User input stays ergonomic and keybinding-driven.
//! - BRP can trigger the same behavior via `world.trigger_event` without simulating key presses.
//! - Tests can exercise either layer:
//!   - behavior-level tests by triggering app command events;
//!   - system-level tests by invoking command systems directly.
mod constants;
mod global_shortcuts;
mod ship_controls;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
pub(crate) use global_shortcuts::AabbsSwitch;
pub(crate) use global_shortcuts::BoundaryBoxSwitch;
pub(crate) use global_shortcuts::CameraHome;
pub(crate) use global_shortcuts::EscapeSwitch;
pub(crate) use global_shortcuts::InspectAabbSwitch;
pub(crate) use global_shortcuts::InspectBoundarySwitch;
pub(crate) use global_shortcuts::InspectCameraSwitch;
pub(crate) use global_shortcuts::InspectFocusSwitch;
pub(crate) use global_shortcuts::InspectLightsSwitch;
pub(crate) use global_shortcuts::InspectMissileSwitch;
pub(crate) use global_shortcuts::InspectNateroidSwitch;
pub(crate) use global_shortcuts::InspectOutlineSwitch;
pub(crate) use global_shortcuts::InspectPortalSwitch;
pub(crate) use global_shortcuts::InspectSpaceshipControlSwitch;
pub(crate) use global_shortcuts::InspectSpaceshipSwitch;
pub(crate) use global_shortcuts::InspectStarSwitch;
pub(crate) use global_shortcuts::InspectZoomSwitch;
pub(crate) use global_shortcuts::PhysicsAabbSwitch;
pub(crate) use global_shortcuts::RestartGameShortcut;
pub(crate) use global_shortcuts::RestartWithSplashShortcut;
pub(crate) use global_shortcuts::ShowFocusSwitch;
pub(crate) use global_shortcuts::ZoomToFitShortcut;
pub(crate) use ship_controls::ShipAccelerate;
pub(crate) use ship_controls::ShipContinuousFire;
pub(crate) use ship_controls::ShipControlsContext;
pub(crate) use ship_controls::ShipFire;
pub(crate) use ship_controls::ShipTurnLeft;
pub(crate) use ship_controls::ShipTurnRight;
pub(crate) use ship_controls::insert_ship_controls;

pub(crate) struct EnhancedInputAppPlugin;

impl Plugin for EnhancedInputAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<global_shortcuts::GlobalShortcutsContext>()
            .add_input_context::<ship_controls::ShipControlsContext>()
            .add_systems(Startup, global_shortcuts::setup_global_shortcuts);
    }
}
