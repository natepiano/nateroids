use avian3d::prelude::*;
use bevy::prelude::*;

use super::Teleporter;
use super::actor_spawner::LOCKED_AXES_SPACESHIP;
use super::actor_spawner::ZERO_GRAVITY;
use super::actor_spawner::spawn_actor;
use super::actor_template::SpaceshipConfig;
use super::spaceship_control::SpaceshipControl;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;
use crate::state::GameState;

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        // we can enter InGame a couple of ways - when we do, spawn a spaceship
        app.add_observer(initialize_spaceship_input)
            .add_systems(OnExit(GameState::Splash), spawn_spaceship)
            .add_systems(OnExit(GameState::GameOver), spawn_spaceship)
            // check if spaceship is destroyed...this will change the GameState
            .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
    }
}

#[derive(Component, Default)]
pub struct ContinuousFire;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,
    LockedAxes = LOCKED_AXES_SPACESHIP
)]
pub struct Spaceship;

fn spawn_spaceship(mut commands: Commands, spaceship_config: Res<SpaceshipConfig>) {
    if !spaceship_config.0.spawnable {
        return;
    }

    spawn_actor(&mut commands, &spaceship_config.0, None, None);
}

fn initialize_spaceship_input(spaceship: On<Add, Spaceship>, mut commands: Commands) {
    commands
        .entity(spaceship.entity)
        .insert(SpaceshipControl::generate_input_map());
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
        println!(
            "spaceship destroyed: {:?}, count {:?}",
            state,
            query.iter().count()
        );
        next_state.set(GameState::GameOver);
    }
}
