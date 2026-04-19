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
pub use global_shortcuts::AabbsSwitch;
pub use global_shortcuts::BoundaryBoxSwitch;
pub use global_shortcuts::CameraHome;
pub use global_shortcuts::EscapeSwitch;
pub use global_shortcuts::InspectAabbSwitch;
pub use global_shortcuts::InspectBoundarySwitch;
pub use global_shortcuts::InspectCameraSwitch;
pub use global_shortcuts::InspectFocusSwitch;
pub use global_shortcuts::InspectLightsSwitch;
pub use global_shortcuts::InspectMissileSwitch;
pub use global_shortcuts::InspectNateroidSwitch;
pub use global_shortcuts::InspectOutlineSwitch;
pub use global_shortcuts::InspectPortalSwitch;
pub use global_shortcuts::InspectSpaceshipControlSwitch;
pub use global_shortcuts::InspectSpaceshipSwitch;
pub use global_shortcuts::InspectStarSwitch;
pub use global_shortcuts::InspectZoomSwitch;
pub use global_shortcuts::PhysicsAabbSwitch;
pub use global_shortcuts::RestartGameShortcut;
pub use global_shortcuts::RestartWithSplashShortcut;
pub use global_shortcuts::ShowFocusSwitch;
pub use global_shortcuts::ZoomToFitShortcut;
pub use ship_controls::ShipAccelerate;
pub use ship_controls::ShipContinuousFire;
pub use ship_controls::ShipControlsContext;
pub use ship_controls::ShipFire;
pub use ship_controls::ShipTurnLeft;
pub use ship_controls::ShipTurnRight;
pub use ship_controls::ship_controls_input_bundle;

pub(crate) struct EnhancedInputAppPlugin;

impl Plugin for EnhancedInputAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<global_shortcuts::GlobalShortcutsContext>()
            .add_input_context::<ship_controls::ShipControlsContext>()
            .add_systems(Startup, global_shortcuts::setup_global_shortcuts);
    }
}
