mod death_effect;
mod flicker;
mod thruster;

use bevy::prelude::*;

use super::constants::FLAME_GIZMO_LINE_WIDTH;
use crate::camera::RenderLayer;
use crate::state::GameState;
use crate::state::PauseState;

pub(super) struct FlameGizmoPlugin;

impl Plugin for FlameGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FlameGizmo>()
            .add_systems(Startup, configure_flame_gizmo)
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

fn configure_flame_gizmo(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<FlameGizmo>();
    config.line.width = FLAME_GIZMO_LINE_WIDTH;
    config.render_layers = RenderLayer::Game.layers();
}
