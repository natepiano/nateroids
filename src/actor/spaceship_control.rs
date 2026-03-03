use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::action::TriggerState;
use bevy_enhanced_input::action::events as input_events;
use bevy_enhanced_input::prelude::Action;
use bevy_enhanced_input::prelude::ActionOf;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;

use super::actor_template::SpaceshipConfig;
use super::spaceship::ContinuousFire;
use super::spaceship::Spaceship;
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
use crate::switches::Switches;

event!(SpaceshipControlInspectorEvent);

pub struct SpaceshipControlPlugin;

impl Plugin for SpaceshipControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<SpaceshipControlConfig>::default()
                .run_if(switches::is_switch_on(Switch::InspectSpaceshipControl)),
        )
        .init_resource::<SpaceshipControlConfig>()
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

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct SpaceshipControlConfig {
    #[inspector(min = 30., max = 300.0, display = NumberDisplay::Slider)]
    pub acceleration:   f32,
    #[inspector(min = 50., max = 300.0, display = NumberDisplay::Slider)]
    pub max_speed:      f32,
    #[inspector(min = 1.0, max = 10.0, display = NumberDisplay::Slider)]
    pub rotation_speed: f32,
}

impl Default for SpaceshipControlConfig {
    fn default() -> Self {
        Self {
            acceleration:   60.,
            rotation_speed: 5.0,
            max_speed:      80.,
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

fn spaceship_movement_controls(
    mut q_spaceship: Query<
        (&mut Transform, &mut LinearVelocity, &mut AngularVelocity),
        With<Spaceship>,
    >,
    camera_transform: Single<&Transform, (With<PanOrbitCamera>, Without<Spaceship>)>,
    accelerate_state: ShipAccelerateStateQuery,
    turn_left_state: ShipTurnLeftStateQuery,
    turn_right_state: ShipTurnRightStateQuery,
    spaceship_config: Res<SpaceshipConfig>,
    movement_config: Res<SpaceshipControlConfig>,
    time: Res<Time>,
    orientation_mode: Res<CameraOrientation>,
) {
    // we can use this because there is only exactly one spaceship - so we're not
    // looping over the query
    if let Ok((mut spaceship_transform, mut linear_velocity, mut angular_velocity)) =
        q_spaceship.single_mut()
    {
        // dynamically update from inspector while game is running to change size
        spaceship_transform.scale = spaceship_config.transform.scale;

        let delta_seconds = time.delta_secs();
        let rotation_speed = movement_config.rotation_speed;

        // Set angular velocity based on input
        let turn_right = **turn_right_state != TriggerState::None;
        let turn_left = **turn_left_state != TriggerState::None;
        let accelerate = **accelerate_state != TriggerState::None;

        let mut target_angular_velocity = 0.0;
        if turn_right {
            target_angular_velocity = rotation_speed;
        } else if turn_left {
            target_angular_velocity = -rotation_speed;
        }

        // Flip rotation direction if camera is facing opposite
        let camera_forward = camera_transform.forward();
        let facing_opposite = camera_forward.dot(Vec3::new(0.0, 0.0, -1.0)) > 0.0;
        if facing_opposite {
            target_angular_velocity = -target_angular_velocity;
        }

        // Apply angular velocity through physics engine
        // Explicitly enforce 2D rotation by zeroing X/Y components
        angular_velocity.x = 0.0;
        angular_velocity.y = 0.0;
        angular_velocity.z = target_angular_velocity;

        let max_speed = movement_config.max_speed;
        let accel = movement_config.acceleration;

        if accelerate {
            apply_acceleration(
                &mut linear_velocity,
                spaceship_transform.forward().as_vec3(),
                accel,
                max_speed,
                delta_seconds,
                orientation_mode,
            );
        }
    }
}

fn apply_acceleration(
    linear_velocity: &mut LinearVelocity,
    direction: Vec3,
    acceleration: f32,
    max_speed: f32,
    delta_seconds: f32,
    orientation: Res<CameraOrientation>,
) {
    let proposed_velocity = **linear_velocity + direction * (acceleration * delta_seconds);
    let proposed_speed = proposed_velocity.length();

    // Ensure we're not exceeding max velocity
    if proposed_speed > max_speed {
        **linear_velocity = proposed_velocity.normalize() * max_speed;
    } else {
        **linear_velocity = proposed_velocity;
    }

    //todo: #handl3d
    match orientation.orientation {
        // in 3d we can accelerate in all dirs
        OrientationType::BehindSpaceship3D => (),
        _ => linear_velocity.z = 0.0, // Force the `z` value of linear_velocity to be 0
    }
}

fn on_toggle_continuous_fire_input(
    _trigger: On<input_events::Start<ShipContinuousFire>>,
    mut commands: Commands,
) {
    commands.run_system_cached(toggle_continuous_fire_command);
}

/// Reusable on-demand command for toggling ship continuous fire mode.
fn toggle_continuous_fire_command(
    mut commands: Commands,
    spaceship: Single<(Entity, Option<&ContinuousFire>), With<Spaceship>>,
) {
    let (entity, continuous) = *spaceship;
    if continuous.is_some() {
        info!("removing continuous");
        commands.entity(entity).remove::<ContinuousFire>();
    } else {
        info!("adding continuous");
        commands.entity(entity).insert(ContinuousFire);
    }
}
