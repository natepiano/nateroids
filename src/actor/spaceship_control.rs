use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::action::TriggerState;
use bevy_enhanced_input::action::events as input_events;
use bevy_enhanced_input::prelude::Action;
use bevy_enhanced_input::prelude::ActionOf;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_lagrange::OrbitCam;
use input_events::Start;

use super::constants::SPACESHIP_ACCELERATION;
use super::constants::SPACESHIP_ACCELERATION_MAX;
use super::constants::SPACESHIP_ACCELERATION_MIN;
use super::constants::SPACESHIP_MAX_SPEED;
use super::constants::SPACESHIP_MAX_SPEED_MAX;
use super::constants::SPACESHIP_MAX_SPEED_MIN;
use super::constants::SPACESHIP_ROTATION_SPEED;
use super::constants::SPACESHIP_ROTATION_SPEED_MAX;
use super::constants::SPACESHIP_ROTATION_SPEED_MIN;
use super::spaceship::ContinuousFire;
use super::spaceship::Spaceship;
use super::spaceship::SpaceshipSettings;
use crate::input::InspectSpaceshipControlSwitch;
use crate::input::ShipAccelerate;
use crate::input::ShipContinuousFire;
use crate::input::ShipControlsContext;
use crate::input::ShipTurnLeft;
use crate::input::ShipTurnRight;
use crate::orientation::CameraOrientation;
use crate::orientation::OrientationType;
use crate::schedule::InGameSet;
use crate::switches;
use crate::switches::Switch;

pub(super) struct SpaceshipControlPlugin;

impl Plugin for SpaceshipControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<SpaceshipControlSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectSpaceshipControl)),
        )
        .init_resource::<SpaceshipControlSettings>()
        .add_observer(on_toggle_continuous_fire_input)
        .add_systems(
            Update,
            spaceship_movement_controls.in_set(InGameSet::UserInput),
        );
        bind_action_switch!(
            app,
            InspectSpaceshipControlSwitch,
            SpaceshipControlInspectorEvent,
            Switch::InspectSpaceshipControl
        );
    }
}

event!(SpaceshipControlInspectorEvent);

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct SpaceshipControlSettings {
    #[inspector(
        min = SPACESHIP_ACCELERATION_MIN,
        max = SPACESHIP_ACCELERATION_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) acceleration:   f32,
    #[inspector(
        min = SPACESHIP_MAX_SPEED_MIN,
        max = SPACESHIP_MAX_SPEED_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) max_speed:      f32,
    #[inspector(
        min = SPACESHIP_ROTATION_SPEED_MIN,
        max = SPACESHIP_ROTATION_SPEED_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) rotation_speed: f32,
}

impl Default for SpaceshipControlSettings {
    fn default() -> Self {
        Self {
            acceleration:   SPACESHIP_ACCELERATION,
            rotation_speed: SPACESHIP_ROTATION_SPEED,
            max_speed:      SPACESHIP_MAX_SPEED,
        }
    }
}

type ShipAccelerateStateQuery<'w, 's> = Single<
    'w,
    's,
    &'static TriggerState,
    (
        With<Action<ShipAccelerate>>,
        With<ActionOf<ShipControlsContext>>,
    ),
>;

type ShipTurnLeftStateQuery<'w, 's> = Single<
    'w,
    's,
    &'static TriggerState,
    (
        With<Action<ShipTurnLeft>>,
        With<ActionOf<ShipControlsContext>>,
    ),
>;

type ShipTurnRightStateQuery<'w, 's> = Single<
    'w,
    's,
    &'static TriggerState,
    (
        With<Action<ShipTurnRight>>,
        With<ActionOf<ShipControlsContext>>,
    ),
>;

enum TurnDirection {
    Right,
    Left,
    Neutral,
}

impl TurnDirection {
    fn from_trigger_states(turn_right: TriggerState, turn_left: TriggerState) -> Self {
        match (
            turn_right != TriggerState::None,
            turn_left != TriggerState::None,
        ) {
            (true, _) => Self::Right,
            (false, true) => Self::Left,
            (false, false) => Self::Neutral,
        }
    }
}

