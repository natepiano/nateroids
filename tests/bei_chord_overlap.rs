//! Integration test for BEI chord overlap behavior across contexts.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use bevy_enhanced_input::action::events;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
struct GlobalContext;

#[derive(Component)]
struct ShipContext;

#[derive(InputAction)]
#[action_output(bool)]
struct ShowFocusToggle;

#[derive(InputAction)]
#[action_output(bool)]
struct ContinuousFireToggle;

#[derive(Resource, Default)]
struct TriggerCounts {
    show_focus:      u32,
    continuous_fire: u32,
}

#[derive(InputAction)]
#[action_output(bool)]
struct PlainF;

#[derive(InputAction)]
#[action_output(bool)]
struct ShiftF;

#[derive(InputAction)]
#[action_output(bool)]
struct ShiftHeld;

#[derive(InputAction)]
#[action_output(bool)]
struct ContinuousFireBlocked;

#[derive(Resource, Default)]
struct SameContextCounts {
    plain_f: u32,
    shift_f: u32,
}

#[derive(Resource, Default)]
struct BlockedCounts {
    continuous_fire: u32,
}

fn on_show_focus_input(
    _trigger: On<events::Start<ShowFocusToggle>>,
    mut counts: ResMut<TriggerCounts>,
) {
    counts.show_focus += 1;
}

fn on_continuous_fire_input(
    _trigger: On<events::Start<ContinuousFireToggle>>,
    mut counts: ResMut<TriggerCounts>,
) {
    counts.continuous_fire += 1;
}

fn on_plain_f_input(_trigger: On<events::Start<PlainF>>, mut counts: ResMut<SameContextCounts>) {
    counts.plain_f += 1;
}

fn on_shift_f_input(_trigger: On<events::Start<ShiftF>>, mut counts: ResMut<SameContextCounts>) {
    counts.shift_f += 1;
}

fn on_continuous_fire_blocked_input(
    _trigger: On<events::Start<ContinuousFireBlocked>>,
    mut counts: ResMut<BlockedCounts>,
) {
    counts.continuous_fire += 1;
}

fn setup_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<GlobalContext>()
        .add_input_context::<ShipContext>()
        .init_resource::<TriggerCounts>()
        .add_observer(on_show_focus_input)
        .add_observer(on_continuous_fire_input);

    app.finish();
    app
}

fn spawn_contexts(app: &mut App) {
    app.world_mut().spawn((
        GlobalContext,
        ContextPriority::<GlobalContext>::new(100),
        actions!(GlobalContext[
            (
                Action::<ShowFocusToggle>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                bindings![KeyCode::KeyF.with_mod_keys(ModKeys::SHIFT)],
            ),
        ]),
    ));

    app.update();

    app.world_mut().spawn((
        ShipContext,
        actions!(ShipContext[
            (
                Action::<ContinuousFireToggle>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                bindings![KeyCode::KeyF],
            ),
        ]),
    ));

    app.update();
}

#[test]
fn shift_f_should_not_trigger_plain_f_in_other_context_when_consuming() {
    let mut app = setup_app();
    spawn_contexts(&mut app);

    // Hold Shift first.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.update();

    // Then press F.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();

    let counts = app.world().resource::<TriggerCounts>();
    assert_eq!(
        counts.show_focus, 1,
        "Shift+F should trigger ShowFocus once"
    );
    assert_eq!(
        counts.continuous_fire, 0,
        "Shift+F should not trigger plain F in another context when consumption works"
    );
}

#[test]
fn releasing_shift_while_holding_f_triggers_plain_f_after_chord() {
    let mut app = setup_app();
    spawn_contexts(&mut app);

    // Hold Shift first.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.update();

    // Then press F.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();

    // Release Shift while still holding F.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ShiftLeft);
    app.update();

    let counts = app.world().resource::<TriggerCounts>();
    assert_eq!(
        counts.show_focus, 1,
        "Shift+F should trigger ShowFocus once"
    );
    assert_eq!(
        counts.continuous_fire, 1,
        "Releasing Shift while holding F should trigger plain F once"
    );
}

#[test]
fn same_context_also_triggers_plain_f_after_shift_release() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<GlobalContext>()
        .init_resource::<SameContextCounts>()
        .add_observer(on_plain_f_input)
        .add_observer(on_shift_f_input);
    app.finish();

    app.world_mut().spawn((
        GlobalContext,
        actions!(GlobalContext[
            (
                Action::<ShiftF>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                bindings![KeyCode::KeyF.with_mod_keys(ModKeys::SHIFT)],
            ),
            (
                Action::<PlainF>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                bindings![KeyCode::KeyF],
            ),
        ]),
    ));
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ShiftLeft);
    app.update();

    let counts = app.world().resource::<SameContextCounts>();
    assert_eq!(counts.shift_f, 1, "Shift+F should trigger once");
    assert_eq!(
        counts.plain_f, 1,
        "Releasing Shift while holding F should trigger plain F in same context too"
    );
}

#[test]
fn press_plus_blockby_shift_prevents_toggle_on_shift_release() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_plugins(EnhancedInputPlugin)
        .add_input_context::<ShipContext>()
        .init_resource::<BlockedCounts>()
        .add_observer(on_continuous_fire_blocked_input);
    app.finish();

    app.world_mut().spawn((
        ShipContext,
        Actions::<ShipContext>::spawn(SpawnWith(|context: &mut ActionSpawner<ShipContext>| {
            let shift = context
                .spawn((
                    Action::<ShiftHeld>::new(),
                    ActionSettings {
                        consume_input: false,
                        ..default()
                    },
                    bindings![KeyCode::ShiftLeft, KeyCode::ShiftRight],
                ))
                .id();

            context.spawn((
                Action::<ContinuousFireBlocked>::new(),
                ActionSettings {
                    consume_input: true,
                    ..default()
                },
                bevy_enhanced_input::condition::press::Press::default(),
                BlockBy::single(shift),
                bindings![KeyCode::KeyF],
            ));
        })),
    ));
    app.update();

    // Hold Shift first, then press F.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();

    // Release Shift while still holding F.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::ShiftLeft);
    app.update();

    let counts = app.world().resource::<BlockedCounts>();
    assert_eq!(
        counts.continuous_fire, 0,
        "Press + BlockBy(Shift) should prevent firing when Shift is released while F is held"
    );

    // After F is released, a clean unmodified press should fire.
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::KeyF);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyF);
    app.update();

    let counts = app.world().resource::<BlockedCounts>();
    assert_eq!(
        counts.continuous_fire, 1,
        "plain F should still fire on a clean press"
    );
}
