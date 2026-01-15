use avian3d::prelude::*;
use bevy::dev_tools::states::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::game_input::GameAction;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(GameState::InGame)),
                    restart_game.run_if(in_state(GameState::InGame)),
                    restart_with_splash.run_if(in_state(GameState::InGame)),
                    transition_to_in_game.run_if(in_state(GameState::GameOver)),
                ),
            )
            .add_systems(OnEnter(PauseState::Paused), physics_paused)
            .add_systems(OnEnter(PauseState::Playing), physics_playing)
            .add_systems(PostStartup, transition_to_splash_on_startup)
            .add_systems(Update, log_transitions::<GameState>);
    }
}

/// `GameState`'s for Nateroids
/// `PostStartup` transitions to Splash _after_ camera is spawned.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect, States)]
pub enum GameState {
    // Launch is the default to prevent OnEnter(Splash) from firing before camera exists. (is that
    // even possible? this is an old comment, maybe it's not actually true).
    //
    // For sure, without Launch, we could get spurious random bugs where just spawning a component
    // on an unrelated entity could cause the stars to flash and disappear. I don't understand
    // the timing/sequencing that causes this it's as if it's some kind of conflict between
    // running GameOver (which was our prior default) and Splash but in any case, this seems
    // to work for now so we'll go with it. Something about archetype restructing interfering
    // maybe. We proved that when we switched to spawning a Resource instead of a component,
    // the bug didn't surface. See: the camera/stars.rs system set scheduling.
    #[default]
    Launch,
    Splash,
    InGame,
    GameOver,
}

/// Pause state as a `SubState` of `GameState::InGame`.
/// Only exists when in `GameState::InGame`.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::InGame)]
pub enum PauseState {
    #[default]
    Playing,
    Paused,
}

fn toggle_pause(
    user_input: Res<ActionState<GameAction>>,
    mut next_state: ResMut<NextState<PauseState>>,
    state: Res<State<PauseState>>,
) {
    if user_input.just_pressed(&GameAction::Pause) {
        match state.get() {
            PauseState::Playing => next_state.set(PauseState::Paused),
            PauseState::Paused => next_state.set(PauseState::Playing),
        }
    }
}

fn restart_game(
    user_input: Res<ActionState<GameAction>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Quick restart flow (Cmd+Shift+N):
    // 1. InGame → GameOver: Stars regenerate, actors despawn
    // 2. GameOver → InGame: No star regeneration (stars from step 1 persist)
    // 3. Fresh game starts with stars already generated
    if user_input.just_pressed(&GameAction::RestartGame) {
        debug!("restart quick");
        next_state.set(GameState::GameOver);
    }
}

fn restart_with_splash(
    user_input: Res<ActionState<GameAction>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Full restart with splash flow (Cmd+Shift+S):
    // 1. InGame → Splash: Stars regenerate, actors despawn, splash timer resets
    // 2. Splash → InGame: No star regeneration (stars from step 1 persist)
    // 3. Game starts with stars that were generated during splash
    if user_input.just_pressed(&GameAction::RestartWithSplash) {
        debug!("restart with splash");
        next_state.set(GameState::Splash);
    }
}

fn transition_to_in_game(mut next_state: ResMut<NextState<GameState>>) {
    debug!("transitioning to InGame");
    next_state.set(GameState::InGame);
}

fn transition_to_splash_on_startup(mut next_state: ResMut<NextState<GameState>>) {
    debug!("transitioning to Splash on startup");
    next_state.set(GameState::Splash);
}

fn physics_paused(mut time: ResMut<Time<Physics>>) {
    debug!("pausing game and physics");
    time.pause();
}

fn physics_playing(mut time: ResMut<Time<Physics>>) {
    debug!("unpausing game and physics");
    time.unpause();
}
