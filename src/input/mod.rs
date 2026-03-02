//! Input action definitions and context setup for keyboard/game controls.
//!
//! Command flow pattern used by the app:
//! 1. Keyboard input fires enhanced-input action events (`Start<...>`).
//! 2. Input observers adapt those events into app-level command events
//!    (for example `RestartGameEvent`, `PauseEvent`, `ZoomToFitEvent`).
//! 3. Command-event observers run reusable command systems via
//!    `run_system_cached(...)`.
//!
//! Why this split:
//! - User input stays ergonomic and keybinding-driven.
//! - BRP can trigger the same behavior via `world.trigger_event` without
//!   simulating key presses.
//! - Tests can exercise either layer:
//!   - behavior-level tests by triggering app command events;
//!   - system-level tests by invoking command systems directly.
mod global_shortcuts;
mod ship_controls;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
pub use global_shortcuts::AabbConfigInspectorToggle;
pub use global_shortcuts::AabbsToggle;
pub use global_shortcuts::BoundaryBoxToggle;
pub use global_shortcuts::BoundaryInspectorToggle;
pub use global_shortcuts::CameraConfigInspectorToggle;
pub use global_shortcuts::CameraHome;
pub use global_shortcuts::FocusConfigInspectorToggle;
pub use global_shortcuts::LightsInspectorToggle;
pub use global_shortcuts::MissileInspectorToggle;
pub use global_shortcuts::NateroidInspectorToggle;
pub use global_shortcuts::OutlineInspectorToggle;
pub use global_shortcuts::PauseToggle;
pub use global_shortcuts::PhysicsAabbToggle;
pub use global_shortcuts::PlanesInspectorToggle;
pub use global_shortcuts::PortalInspectorToggle;
pub use global_shortcuts::RestartGameShortcut;
pub use global_shortcuts::RestartWithSplashShortcut;
pub use global_shortcuts::ShowFocusToggle;
pub use global_shortcuts::SpaceshipControlInspectorToggle;
pub use global_shortcuts::SpaceshipInspectorToggle;
pub use global_shortcuts::StarConfigInspectorToggle;
pub use global_shortcuts::ZoomConfigInspectorToggle;
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
            .add_systems(Startup, global_shortcuts::setup_global_shortcuts_input);
    }
}
