use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component)]
pub struct ShipControlsContext;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShipAccelerate;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShipTurnLeft;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShipTurnRight;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShipFire;

#[derive(InputAction)]
#[action_output(bool)]
pub struct ShipContinuousFire;

#[derive(InputAction)]
#[action_output(bool)]
struct ShipShiftModifier;

pub fn ship_controls_input_bundle() -> impl Bundle {
    let consume_input = ActionSettings {
        consume_input: true,
        ..default()
    };
    // Internal "modifier state" action used only for gating other ship actions.
    //
    // Why non-consuming:
    // - This action should observe Shift state, not steal it from other bindings.
    //
    // Why require_reset:
    // - If a ship/context is spawned while Shift is already held, we avoid treating that as a
    //   fresh modifier activation until Shift is released and pressed again.
    let non_consuming_modifier = ActionSettings {
        consume_input: false,
        require_reset: true,
        ..default()
    };

    (
        ShipControlsContext,
        Actions::<ShipControlsContext>::spawn(SpawnWith(
            move |context: &mut ActionSpawner<ShipControlsContext>| {
                let shift_modifier = context
                    .spawn((
                        Action::<ShipShiftModifier>::new(),
                        non_consuming_modifier,
                        bindings![KeyCode::ShiftLeft, KeyCode::ShiftRight],
                    ))
                    .id();

                context.spawn((
                    Action::<ShipAccelerate>::new(),
                    consume_input,
                    bindings![KeyCode::KeyW, KeyCode::ArrowUp],
                ));
                context.spawn((
                    Action::<ShipTurnLeft>::new(),
                    consume_input,
                    bindings![KeyCode::KeyA, KeyCode::ArrowLeft],
                ));
                context.spawn((
                    Action::<ShipTurnRight>::new(),
                    consume_input,
                    bindings![KeyCode::KeyD, KeyCode::ArrowRight],
                ));
                context.spawn((
                    Action::<ShipFire>::new(),
                    consume_input,
                    bindings![KeyCode::Space],
                ));
                context.spawn((
                    Action::<ShipContinuousFire>::new(),
                    consume_input,
                    // Concrete edge case this handles:
                    // 1) Hold Shift
                    // 2) Press F (Shift+F is used by a different action)
                    // 3) Release Shift while still holding F
                    //
                    // Without these two conditions together, step (3) can produce a plain-F
                    // Start event and incorrectly toggle continuous fire.
                    //
                    // - Press: fire only on a fresh press edge.
                    // - BlockBy(shift_modifier): never fire while Shift modifier action is active.
                    //
                    // See regression coverage:
                    // tests/bei_chord_overlap.rs::press_plus_blockby_shift_prevents_toggle_on_shift_release
                    bevy_enhanced_input::condition::press::Press::default(),
                    BlockBy::single(shift_modifier),
                    bindings![KeyCode::KeyF],
                ));
            },
        )),
    )
}
