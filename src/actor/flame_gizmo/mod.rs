mod death_effect;
mod flicker;
mod thruster;

use bevy::prelude::*;

use crate::state::GameState;
use crate::state::PauseState;

pub(super) struct FlameGizmoPlugin;

impl Plugin for FlameGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FlameGizmo>()
            .add_systems(Startup, thruster::configure_flame_gizmo)
            .add_observer(death_effect::on_deaderoid_added)
            .add_systems(
                Update,
                thruster::update_thruster_effect.run_if(in_state(PauseState::Playing)),
            )
            .add_systems(
                Update,
                (
                    thruster::draw_thruster_flames,
                    death_effect::draw_death_effects,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub(super) struct FlameGizmo {}
