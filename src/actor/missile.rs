use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::actor::Teleporter;
use crate::actor::aabb::Aabb;
use crate::actor::actor_spawner::ActorConfig;
use crate::actor::actor_spawner::LOCKED_AXES_2D;
use crate::actor::actor_spawner::ZERO_GRAVITY;
use crate::actor::actor_spawner::create_spawn_timer;
use crate::actor::actor_spawner::spawn_actor;
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
        app.add_observer(initialize_missile_position)
            .add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
            .add_systems(Update, missile_movement.in_set(InGameSet::EntityUpdates));
    }
}

// todo: #rustquestion - how can i make it so that new has to be used and
// DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Missile;

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
#[reflect(Component)]
pub struct MissilePosition {
    pub total_distance: f32,
    pub traveled_distance: f32,
    remaining_distance: f32,
    pub last_position: Option<Vec3>,
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

fn initialize_missile_position(
    add: On<Add, Missile>,
    mut commands: Commands,
    boundary_config: Res<Boundary>,
) {
    let missile_position = MissilePosition::new(boundary_config.max_missile_distance());
    commands.entity(add.entity).insert(missile_position);
}

fn fire_missile(
    mut commands: Commands,
    q_spaceship: Query<
        (&Transform, &LinearVelocity, &Aabb, Option<&ContinuousFire>),
        With<Spaceship>,
    >,
    mut missile_config: ResMut<MissileConfig>,
    fire_button: Single<&ActionState<SpaceshipControl>>,
    time: Res<Time>,
) {
    let Ok((spaceship_transform, spaceship_linear_velocity, aabb, continuous_fire_enabled)) =
        q_spaceship.single()
    else {
        return;
    };

    if !should_fire(
        continuous_fire_enabled,
        &mut missile_config.0,
        time,
        fire_button,
    ) {
        return;
    }

    let parent = (spaceship_transform, spaceship_linear_velocity, aabb);

    spawn_actor(&mut commands, &missile_config.0, None, Some(parent));

    // Recreate timer from spawn_timer_seconds to pick up inspector changes
    missile_config.0.spawn_timer = create_spawn_timer(missile_config.0.spawn_timer_seconds);
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
