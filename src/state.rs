use crate::global_input::GlobalAction;
use bevy::{
    dev_tools::states::*,
    prelude::*,
};
use bevy_rapier3d::plugin::RapierConfiguration;
use leafwing_input_manager::prelude::ActionState;

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
                    transition_to_in_game.run_if(in_state(GameState::GameOver)),
                ),
            )
            .add_systems(OnEnter(IsPaused::Paused), pause_rapier)
            .add_systems(OnEnter(IsPaused::NotPaused), unpause_rapier)
            .add_systems(Update, log_transitions::<GameState>);
    }

    fn name(&self) -> &str { "state plugin" }
}

// splash is the default so bevy will automatically enter this state
// we catch that in splash.rs to do the splash screen
//
// in state/computed_states bevy example, they have a tutorial state that is
// active/inactive that is computed and shows tutorial text while in various
// GameState modes
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Reflect, States)]
pub enum GameState {
    #[default]
    Splash,
    InGame {
        paused:     bool,
        inspecting: bool,
    },
    GameOver,
}

// as PlayingGame is a computed state that covers paused - we wanted it to have
// a different name than InGame.  Playing is "true" whether we are paused or not
// in the future, as in the bevy computed_states example - we might add other
// "modes" other than paused. The example has turbo mode - which is global, just
// like paused so that might be useful to have around
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct PlayingGame;

impl ComputedStates for PlayingGame {
    // Our computed state depends on `AppState`, so we need to specify it as the
    // SourceStates type.
    type SourceStates = GameState;

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
    user_input: Res<ActionState<GlobalAction>>,
    mut next_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if user_input.just_pressed(&GlobalAction::Pause) {
        if let GameState::InGame { paused, inspecting } = state.get() {
            next_state.set(GameState::InGame {
                paused:     !*paused,
                inspecting: *inspecting,
            });
        }
    }
}

fn transition_to_in_game(mut next_state: ResMut<NextState<GameState>>) {
    println!("Transitioning to InGame");
    next_state.set(GameState::InGame {
        paused:     false,
        inspecting: false,
    });
}

fn pause_rapier(mut rapier_config_query: Query<&mut RapierConfiguration>) {
    if let Ok(mut rapier_config) = rapier_config_query.single_mut() {
        println!("pausing game and physics");
        rapier_config.physics_pipeline_active = false;
    } else {
        error!("Error: Unable to find the RapierConfiguration component.");
    }
}

fn unpause_rapier(mut rapier_config_query: Query<&mut RapierConfiguration>) {
    if let Ok(mut rapier_config) = rapier_config_query.single_mut() {
        println!("unpausing game and physics");
        rapier_config.physics_pipeline_active = true;
    } else {
        error!("Error: Unable to find the RapierConfiguration component.");
    }
}
