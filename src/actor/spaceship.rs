use avian3d::prelude::*;
use bevy::prelude::*;

use super::Teleporter;
use super::actor_config::GLTF_ROTATION_X;
use super::actor_config::LOCKED_AXES_SPACESHIP;
use super::actor_config::insert_configured_components;
use super::actor_template::SpaceshipConfig;
use super::spaceship_control::SpaceshipControl;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;
use crate::splash::SplashText;
use crate::state::GameState;

/// Returns the default spaceship rotation: model correction (90° around X)
fn default_spaceship_rotation() -> Quat { Quat::from_rotation_x(GLTF_ROTATION_X) }

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        // we can enter InGame a couple of ways - when we do, spawn a spaceship
        app.add_observer(initialize_spaceship)
            .add_observer(spawn_after_splash_text_removed)
            .add_systems(
                OnEnter(GameState::InGame {
                    paused:     false,
                    inspecting: false,
                }),
                spawn_spaceship_if_needed,
            )
            // check if spaceship is destroyed...this will change the GameState
            .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates))
            .add_systems(
                FixedUpdate,
                enforce_spaceship_2d_rotation
                    .after(PhysicsSystems::StepSimulation)
                    .in_set(InGameSet::EntityUpdates),
            );
    }
}

#[derive(Component, Default)]
pub struct ContinuousFire;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    LockedAxes = LOCKED_AXES_SPACESHIP,
    LinearVelocity::ZERO,
    AngularVelocity::ZERO,
)]
pub struct Spaceship;

/// Observer that spawns the spaceship when splash text is removed
fn spawn_after_splash_text_removed(
    _trigger: On<Remove, SplashText>,
    commands: Commands,
    spaceship_config: Res<SpaceshipConfig>,
) {
    spawn_spaceship(commands, spaceship_config);
}

/// Spawns a spaceship only if one doesn't already exist
fn spawn_spaceship_if_needed(
    commands: Commands,
    spaceship_config: Res<SpaceshipConfig>,
    query: Query<(), With<Spaceship>>,
) {
    // Only spawn if no spaceship exists (e.g., coming from GameOver)
    if query.is_empty() {
        spawn_spaceship(commands, spaceship_config);
    }
}

fn spawn_spaceship(mut commands: Commands, spaceship_config: Res<SpaceshipConfig>) {
    if !spaceship_config.spawnable {
        return;
    }

    commands.spawn((Spaceship, ContinuousFire, Name::new("Spaceship")));
}

fn initialize_spaceship(
    spaceship: On<Add, Spaceship>,
    mut commands: Commands,
    mut spaceship_config: ResMut<SpaceshipConfig>,
) {
    commands
        .entity(spaceship.entity)
        .insert(spaceship_config.transform)
        .insert(SpaceshipControl::generate_input_map());

    insert_configured_components(
        &mut commands,
        &mut spaceship_config.actor_config,
        spaceship.entity,
    );
}

// check if spaceship exists or not - query if get_single()
// there should only be one - if it returns an error then the
// spaceship doesn't exist
fn spaceship_destroyed(
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<Spaceship>>,
    state: Res<State<GameState>>,
) {
    if query.single().is_err() {
        info!(
            "spaceship destroyed: {:?}, count {:?}",
            state,
            query.iter().count()
        );
        next_state.set(GameState::GameOver);
    }
}

/// Enforce strict 2D rotation by zeroing X/Y angular velocity and correcting transform if tilted
/// Keeps the spaceship flat in the XY plane (up vector should point in +Z)
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

        // Only correct transform if significantly tilted (threshold: ~5 degrees)
        if tilt_amount > 0.087 {
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

            if forward_len_sq > 0.0001 {
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
