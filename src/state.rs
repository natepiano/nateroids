use avian3d::prelude::*;
use bevy::dev_tools::states::*;
use bevy::prelude::*;
use bevy_enhanced_input::action::events as input_events;

use crate::input::PauseToggle;
use crate::input::RestartGameShortcut;
use crate::input::RestartWithSplashShortcut;

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<PauseState>()
            .add_observer(on_pause_input)
            .add_observer(on_restart_game_input)
            .add_observer(on_restart_with_splash_input)
            .add_observer(on_pause_event)
            .add_observer(on_restart_game_event)
            .add_observer(on_restart_with_splash_event)
            .add_systems(
                Update,
                transition_to_in_game.run_if(in_state(GameState::GameOver)),
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

/// App command event for toggling pause state (input/UI/BRP).
#[derive(Event, Reflect)]
#[reflect(Event)]
pub struct PauseEvent;

/// App command event for quick restart flow (Shift+R behavior).
#[derive(Event, Reflect)]
#[reflect(Event)]
pub struct RestartGameEvent;

/// App command event for restart-with-splash flow (Super+Shift+R behavior).
#[derive(Event, Reflect)]
#[reflect(Event)]
pub struct RestartWithSplashEvent;

fn on_pause_input(_trigger: On<input_events::Start<PauseToggle>>, mut commands: Commands) {
    // Input adapter only: command behavior is owned by the app command event observer,
    // which is also triggerable through BRP `world.trigger_event`.
    commands.trigger(PauseEvent);
}

fn on_pause_event(_trigger: On<PauseEvent>, mut commands: Commands) {
    commands.run_system_cached(pause_command);
}

/// Reusable on-demand command for toggling pause state.
fn pause_command(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<PauseState>>,
    pause_state: Option<Res<State<PauseState>>>,
) {
    if *game_state.get() != GameState::InGame {
        return;
    }

    let Some(state) = pause_state else {
        return;
    };

    match state.get() {
        PauseState::Playing => next_state.set(PauseState::Paused),
        PauseState::Paused => next_state.set(PauseState::Playing),
    }
}

fn on_restart_game_input(
    _trigger: On<input_events::Start<RestartGameShortcut>>,
    mut commands: Commands,
) {
    commands.trigger(RestartGameEvent);
}

fn on_restart_game_event(_trigger: On<RestartGameEvent>, mut commands: Commands) {
    commands.run_system_cached(restart_game_command);
}

/// Reusable on-demand command for quick restart flow.
fn restart_game_command(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() != GameState::InGame {
        return;
    }

    // Quick restart flow (Shift+R):
    // 1. InGame → GameOver: Stars regenerate, actors despawn
    // 2. GameOver → InGame: No star regeneration (stars from step 1 persist)
    // 3. Fresh game starts with stars already generated
    debug!("restart quick");
    next_state.set(GameState::GameOver);
}

fn on_restart_with_splash_input(
    _trigger: On<input_events::Start<RestartWithSplashShortcut>>,
    mut commands: Commands,
) {
    commands.trigger(RestartWithSplashEvent);
}

fn on_restart_with_splash_event(_trigger: On<RestartWithSplashEvent>, mut commands: Commands) {
    commands.run_system_cached(restart_with_splash_command);
}

/// Reusable on-demand command for full restart-with-splash flow.
fn restart_with_splash_command(
    game_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if *game_state.get() != GameState::InGame {
        return;
    }

    // Full restart with splash flow (Super+Shift+R):
    // 1. InGame → Splash: Stars regenerate, actors despawn, splash timer resets
    // 2. Splash → InGame: No star regeneration (stars from step 1 persist)
    // 3. Game starts with stars that were generated during splash
    debug!("restart with splash");
    next_state.set(GameState::Splash);
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
