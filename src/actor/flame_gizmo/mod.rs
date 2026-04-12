mod death_effect;
mod thruster;

use bevy::prelude::*;

use super::constants::FLAME_COLOR_FLICKER_SPEED;
use super::constants::FLAME_GIZMO_LINE_WIDTH;
use super::constants::FLAME_LENGTH_FLICKER_PHASE_MULTIPLIER;
use super::constants::FLAME_LENGTH_FLICKER_SPEED;
use super::constants::FLAME_PHASE_SPREAD;
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

pub(super) struct FlickerValues {
    pub length: f32,
    pub color:  f32,
}

pub(super) fn compute_flicker(elapsed: f32, line_index: f32, phase_offset: f32) -> FlickerValues {
    let phase = line_index.mul_add(FLAME_PHASE_SPREAD, phase_offset);
    FlickerValues {
        length: elapsed
            .mul_add(
                FLAME_LENGTH_FLICKER_SPEED,
                phase * FLAME_LENGTH_FLICKER_PHASE_MULTIPLIER,
            )
            .sin()
            .mul_add(0.5, 0.5),
        color:  elapsed
            .mul_add(FLAME_COLOR_FLICKER_SPEED, phase)
            .sin()
            .mul_add(0.5, 0.5),
    }
}

pub(super) fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::from(a.to_linear().mix(&b.to_linear(), t.clamp(0.0, 1.0)))
}
