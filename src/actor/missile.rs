use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::actor::Teleporter;
use crate::actor::actor_config::ActorConfig;
use crate::actor::actor_config::LOCKED_AXES_2D;
use crate::actor::actor_config::insert_configured_components;
use crate::actor::actor_template::MissileConfig;
use crate::actor::spaceship::ContinuousFire;
use crate::actor::spaceship::Spaceship;
use crate::actor::spaceship_control::SpaceshipControl;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initialize_missile)
            .add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
            .add_systems(Update, missile_movement.in_set(InGameSet::EntityUpdates));
    }
}

// todo: #rustquestion - how can i make it so that new has to be used and
// DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Missile;

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
#[reflect(Component)]
pub struct MissilePosition {
    pub total_distance:     f32,
    pub traveled_distance:  f32,
    remaining_distance:     f32,
    pub last_position:      Option<Vec3>,
    last_teleport_position: Option<Vec3>, // Add this field
}

impl MissilePosition {
    pub fn new(total_distance: f32) -> Self {
        MissilePosition {
            total_distance,
            traveled_distance: 0.,
            remaining_distance: 0.,
            last_position: None,
            last_teleport_position: None,
        }
    }
}

/// Logic to handle whether we're in continuous fire mode or just regular fire
/// mode if continuous we want to make sure that enough time has passed and that
/// we're holding down the fire button
fn should_fire(
    continuous_fire: Option<&ContinuousFire>,
    missile_config: &mut ActorConfig,
    time: Res<Time>,
    fire_button: Single<&ActionState<SpaceshipControl>>,
) -> bool {
    if !missile_config.spawnable {
        return false;
    }

    if continuous_fire.is_some() {
        // We know the timer exists, so we can safely unwrap it
        let timer = missile_config
            .spawn_timer
            .as_mut()
            .expect("configure missile spawn timer here: impl Default for InitialEnsembleConfig");
        timer.tick(time.delta());
        if !timer.just_finished() {
            return false;
        }

        fire_button.pressed(&SpaceshipControl::Fire)
    } else {
        fire_button.just_pressed(&SpaceshipControl::Fire)
    }
}

fn initialize_missile(
    missile: On<Add, Missile>,
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut config: ResMut<MissileConfig>,
    transform_and_linvel: Single<(&Transform, &LinearVelocity), With<Spaceship>>,
) {
    let missile_position = MissilePosition::new(boundary.max_missile_distance());

    let (spaceship_transform, spaceship_velocity) = *transform_and_linvel;

    let transform = initialize_transform(spaceship_transform, &config);

    // Calculate velocity: forward direction * base_velocity + spaceship velocity
    let (linear_velocity, angular_velocity) = calculate_missile_velocity(
        spaceship_transform,
        spaceship_velocity,
        config.base_velocity,
    );

    commands
        .entity(missile.entity)
        .insert(missile_position)
        .insert(transform)
        .insert(linear_velocity)
        .insert(angular_velocity);

    insert_configured_components(&mut commands, &mut config.actor_config, missile.entity);
}

fn initialize_transform(
    spaceship_transform: &Transform,
    missile_config: &MissileConfig,
) -> Transform {
    // Calculate transform and velocity from spaceship position
    let forward = -spaceship_transform.forward();
    let spawn_position =
        spaceship_transform.translation + forward * missile_config.forward_distance_scalar;

    // Combine rotations: spaceship rotation * missile config rotation
    let combined_rotation =
        spaceship_transform.rotation * missile_config.actor_config.transform.rotation;

    Transform::from_translation(spawn_position)
        .with_rotation(combined_rotation)
        .with_scale(missile_config.actor_config.transform.scale)
}

fn fire_missile(
    mut commands: Commands,
    q_spaceship: Query<Option<&ContinuousFire>, With<Spaceship>>,
    mut missile_config: ResMut<MissileConfig>,
    fire_button: Single<&ActionState<SpaceshipControl>>,
    time: Res<Time>,
) {
    let Ok(continuous_fire_enabled) = q_spaceship.single() else {
        return;
    };

    if !should_fire(
        continuous_fire_enabled,
        &mut missile_config,
        time,
        fire_button,
    ) {
        return;
    }

    commands.spawn((Missile, Name::new("Missile")));
}

/// we update missile movement so that it can be despawned after it has traveled
/// its total distance
fn missile_movement(mut query: Query<(&Transform, &mut MissilePosition, &Teleporter)>) {
    for (transform, mut missile, teleporter) in query.iter_mut() {
        let current_position = transform.translation;

        if let Some(last_position) = missile.last_position {
            // Calculate the distance traveled since the last update
            let distance_traveled = if teleporter.just_teleported {
                0.0
            } else {
                last_position.distance(current_position)
            };

            // Update the total traveled distance
            missile.traveled_distance += distance_traveled;
            missile.remaining_distance = missile.total_distance - missile.traveled_distance;

            // Update the last teleport position if the missile wrapped
            if teleporter.just_teleported {
                missile.last_teleport_position = Some(current_position);
            }
        }

        // Always update last_position
        missile.last_position = Some(current_position);
    }
}

fn calculate_missile_velocity(
    spaceship_transform: &Transform,
    spaceship_velocity: &LinearVelocity,
    base_velocity: f32,
) -> (LinearVelocity, AngularVelocity) {
    let forward = -spaceship_transform.forward();
    let mut velocity = forward * base_velocity;
    velocity += **spaceship_velocity;
    (LinearVelocity(velocity), AngularVelocity::ZERO)
}
