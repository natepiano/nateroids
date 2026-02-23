mod global_shortcuts;

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub use global_shortcuts::PhysicsAabbToggle;

pub struct EnhancedInputAppPlugin;

impl Plugin for EnhancedInputAppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            .add_input_context::<global_shortcuts::GlobalShortcutsContext>()
            .add_systems(Startup, global_shortcuts::setup_global_shortcuts_input);
    }
}
