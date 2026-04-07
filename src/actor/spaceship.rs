use avian3d::prelude::*;
use bevy::prelude::*;

use super::Teleporter;
use super::actor_settings;
use super::actor_settings::Spawnability;
use super::actor_template::SpaceshipSettings;
use super::constants::GLTF_ROTATION_X;
use super::constants::LOCKED_AXES_SPACESHIP;
use super::constants::SPACESHIP_FORWARD_EPSILON;
use super::constants::SPACESHIP_TILT_THRESHOLD;
use crate::input;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;
use crate::splash::SplashText;
use crate::state::GameState;
use crate::state::PauseState;

/// Returns the default `Spaceship` rotation: model correction (90° around X)
fn default_spaceship_rotation() -> Quat { Quat::from_rotation_x(GLTF_ROTATION_X) }

pub(super) struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    // make sure this is done after `asset_loader` has run
    fn build(&self, app: &mut App) {
        // Spawn `Spaceship` when entering `PauseState::Playing` (game start or unpause)
        app.add_observer(initialize_spaceship)
            .add_observer(spawn_after_splash_text_removed)
            .add_observer(on_spaceship_removed)
            .add_systems(OnEnter(PauseState::Playing), spawn_spaceship_if_needed)
            .add_systems(
                FixedUpdate,
                enforce_spaceship_2d_rotation
                    .after(PhysicsSystems::StepSimulation)
                    .in_set(InGameSet::EntityUpdates),
            );
    }
}

#[derive(Component, Default)]
pub(super) struct ContinuousFire;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    LockedAxes = LOCKED_AXES_SPACESHIP,
    LinearVelocity::ZERO,
    AngularVelocity::ZERO,
)]
pub struct Spaceship;

/// Observer that spawns the `Spaceship` when `SplashText` is removed
fn spawn_after_splash_text_removed(
    _trigger: On<Remove, SplashText>,
    commands: Commands,
    spaceship_settings: Res<SpaceshipSettings>,
) {
    spawn_spaceship(commands, spaceship_settings);
}

/// Spawns a `Spaceship` only if one doesn't already exist
fn spawn_spaceship_if_needed(
    commands: Commands,
    spaceship_settings: Res<SpaceshipSettings>,
    query: Query<(), With<Spaceship>>,
) {
    // Only spawn if no spaceship exists (e.g., coming from GameOver)
    if query.is_empty() {
        spawn_spaceship(commands, spaceship_settings);
    }
}

fn spawn_spaceship(mut commands: Commands, spaceship_settings: Res<SpaceshipSettings>) {
    if spaceship_settings.spawnability == Spawnability::Disabled {
        return;
    }

    commands.spawn((Spaceship, ContinuousFire, Name::new("Spaceship")));
}

fn initialize_spaceship(
    spaceship: On<Add, Spaceship>,
    mut commands: Commands,
    mut spaceship_settings: ResMut<SpaceshipSettings>,
) {
    commands
        .entity(spaceship.entity)
        .insert(spaceship_settings.transform)
        // Ship controls now come from enhanced-input on the spaceship context entity.
        .insert(input::ship_controls_input_bundle());

    actor_settings::insert_configured_components(
        &mut commands,
        &mut spaceship_settings.actor_settings,
        spaceship.entity,
    );
}

fn on_spaceship_removed(
    trigger: On<Remove, Spaceship>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    info!("spaceship destroyed: entity {:?}", trigger.entity);
    next_state.set(GameState::GameOver);
}

/// Enforce strict 2D rotation by zeroing X/Y angular velocity and correcting transform if tilted
/// Keeps the `Spaceship` flat in the XY plane (up vector should point in +Z)
fn enforce_spaceship_2d_rotation(
    mut spaceship: Query<(&mut Transform, &mut AngularVelocity), With<Spaceship>>,
) {
    if let Ok((mut transform, mut angular_velocity)) = spaceship.single_mut() {
        // Always zero angular velocity on X/Y axes to prevent future off-axis rotation
        angular_velocity.x = 0.0;
        angular_velocity.y = 0.0;

        // Check if rotation quaternion is valid (not NaN or denormalized)
        if !transform.rotation.is_finite() || !transform.rotation.is_normalized() {
            warn!("Spaceship rotation became invalid (NaN or denormalized), resetting to default");
            transform.rotation = default_spaceship_rotation();
            return;
        }

        // Check if spaceship is tilted by looking at up vector
        // After +90° X rotation, up should point in +Z (0, 0, 1)
        let up = transform.up();

        // Guard against NaN from corrupted transform
        if !up.is_finite() {
            warn!("Spaceship up vector is NaN, resetting rotation");
            transform.rotation = default_spaceship_rotation();
            return;
        }

        let tilt_amount = up.x.hypot(up.y);

        if tilt_amount > SPACESHIP_TILT_THRESHOLD {
            // Get current forward direction and project to XY plane
            let forward = transform.forward();

            // Guard against NaN
            if !forward.is_finite() {
                warn!("Spaceship forward vector is NaN, resetting rotation");
                transform.rotation = default_spaceship_rotation();
                return;
            }

            let forward_2d = Vec3::new(forward.x, forward.y, 0.0);
            let forward_len_sq = forward_2d.length_squared();

            if forward_len_sq > SPACESHIP_FORWARD_EPSILON {
                let forward_2d_normalized = forward_2d / forward_len_sq.sqrt();

                // Calculate angle in XY plane (from +Y axis)
                let z_angle = forward_2d_normalized.y.atan2(forward_2d_normalized.x)
                    - std::f32::consts::FRAC_PI_2;

                // Guard against NaN from atan2
                if z_angle.is_finite() {
                    // Rebuild rotation: model correction + gameplay rotation
                    let new_rotation =
                        Quat::from_rotation_x(GLTF_ROTATION_X) * Quat::from_rotation_z(z_angle);

                    // Normalize to prevent drift over many frames
                    transform.rotation = new_rotation.normalize();
                }
            }
        }
    }
}
