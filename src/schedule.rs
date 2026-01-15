use bevy::prelude::*;

use crate::state::PauseState;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum InGameSet {
    UserInput,
    EntityUpdates,
    CollisionDetection,
    DespawnEntities,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        const IN_GAME_SETS: (InGameSet, InGameSet, InGameSet, InGameSet) = (
            InGameSet::DespawnEntities,
            InGameSet::CollisionDetection,
            InGameSet::UserInput,
            InGameSet::EntityUpdates,
        );

        app.configure_sets(
            Update,
            // All in-game systems are gated by `PauseState::Playing`. When paused,
            // `PauseState` transitions to `Paused` and these systems stop running.
            // `PauseState` is a SubState that only exists while in `GameState::InGame`.
            IN_GAME_SETS.chain().run_if(in_state(PauseState::Playing)),
        )
        .configure_sets(
            FixedUpdate,
            IN_GAME_SETS.chain().run_if(in_state(PauseState::Playing)),
        );
    }
}
