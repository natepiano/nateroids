use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::bei_extras::ActionSpawnerExt;

#[derive(Component)]
pub struct GlobalShortcutsContext;

action!(AabbConfigInspectorSwitch);
action!(AabbsSwitch);
action!(BoundaryInspectorToggle);
action!(CameraConfigInspectorSwitch);
action!(FocusConfigInspectorSwitch);
action!(LightsInspectorSwitch);
action!(MissileInspectorSwitch);
action!(NateroidInspectorSwitch);
action!(OutlineInspectorSwitch);
action!(PhysicsAabbSwitch);
action!(PlanesInspectorSwitch);
action!(PortalInspectorSwitch);
action!(ShowFocusSwitch);
action!(SpaceshipControlInspectorSwitch);
action!(SpaceshipInspectorSwitch);
action!(StarConfigInspectorSwitch);
action!(ZoomConfigInspectorSwitch);

action_event!(BoundaryBoxSwitch, ToggleFitTargetDebugEvent);
action_event!(CameraHome, CameraHomeEvent);
action_event!(PauseToggle, PauseEvent);
action_event!(RestartGameShortcut, RestartGameEvent);
action_event!(RestartWithSplashShortcut, RestartWithSplashEvent);
action_event!(ZoomToFitShortcut, ZoomToFitEvent);

fn spawn_main_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    settings: ActionSettings,
) {
    context.spawn_key::<AabbsSwitch>(settings, KeyCode::F1);
    context.spawn_key::<PhysicsAabbSwitch>(settings, KeyCode::F2);
    context.spawn_key::<BoundaryBoxSwitch>(settings, KeyCode::KeyB);
    context.spawn_key::<CameraHome>(settings, KeyCode::F12);
    context.spawn_key::<ZoomToFitShortcut>(settings, KeyCode::KeyZ);
    context.spawn_key::<PauseToggle>(settings, KeyCode::Escape);
}

fn spawn_inspector_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    settings: ActionSettings,
) {
    context.spawn_shift_key::<AabbConfigInspectorSwitch>(settings, KeyCode::KeyA);
    context.spawn_shift_key::<BoundaryInspectorToggle>(settings, KeyCode::KeyB);
    context.spawn_shift_key::<CameraConfigInspectorSwitch>(settings, KeyCode::KeyC);
    context.spawn_shift_key::<FocusConfigInspectorSwitch>(settings, KeyCode::Digit5);
    context.spawn_shift_key::<LightsInspectorSwitch>(settings, KeyCode::KeyL);
    context.spawn_shift_key::<MissileInspectorSwitch>(settings, KeyCode::Digit1);
    context.spawn_shift_key::<NateroidInspectorSwitch>(settings, KeyCode::Digit2);
    context.spawn_shift_key::<OutlineInspectorSwitch>(settings, KeyCode::KeyO);
    context.spawn_shift_key::<PlanesInspectorSwitch>(settings, KeyCode::KeyP);
    context.spawn_shift_key::<PortalInspectorSwitch>(settings, KeyCode::KeyG);
    context.spawn_shift_key::<ShowFocusSwitch>(settings, KeyCode::KeyF);
    context.spawn_shift_key::<SpaceshipInspectorSwitch>(settings, KeyCode::Digit3);
    context.spawn_shift_key::<SpaceshipControlInspectorSwitch>(settings, KeyCode::Digit4);
    context.spawn_shift_key::<StarConfigInspectorSwitch>(settings, KeyCode::KeyS);
    context.spawn_shift_key::<ZoomConfigInspectorSwitch>(settings, KeyCode::KeyZ);
}

fn spawn_restart_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    settings: ActionSettings,
) {
    // The more specific shortcut evaluates first (more ModKeys) and consumes input,
    // preventing Shift+R from also firing on Super+Shift+R.
    context.spawn_binding::<RestartWithSplashShortcut, _>(
        settings,
        bindings![KeyCode::KeyR.with_mod_keys(ModKeys::SUPER | ModKeys::SHIFT)],
    );
    context.spawn_shift_key::<RestartGameShortcut>(settings, KeyCode::KeyR);
}

pub fn setup_global_shortcuts(mut commands: Commands) {
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
