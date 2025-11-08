use bevy::prelude::*;

use crate::state::IsPaused;

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
            IN_GAME_SETS
                .chain()
                // the following is pretty cool - because we added an `InGameSet` system set to
                // all the systems that are "in game" - in order to ensure proper ordering
                // the following comes along for the ride - i.e., they will only run _if_
                // `in_state` evaluates to true - i.e., we are in_game
                // and we have a system that runs on state to watch for keyboard control
                // that takes us in or out of `InGame` - i.e., pausing
                // 1 line of code right here allows for pausing and starting the game!
                .run_if(in_state(IsPaused::NotPaused)),
        )
        .configure_sets(
            FixedUpdate,
            IN_GAME_SETS.chain().run_if(in_state(IsPaused::NotPaused)),
        );
    }
}
