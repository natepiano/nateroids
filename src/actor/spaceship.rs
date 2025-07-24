use crate::{
    actor::{
        actor_spawner::spawn_actor,
        actor_template::SpaceshipConfig,
        spaceship_control::SpaceshipControl,
    },
    schedule::InGameSet,
    state::GameState,
};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Spaceship;

#[derive(Component, Default)]
pub struct ContinuousFire;

pub struct SpaceshipPlugin;
impl Plugin for SpaceshipPlugin {
    // make sure this is done after asset_loader has run
    fn build(&self, app: &mut App) {
        // we can enter InGame a couple of ways - when we do, spawn a spaceship
        app.add_systems(OnExit(GameState::Splash), spawn_spaceship)
            .add_systems(OnExit(GameState::GameOver), spawn_spaceship)
            // check if spaceship is destroyed...this will change the GameState
            .add_systems(Update, spaceship_destroyed.in_set(InGameSet::EntityUpdates));
    }
}

fn spawn_spaceship(mut commands: Commands, spaceship_config: Res<SpaceshipConfig>) {
    if !spaceship_config.0.spawnable {
        return;
    }

    spawn_actor(&mut commands, &spaceship_config.0, None, None)
        .insert(SpaceshipControl::generate_input_map())
        .insert(Spaceship);
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
