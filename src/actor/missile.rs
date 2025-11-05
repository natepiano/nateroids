use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::actor::Teleporter;
use crate::actor::aabb::Aabb;
use crate::actor::actor_spawner::ActorConfig;
use crate::actor::actor_spawner::spawn_actor;
use crate::actor::actor_template::MissileConfig;
use crate::actor::spaceship::ContinuousFire;
use crate::actor::spaceship::Spaceship;
use crate::actor::spaceship_control::SpaceshipControl;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;

pub struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, fire_missile.in_set(InGameSet::UserInput))
            .add_systems(Update, missile_movement.in_set(InGameSet::EntityUpdates));
    }
}

// todo: #rustquestion - how can i make it so that new has to be used and
// DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Copy, Clone, Component, Debug)]
pub struct Missile {
    // velocity:               Vec3,
    pub total_distance:     f32,
    pub traveled_distance:  f32,
    remaining_distance:     f32,
    pub last_position:      Option<Vec3>,
    last_teleport_position: Option<Vec3>, // Add this field
}

impl Missile {
    fn new(total_distance: f32) -> Self {
        Missile {
            // velocity,
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
    q_input_map: Query<&ActionState<SpaceshipControl>>,
) -> bool {
    if !missile_config.spawnable {
        return false;
    }

    if let Ok(action_state) = q_input_map.single() {
        if continuous_fire.is_some() {
            // We know the timer exists, so we can safely unwrap it
            let timer = missile_config.spawn_timer.as_mut().expect(
                "configure missile spawn timer here: impl Default for
InitialEnsembleConfig",
            );
            timer.tick(time.delta());
            if !timer.just_finished() {
                return false;
            }
            action_state.pressed(&SpaceshipControl::Fire)
        } else {
            action_state.just_pressed(&SpaceshipControl::Fire)
        }
    } else {
        false
    }
}

// todo: #bevyquestion - in an object oriented world i think of attaching fire
// as a method to                       the spaceship - but there's a lot of
// missile logic so i have it setup in missile                       so should i
// have a simple fire method in method in spaceship that in turn calls this
//                       fn or is having it here fine?
fn fire_missile(
    mut commands: Commands,
    q_input_map: Query<&ActionState<SpaceshipControl>>,
    q_spaceship: Query<
        (&Transform, &LinearVelocity, &Aabb, Option<&ContinuousFire>),
        With<Spaceship>,
    >,
    boundary_config: Res<Boundary>,
    mut missile_config: ResMut<MissileConfig>,
    time: Res<Time>,
) {
    let Ok((spaceship_transform, spaceship_linear_velocity, aabb, continuous_fire)) =
        q_spaceship.single()
    else {
        return;
    };

    if !should_fire(continuous_fire, &mut missile_config.0, time, q_input_map) {
        return;
    }

    let missile = Missile::new(boundary_config.max_missile_distance());

    spawn_actor(
        &mut commands,
        &missile_config.0,
        None,
        Some((spaceship_transform, spaceship_linear_velocity, aabb)),
    )
    .insert(missile);
}

/// we update missile movement so that it can be despawned after it has traveled
/// its total distance
fn missile_movement(mut query: Query<(&Transform, &mut Missile, &Teleporter)>) {
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
