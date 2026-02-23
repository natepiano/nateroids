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

#[derive(InputAction)]
#[action_output(bool)]
pub struct AabbConfigInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct AabbsToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct BoundaryInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct CameraConfigInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct FocusConfigInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct LightsInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct MissileInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct NateroidInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct OutlineInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PlanesInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct PortalInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShowFocusToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct SpaceshipInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct SpaceshipControlInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct StarConfigInspectorToggle;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ZoomConfigInspectorToggle;

fn spawn_shortcut<A, B>(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    action_settings: ActionSettings,
    bindings: B,
) where
    A: InputAction,
    B: Bundle,
{
    context.spawn((Action::<A>::new(), action_settings, bindings));
}

fn spawn_shortcut_key<A>(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    action_settings: ActionSettings,
    key: KeyCode,
) where
    A: InputAction,
{
    spawn_shortcut::<A, _>(context, action_settings, bindings![key]);
}

fn spawn_shift_shortcut<A>(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    action_settings: ActionSettings,
    key: KeyCode,
) where
    A: InputAction,
{
    spawn_shortcut::<A, _>(
        context,
        action_settings,
        bindings![key.with_mod_keys(ModKeys::SHIFT)],
    );
}

fn spawn_main_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    action_settings: ActionSettings,
) {
    spawn_shortcut_key::<AabbsToggle>(context, action_settings, KeyCode::F1);
    spawn_shortcut_key::<PhysicsAabbToggle>(context, action_settings, KeyCode::F2);
    spawn_shortcut_key::<BoundaryBoxToggle>(context, action_settings, KeyCode::KeyB);
    spawn_shortcut_key::<CameraHome>(context, action_settings, KeyCode::F12);
    spawn_shortcut_key::<ZoomToFitShortcut>(context, action_settings, KeyCode::KeyZ);
    spawn_shortcut_key::<PauseToggle>(context, action_settings, KeyCode::Escape);
}

fn spawn_inspector_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    action_settings: ActionSettings,
) {
    spawn_shift_shortcut::<AabbConfigInspectorToggle>(context, action_settings, KeyCode::KeyA);
    spawn_shift_shortcut::<BoundaryInspectorToggle>(context, action_settings, KeyCode::KeyB);
    spawn_shift_shortcut::<CameraConfigInspectorToggle>(context, action_settings, KeyCode::KeyC);
    spawn_shift_shortcut::<FocusConfigInspectorToggle>(context, action_settings, KeyCode::Digit5);
    spawn_shift_shortcut::<LightsInspectorToggle>(context, action_settings, KeyCode::KeyL);
    spawn_shift_shortcut::<MissileInspectorToggle>(context, action_settings, KeyCode::Digit1);
    spawn_shift_shortcut::<NateroidInspectorToggle>(context, action_settings, KeyCode::Digit2);
    spawn_shift_shortcut::<OutlineInspectorToggle>(context, action_settings, KeyCode::KeyO);
    spawn_shift_shortcut::<PlanesInspectorToggle>(context, action_settings, KeyCode::KeyP);
    spawn_shift_shortcut::<PortalInspectorToggle>(context, action_settings, KeyCode::KeyG);
    spawn_shift_shortcut::<ShowFocusToggle>(context, action_settings, KeyCode::KeyF);
    spawn_shift_shortcut::<SpaceshipInspectorToggle>(context, action_settings, KeyCode::Digit3);
    spawn_shift_shortcut::<SpaceshipControlInspectorToggle>(
        context,
        action_settings,
        KeyCode::Digit4,
    );
    spawn_shift_shortcut::<StarConfigInspectorToggle>(context, action_settings, KeyCode::KeyS);
    spawn_shift_shortcut::<ZoomConfigInspectorToggle>(context, action_settings, KeyCode::KeyZ);
}

fn spawn_restart_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    action_settings: ActionSettings,
) {
    // The more specific shortcut evaluates first (more ModKeys) and consumes input,
    // preventing Shift+R from also firing on Super+Shift+R.
    spawn_shortcut::<RestartWithSplashShortcut, _>(
        context,
        action_settings,
        bindings![KeyCode::KeyR.with_mod_keys(ModKeys::SUPER | ModKeys::SHIFT)],
    );
    spawn_shift_shortcut::<RestartGameShortcut>(context, action_settings, KeyCode::KeyR);
}

pub fn setup_global_shortcuts_input(mut commands: Commands) {
    let consume_input = ActionSettings {
        consume_input: true,
        ..default()
    };

    commands.spawn((
        GlobalShortcutsContext,
        // Evaluate global shortcuts before entity-scoped contexts so Shift+<key>
        // chords can win over ship controls bound to the base key.
        ContextPriority::<GlobalShortcutsContext>::new(100),
        Actions::<GlobalShortcutsContext>::spawn(SpawnWith(
            move |context: &mut ActionSpawner<GlobalShortcutsContext>| {
                spawn_main_shortcuts(context, consume_input);
                spawn_inspector_shortcuts(context, consume_input);
                spawn_restart_shortcuts(context, consume_input);
            },
        )),
    ));
}
