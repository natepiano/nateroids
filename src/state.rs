#![allow(clippy::used_underscore_binding)] // False positive on GameState::InGame fields

use avian3d::prelude::*;
use bevy::dev_tools::states::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::game_input::GameAction;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_computed_state::<PlayingGame>()
            .add_computed_state::<IsPaused>()
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(PlayingGame)),
                    restart_game.run_if(in_state(PlayingGame)),
                    restart_with_splash.run_if(in_state(PlayingGame)),
                    transition_to_in_game.run_if(in_state(GameState::GameOver)),
                ),
            )
            .add_systems(OnEnter(IsPaused::Paused), pause_physics)
            .add_systems(OnEnter(IsPaused::NotPaused), unpause_physics)
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
    InGame {
        paused:     bool,
    },
    GameOver,
}

// as PlayingGame is a computed state that covers paused - we wanted it to have
// a different name than InGame.  Playing is "true" whether we are paused or not
// in the future, as in the bevy computed_states example - we might add other
// "modes" other than paused. The example has turbo mode - which is global, just
// like paused so that might be useful to have around
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct PlayingGame;

impl ComputedStates for PlayingGame {
    // Our computed state depends on `AppState`, so we need to specify it as the
    // SourceStates type.
    type SourceStates = GameState;

    // Bevy 0.18: Prevent OnEnter/OnExit from firing when computed value is unchanged.
    // Without this, pausing (InGame { paused: false } → InGame { paused: true }) would
    // trigger OnExit(PlayingGame) even though both compute to Some(PlayingGame).
    const ALLOW_SAME_STATE_TRANSITIONS: bool = false;

    // The compute function takes in the `SourceStates`
    fn compute(sources: GameState) -> Option<Self> {
        // You might notice that InGame has no values - instead, in this case, the
        // `State<InGame>` resource only exists if the `compute` function would
        // return `Some` - so only when we are in game.
        match sources {
            // No matter what the value of `paused` or `turbo` is, we're still in the game rather
            // than a menu
            GameState::InGame { .. } => Some(Self),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum IsPaused {
    NotPaused,
    Paused,
}

impl ComputedStates for IsPaused {
    type SourceStates = GameState;

    fn compute(sources: GameState) -> Option<Self> {
        // Here we convert from our [`GameState`] to all potential [`IsPaused`]
        // versions.
        match sources {
            GameState::InGame { paused: true, .. } => Some(Self::Paused),
            GameState::InGame { paused: false, .. } => Some(Self::NotPaused),
            // If `GameState` is not `InGame`, pausing is meaningless, and so we set it to `None`.
            _ => None,
        }
    }
}

fn toggle_pause(
    user_input: Res<ActionState<GameAction>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if user_input.just_pressed(&GameAction::Pause)
        && let GameState::InGame { paused,  } = state.get()
    {
        next_state.set(GameState::InGame {
            paused:     !*paused,
        });
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
    next_state.set(GameState::InGame {
        paused:     false,
    });
}

fn transition_to_splash_on_startup(mut next_state: ResMut<NextState<GameState>>) {
    debug!("transitioning to Splash on startup");
    next_state.set(GameState::Splash);
}

fn pause_physics(mut time: ResMut<Time<Physics>>) {
    debug!("pausing game and physics");
    time.pause();
}

fn unpause_physics(mut time: ResMut<Time<Physics>>) {
    debug!("unpausing game and physics");
    time.unpause();
}
