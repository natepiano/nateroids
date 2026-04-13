use bevy::prelude::*;

use crate::actor::constants::FLAME_COLOR_FLICKER_SPEED;
use crate::actor::constants::FLAME_LENGTH_FLICKER_PHASE_MULTIPLIER;
use crate::actor::constants::FLAME_LENGTH_FLICKER_SPEED;
use crate::actor::constants::FLAME_PHASE_SPREAD;

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