fn spaceship_movement_controls(
    mut spaceship_query: Query<
        (&mut Transform, &mut LinearVelocity, &mut AngularVelocity),
        With<Spaceship>,
    >,
    camera_transform: Single<&Transform, (With<OrbitCam>, Without<Spaceship>)>,
    accelerate_state: ShipAccelerateStateQuery,
    turn_left_state: ShipTurnLeftStateQuery,
    turn_right_state: ShipTurnRightStateQuery,
    spaceship_settings: Res<SpaceshipSettings>,
    spaceship_control_settings: Res<SpaceshipControlSettings>,
    time: Res<Time>,
    camera_orientation: Res<CameraOrientation>,
) {
    // `spaceship_query.single_mut()` is valid because the scene has exactly one
    // `Spaceship`; the system does not need to iterate the query.
    if let Ok((mut spaceship_transform, mut linear_velocity, mut angular_velocity)) =
        spaceship_query.single_mut()
    {
        // `SpaceshipSettings::transform.scale` applies inspector changes to the
        // live `Spaceship` entity.
        spaceship_transform.scale = spaceship_settings.transform.scale;

        let delta_secs = time.delta_secs();
        let rotation_speed = spaceship_control_settings.rotation_speed;

        let accelerate = **accelerate_state != TriggerState::None;

        let mut target_angular_velocity =
            match TurnDirection::from_trigger_states(**turn_right_state, **turn_left_state) {
                TurnDirection::Right => rotation_speed,
                TurnDirection::Left => -rotation_speed,
                TurnDirection::Neutral => 0.0,
            };

        let camera_forward = camera_transform.forward();
        let facing_opposite = camera_forward.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.0;
        if facing_opposite {
            target_angular_velocity = -target_angular_velocity;
        }

        // `AngularVelocity` stays on the Z axis while
        // `target_angular_velocity` carries the turn input.
        angular_velocity.x = 0.0;
        angular_velocity.y = 0.0;
        angular_velocity.z = target_angular_velocity;

        let max_speed = spaceship_control_settings.max_speed;
        let acceleration = spaceship_control_settings.acceleration;

        if accelerate {
            apply_acceleration(
                &mut linear_velocity,
                spaceship_transform.forward().as_vec3(),
                acceleration,
                max_speed,
                delta_secs,
                &camera_orientation,
            );
        }
    }
}

fn apply_acceleration(
    linear_velocity: &mut LinearVelocity,
    direction: Vec3,
    acceleration: f32,
    max_speed: f32,
    delta_secs: f32,
    camera_orientation: &CameraOrientation,
) {
    let proposed_velocity = **linear_velocity + direction * (acceleration * delta_secs);
    let proposed_speed = proposed_velocity.length();

    if proposed_speed > max_speed {
        **linear_velocity = proposed_velocity.normalize() * max_speed;
    } else {
        **linear_velocity = proposed_velocity;
    }

    // Non-3D `CameraOrientation` modes constrain `LinearVelocity` to the XY plane.
    match camera_orientation.orientation_type {
        OrientationType::BehindSpaceship3D => (),
        _ => linear_velocity.z = 0.0,
    }
}

fn on_toggle_continuous_fire_input(
    _trigger: On<Start<ShipContinuousFire>>,
    mut commands: Commands,
) {
    commands.run_system_cached(toggle_continuous_fire_command);
}

/// Reusable on-demand command for toggling ship continuous fire mode.
fn toggle_continuous_fire_command(
    mut commands: Commands,
    spaceship: Single<(Entity, Option<&ContinuousFire>), With<Spaceship>>,
) {
    let (entity, continuous_fire) = *spaceship;
    if continuous_fire.is_some() {
        info!("removing continuous fire");
        commands.entity(entity).remove::<ContinuousFire>();
    } else {
        info!("adding continuous fire");
        commands.entity(entity).insert(ContinuousFire);
    }
}
