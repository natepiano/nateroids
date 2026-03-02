//! Input action definitions and context setup for keyboard/game controls.
//!
//! Command flow pattern used by the app:
//! 1. Keyboard input fires enhanced-input action events (`Start<...>`).
//! 2. Input observers adapt those events into app-level command events (for example
//!    `RestartGameEvent`, `PauseEvent`, `ZoomToFitEvent`).
//! 3. Command-event observers run reusable command systems via `run_system_cached(...)`.
//!
//! Why this split:
//! - User input stays ergonomic and keybinding-driven.
//! - BRP can trigger the same behavior via `world.trigger_event` without simulating key presses.
//! - Tests can exercise either layer:
//!   - behavior-level tests by triggering app command events;
//!   - system-level tests by invoking command systems directly.
#[macro_use]
mod bei_extras;
mod global_shortcuts;
mod ship_controls;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
pub use global_shortcuts::AabbConfigInspectorSwitch;
pub use global_shortcuts::AabbsSwitch;
pub use global_shortcuts::BoundaryBoxSwitch;
pub use global_shortcuts::BoundaryInspectorToggle;
pub use global_shortcuts::CameraConfigInspectorSwitch;
pub use global_shortcuts::CameraHome;
pub use global_shortcuts::CameraHomeEvent;
pub use global_shortcuts::FocusConfigInspectorSwitch;
pub use global_shortcuts::LightsInspectorSwitch;
pub use global_shortcuts::MissileInspectorSwitch;
pub use global_shortcuts::NateroidInspectorSwitch;
pub use global_shortcuts::OutlineInspectorSwitch;
pub use global_shortcuts::PauseEvent;
pub use global_shortcuts::PauseToggle;
pub use global_shortcuts::PhysicsAabbSwitch;
pub use global_shortcuts::PlanesInspectorSwitch;
pub use global_shortcuts::PortalInspectorSwitch;
pub use global_shortcuts::RestartGameEvent;
pub use global_shortcuts::RestartGameShortcut;
pub use global_shortcuts::RestartWithSplashEvent;
pub use global_shortcuts::RestartWithSplashShortcut;
pub use global_shortcuts::ShowFocusSwitch;
pub use global_shortcuts::SpaceshipControlInspectorSwitch;
pub use global_shortcuts::SpaceshipInspectorSwitch;
pub use global_shortcuts::StarConfigInspectorSwitch;
pub use global_shortcuts::ToggleFitTargetDebugEvent;
pub use global_shortcuts::ZoomConfigInspectorSwitch;
pub use global_shortcuts::ZoomToFitEvent;
pub use global_shortcuts::ZoomToFitShortcut;
pub use ship_controls::ShipAccelerate;
pub use ship_controls::ShipContinuousFire;
pub use ship_controls::ShipControlsContext;
pub use ship_controls::ShipFire;
pub use ship_controls::ShipTurnLeft;
pub use ship_controls::ShipTurnRight;
pub use ship_controls::ship_controls_input_bundle;

pub struct EnhancedInputAppPlugin;

impl Plugin for EnhancedInputAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<global_shortcuts::GlobalShortcutsContext>()
            .add_input_context::<ship_controls::ShipControlsContext>()
            .add_systems(Startup, global_shortcuts::setup_global_shortcuts);
    }
}
