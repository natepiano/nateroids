use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use bevy_kana::Keybindings;

use super::constants::GLOBAL_SHORTCUTS_PRIORITY;

#[derive(Component)]
pub(super) struct GlobalShortcutsContext;

action!(AabbsSwitch);
action!(BoundaryBoxSwitch);
action!(InspectAabbSwitch);
action!(InspectBoundarySwitch);
action!(InspectCameraSwitch);
action!(InspectFocusSwitch);
action!(InspectLightsSwitch);
action!(InspectMissileSwitch);
action!(InspectNateroidSwitch);
action!(InspectOutlineSwitch);
action!(InspectPortalSwitch);
action!(InspectSpaceshipControlSwitch);
action!(InspectSpaceshipSwitch);
action!(InspectStarSwitch);
action!(InspectZoomSwitch);
action!(EscapeSwitch);
action!(PhysicsAabbSwitch);
action!(ShowFocusSwitch);

action!(CameraHome);
action!(RestartGameShortcut);
action!(RestartWithSplashShortcut);
action!(ZoomToFitShortcut);

// used by `Keybindings` for the shift modifier entity
action!(ModifySelection);

fn spawn_main_shortcuts(
    keybindings: &Keybindings<GlobalShortcutsContext>,
    spawner: &mut ActionSpawner<GlobalShortcutsContext>,
) {
    keybindings.spawn_key::<AabbsSwitch>(spawner, KeyCode::F1);
    keybindings.spawn_key::<PhysicsAabbSwitch>(spawner, KeyCode::F2);
    keybindings.spawn_key::<BoundaryBoxSwitch>(spawner, KeyCode::KeyB);
    keybindings.spawn_key::<CameraHome>(spawner, KeyCode::F12);
    keybindings.spawn_key::<ZoomToFitShortcut>(spawner, KeyCode::KeyZ);
    keybindings.spawn_key::<EscapeSwitch>(spawner, KeyCode::Escape);
}

fn spawn_inspector_shortcuts(
    keybindings: &Keybindings<GlobalShortcutsContext>,
    spawner: &mut ActionSpawner<GlobalShortcutsContext>,
) {
    keybindings.spawn_shift_key::<InspectAabbSwitch>(spawner, KeyCode::KeyA);
    keybindings.spawn_shift_key::<InspectBoundarySwitch>(spawner, KeyCode::KeyB);
    keybindings.spawn_shift_key::<InspectCameraSwitch>(spawner, KeyCode::KeyC);
    keybindings.spawn_shift_key::<InspectFocusSwitch>(spawner, KeyCode::Digit5);
    keybindings.spawn_shift_key::<InspectLightsSwitch>(spawner, KeyCode::KeyL);
    keybindings.spawn_shift_key::<InspectMissileSwitch>(spawner, KeyCode::Digit1);
    keybindings.spawn_shift_key::<InspectNateroidSwitch>(spawner, KeyCode::Digit2);
    keybindings.spawn_shift_key::<InspectOutlineSwitch>(spawner, KeyCode::KeyO);
    keybindings.spawn_shift_key::<InspectPortalSwitch>(spawner, KeyCode::KeyG);
    keybindings.spawn_shift_key::<ShowFocusSwitch>(spawner, KeyCode::KeyF);
    keybindings.spawn_shift_key::<InspectSpaceshipSwitch>(spawner, KeyCode::Digit3);
    keybindings.spawn_shift_key::<InspectSpaceshipControlSwitch>(spawner, KeyCode::Digit4);
    keybindings.spawn_shift_key::<InspectStarSwitch>(spawner, KeyCode::KeyS);
    keybindings.spawn_shift_key::<InspectZoomSwitch>(spawner, KeyCode::KeyZ);
}

fn spawn_restart_shortcuts(
    keybindings: &Keybindings<GlobalShortcutsContext>,
    spawner: &mut ActionSpawner<GlobalShortcutsContext>,
) {
    // No `BlockBy` — the Cmd+Shift chord is its own disambiguator.
    spawner.spawn((
        Action::<RestartWithSplashShortcut>::new(),
        ActionSettings {
            consume_input: true,
            ..default()
        },
        bindings![KeyCode::KeyR.with_mod_keys(ModKeys::SUPER | ModKeys::SHIFT)],
    ));
    keybindings.spawn_shift_key::<RestartGameShortcut>(spawner, KeyCode::KeyR);
}

pub(super) fn setup_global_shortcuts(mut commands: Commands) {
    let consume_input = ActionSettings {
        consume_input: true,
        ..default()
    };

    commands.spawn((
        GlobalShortcutsContext,
        // Evaluate global shortcuts before entity-scoped contexts so Shift+<key>
        // chords can win over ship controls bound to the base key.
        ContextPriority::<GlobalShortcutsContext>::new(GLOBAL_SHORTCUTS_PRIORITY),
        Actions::<GlobalShortcutsContext>::spawn(SpawnWith(
            move |spawner: &mut ActionSpawner<GlobalShortcutsContext>| {
                let keybindings = Keybindings::new::<ModifySelection>(spawner, consume_input);
                spawn_main_shortcuts(&keybindings, spawner);
                spawn_inspector_shortcuts(&keybindings, spawner);
                spawn_restart_shortcuts(&keybindings, spawner);
            },
        )),
    ));
}
