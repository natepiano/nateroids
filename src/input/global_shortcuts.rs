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

#[derive(InputAction)]
#[action_output(bool)]
pub struct PauseToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct RestartGameShortcut;

#[derive(InputAction)]
#[action_output(bool)]
pub struct RestartWithSplashShortcut;

pub fn setup_global_shortcuts_input(mut commands: Commands) {
    let restart_game_bindings = bindings![KeyCode::KeyR.with_mod_keys(ModKeys::SHIFT)];
    let restart_with_splash_bindings =
        bindings![KeyCode::KeyR.with_mod_keys(ModKeys::SUPER | ModKeys::SHIFT)];

    commands.spawn((
        GlobalShortcutsContext,
        actions!(GlobalShortcutsContext[
            (Action::<PhysicsAabbToggle>::new(), bindings![KeyCode::F2]),
            (Action::<BoundaryBoxToggle>::new(), bindings![KeyCode::KeyB]),
            (Action::<CameraHome>::new(), bindings![KeyCode::F12]),
            (Action::<ZoomToFitShortcut>::new(), bindings![KeyCode::KeyZ]),
            (Action::<PauseToggle>::new(), bindings![KeyCode::Escape]),
            // The more specific shortcut evaluates first (more ModKeys) and consumes input,
            // preventing Shift+R from also firing on Super+Shift+R.
            (
                Action::<RestartWithSplashShortcut>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                restart_with_splash_bindings,
            ),
            (
                Action::<RestartGameShortcut>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                restart_game_bindings,
            ),
        ]),
    ));
}
