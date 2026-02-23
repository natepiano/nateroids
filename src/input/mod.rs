mod global_shortcuts;
mod ship_controls;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub use global_shortcuts::BoundaryBoxToggle;
pub use global_shortcuts::CameraHome;
pub use global_shortcuts::PauseToggle;
pub use global_shortcuts::PhysicsAabbToggle;
pub use global_shortcuts::RestartGameShortcut;
pub use global_shortcuts::RestartWithSplashShortcut;
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
