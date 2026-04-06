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
action!(PauseSwitch);
action!(PhysicsAabbSwitch);
action!(ShowFocusSwitch);

action!(CameraHome);
action!(RestartGameShortcut);
action!(RestartWithSplashShortcut);
action!(ZoomToFitShortcut);

// used by `Keybindings` for the shift modifier entity
action!(ModifySelection);

fn spawn_main_shortcuts(
    kb: &Keybindings<GlobalShortcutsContext>,
    ctx: &mut ActionSpawner<GlobalShortcutsContext>,
) {
    kb.spawn_key::<AabbsSwitch>(ctx, KeyCode::F1);
    kb.spawn_key::<PhysicsAabbSwitch>(ctx, KeyCode::F2);
    kb.spawn_key::<BoundaryBoxSwitch>(ctx, KeyCode::KeyB);
    kb.spawn_key::<CameraHome>(ctx, KeyCode::F12);
    kb.spawn_key::<ZoomToFitShortcut>(ctx, KeyCode::KeyZ);
    kb.spawn_key::<PauseSwitch>(ctx, KeyCode::Escape);
}

fn spawn_inspector_shortcuts(
    kb: &Keybindings<GlobalShortcutsContext>,
    ctx: &mut ActionSpawner<GlobalShortcutsContext>,
) {
    kb.spawn_shift_key::<InspectAabbSwitch>(ctx, KeyCode::KeyA);
    kb.spawn_shift_key::<InspectBoundarySwitch>(ctx, KeyCode::KeyB);
    kb.spawn_shift_key::<InspectCameraSwitch>(ctx, KeyCode::KeyC);
    kb.spawn_shift_key::<InspectFocusSwitch>(ctx, KeyCode::Digit5);
    kb.spawn_shift_key::<InspectLightsSwitch>(ctx, KeyCode::KeyL);
    kb.spawn_shift_key::<InspectMissileSwitch>(ctx, KeyCode::Digit1);
    kb.spawn_shift_key::<InspectNateroidSwitch>(ctx, KeyCode::Digit2);
    kb.spawn_shift_key::<InspectOutlineSwitch>(ctx, KeyCode::KeyO);
    kb.spawn_shift_key::<InspectPortalSwitch>(ctx, KeyCode::KeyG);
    kb.spawn_shift_key::<ShowFocusSwitch>(ctx, KeyCode::KeyF);
    kb.spawn_shift_key::<InspectSpaceshipSwitch>(ctx, KeyCode::Digit3);
    kb.spawn_shift_key::<InspectSpaceshipControlSwitch>(ctx, KeyCode::Digit4);
    kb.spawn_shift_key::<InspectStarSwitch>(ctx, KeyCode::KeyS);
    kb.spawn_shift_key::<InspectZoomSwitch>(ctx, KeyCode::KeyZ);
}

fn spawn_restart_shortcuts(
    kb: &Keybindings<GlobalShortcutsContext>,
    ctx: &mut ActionSpawner<GlobalShortcutsContext>,
) {
    // No `BlockBy` — the Cmd+Shift chord is its own disambiguator.
    ctx.spawn((
        Action::<RestartWithSplashShortcut>::new(),
        ActionSettings {
            consume_input: true,
            ..default()
        },
        bindings![KeyCode::KeyR.with_mod_keys(ModKeys::SUPER | ModKeys::SHIFT)],
    ));
    kb.spawn_shift_key::<RestartGameShortcut>(ctx, KeyCode::KeyR);
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
            move |ctx: &mut ActionSpawner<GlobalShortcutsContext>| {
                let kb = Keybindings::new::<ModifySelection>(ctx, consume_input);
                spawn_main_shortcuts(&kb, ctx);
                spawn_inspector_shortcuts(&kb, ctx);
                spawn_restart_shortcuts(&kb, ctx);
            },
        )),
    ));
}
