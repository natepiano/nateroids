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

pub fn ship_controls_input_bundle() -> impl Bundle {
    (
        ShipControlsContext,
        actions!(ShipControlsContext[
            (
                Action::<ShipAccelerate>::new(),
                bindings![KeyCode::KeyW, KeyCode::ArrowUp],
            ),
            (
                Action::<ShipTurnLeft>::new(),
                bindings![KeyCode::KeyA, KeyCode::ArrowLeft],
            ),
            (
                Action::<ShipTurnRight>::new(),
                bindings![KeyCode::KeyD, KeyCode::ArrowRight],
            ),
            (Action::<ShipFire>::new(), bindings![KeyCode::Space]),
            (
                Action::<ShipContinuousFire>::new(),
                bindings![KeyCode::KeyF],
            ),
        ]),
    )
}
