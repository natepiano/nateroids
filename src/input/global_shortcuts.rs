use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
pub struct GlobalShortcutsContext;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PhysicsAabbToggle;

pub fn setup_global_shortcuts_input(mut commands: Commands) {
    commands.spawn((
        GlobalShortcutsContext,
        actions!(GlobalShortcutsContext[
            (Action::<PhysicsAabbToggle>::new(), bindings![KeyCode::F2]),
        ]),
    ));
}
