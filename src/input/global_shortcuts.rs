use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::bei_extras::ActionSpawnerExt;

#[derive(Component)]
pub struct GlobalShortcutsContext;

action!(AabbsSwitch);
action!(BoundaryBoxSwitch);
action!(InspectAabbConfigSwitch);
action!(InspectBoundarySwitch);
action!(InspectCameraConfigSwitch);
action!(InspectFocusConfigSwitch);
action!(InspectLightsSwitch);
action!(InspectMissileSwitch);
action!(InspectNateroidSwitch);
action!(InspectOutlineSwitch);
action!(InspectPlanesSwitch);
action!(InspectPortalSwitch);
action!(InspectSpaceshipControlSwitch);
action!(InspectSpaceshipSwitch);
action!(InspectStarConfigSwitch);
action!(InspectZoomConfigSwitch);
action!(PauseSwitch);
action!(PhysicsAabbSwitch);
action!(ShowFocusSwitch);

action!(CameraHome);
action!(RestartGameShortcut);
action!(RestartWithSplashShortcut);
action!(ZoomToFitShortcut);

fn spawn_main_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    settings: ActionSettings,
) {
    context.spawn_key::<AabbsSwitch>(settings, KeyCode::F1);
    context.spawn_key::<PhysicsAabbSwitch>(settings, KeyCode::F2);
    context.spawn_key::<BoundaryBoxSwitch>(settings, KeyCode::KeyB);
    context.spawn_key::<CameraHome>(settings, KeyCode::F12);
    context.spawn_key::<ZoomToFitShortcut>(settings, KeyCode::KeyZ);
    context.spawn_key::<PauseSwitch>(settings, KeyCode::Escape);
}

fn spawn_inspector_shortcuts(
    context: &mut ActionSpawner<GlobalShortcutsContext>,
    settings: ActionSettings,
) {
    context.spawn_shift_key::<InspectAabbConfigSwitch>(settings, KeyCode::KeyA);
    context.spawn_shift_key::<InspectBoundarySwitch>(settings, KeyCode::KeyB);
    context.spawn_shift_key::<InspectCameraConfigSwitch>(settings, KeyCode::KeyC);
    context.spawn_shift_key::<InspectFocusConfigSwitch>(settings, KeyCode::Digit5);
    context.spawn_shift_key::<InspectLightsSwitch>(settings, KeyCode::KeyL);
    context.spawn_shift_key::<InspectMissileSwitch>(settings, KeyCode::Digit1);
    context.spawn_shift_key::<InspectNateroidSwitch>(settings, KeyCode::Digit2);
    context.spawn_shift_key::<InspectOutlineSwitch>(settings, KeyCode::KeyO);
    context.spawn_shift_key::<InspectPlanesSwitch>(settings, KeyCode::KeyP);
    context.spawn_shift_key::<InspectPortalSwitch>(settings, KeyCode::KeyG);
    context.spawn_shift_key::<ShowFocusSwitch>(settings, KeyCode::KeyF);
    context.spawn_shift_key::<InspectSpaceshipSwitch>(settings, KeyCode::Digit3);
    context.spawn_shift_key::<InspectSpaceshipControlSwitch>(settings, KeyCode::Digit4);
    context.spawn_shift_key::<InspectStarConfigSwitch>(settings, KeyCode::KeyS);
    context.spawn_shift_key::<InspectZoomConfigSwitch>(settings, KeyCode::KeyZ);
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
