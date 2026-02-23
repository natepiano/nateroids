use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
pub struct GlobalShortcutsContext;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PhysicsAabbToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct BoundaryBoxToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct CameraHome;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ZoomToFitShortcut;

pub fn setup_global_shortcuts_input(mut commands: Commands) {
    commands.spawn((
        GlobalShortcutsContext,
        actions!(GlobalShortcutsContext[
            (Action::<PhysicsAabbToggle>::new(), bindings![KeyCode::F2]),
            (Action::<BoundaryBoxToggle>::new(), bindings![KeyCode::KeyB]),
            (Action::<CameraHome>::new(), bindings![KeyCode::F12]),
            (Action::<ZoomToFitShortcut>::new(), bindings![KeyCode::KeyZ]),
        ]),
    ));
}
