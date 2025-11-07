use avian3d::prelude::*;
use bevy::prelude::*;

use super::Teleporter;
use super::actor_spawner::LOCKED_AXES_SPACESHIP;
use super::actor_spawner::insert_configured_components;
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
        app.add_observer(initialize_spaceship)
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
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    LockedAxes = LOCKED_AXES_SPACESHIP,
    LinearVelocity::ZERO,
    AngularVelocity::ZERO,
)]
pub struct Spaceship;

fn spawn_spaceship(mut commands: Commands, spaceship_config: Res<SpaceshipConfig>) {
    if !spaceship_config.spawnable {
        return;
    }
    commands.spawn((Spaceship, Name::new("Spaceship")));
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

    insert_configured_components(&mut commands, &mut spaceship_config, spaceship.entity);
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
