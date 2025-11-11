use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use leafwing_input_manager::Actionlike;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::input_map::InputMap;
use leafwing_input_manager::plugin::InputManagerPlugin;
use strum::EnumIter;
use strum::IntoEnumIterator;

use super::actor_template::SpaceshipConfig;
use super::spaceship::ContinuousFire;
use super::spaceship::Spaceship;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::orientation::CameraOrientation;
use crate::orientation::OrientationType;
use crate::schedule::InGameSet;

pub struct SpaceshipControlPlugin;

impl Plugin for SpaceshipControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<SpaceshipControlConfig>::default()
                .run_if(toggle_active(false, GameAction::SpaceshipControlInspector)),
        )
        .init_resource::<SpaceshipControlConfig>()
        // spaceship will have input attached to it when spawning a spaceship
        .add_plugins(InputManagerPlugin::<SpaceshipControl>::default())
        .init_resource::<ActionState<SpaceshipControl>>()
        .insert_resource(SpaceshipControl::generate_input_map())
        .add_systems(
            FixedUpdate,
            (spaceship_movement_controls, toggle_continuous_fire)
                .chain()
                .in_set(InGameSet::UserInput),
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

// This is the list of "things I want the spaceship to be able to do based on
// input"
#[derive(Actionlike, EnumIter, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum SpaceshipControl {
    Accelerate,
    ContinuousFire,
    Fire,
    TurnLeft,
    TurnRight,
}

// #todo handle clash-strategy across InstantMap instances https://github.com/Leafwing-Studios/leafwing-input-manager/issues/617
impl SpaceshipControl {
    pub fn generate_input_map() -> InputMap<Self> {
        Self::iter().fold(InputMap::default(), |input_map, action| match action {
            Self::Accelerate => input_map
                .with(action, KeyCode::KeyW)
                .with(action, KeyCode::ArrowUp),
            Self::TurnLeft => input_map
                .with(action, KeyCode::KeyA)
                .with(action, KeyCode::ArrowLeft),
            Self::TurnRight => input_map
                .with(action, KeyCode::KeyD)
                .with(action, KeyCode::ArrowRight),
            Self::Fire => input_map.with(action, KeyCode::Space),
            Self::ContinuousFire => input_map.with(action, KeyCode::KeyF),
        })
    }
}

fn spaceship_movement_controls(
    mut q_spaceship: Query<
        (&mut Transform, &mut LinearVelocity, &mut AngularVelocity),
        With<Spaceship>,
    >,
    camera_transform: Single<&Transform, (With<PanOrbitCamera>, Without<Spaceship>)>,
    controls: Single<&ActionState<SpaceshipControl>>,
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
        let mut target_angular_velocity = 0.0;
        if controls.pressed(&SpaceshipControl::TurnRight) {
            target_angular_velocity = rotation_speed;
        } else if controls.pressed(&SpaceshipControl::TurnLeft) {
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

        if controls.pressed(&SpaceshipControl::Accelerate) {
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

// todo: how can i avoid setting this allow - i'm guessing a system param would
// be just as problematic
#[allow(clippy::type_complexity)]
fn toggle_continuous_fire(
    mut commands: Commands,
    q_spaceship: Query<
        (
            Entity,
            &ActionState<SpaceshipControl>,
            Option<&ContinuousFire>,
        ),
        With<Spaceship>,
    >,
) {
    if let Ok((entity, control, continuous)) = q_spaceship.single()
        && control.just_pressed(&SpaceshipControl::ContinuousFire)
    {
        if continuous.is_some() {
            println!("removing continuous");
            commands.entity(entity).remove::<ContinuousFire>();
        } else {
            println!("adding continuous");
            commands.entity(entity).insert(ContinuousFire);
        }
    }
}
