use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_enhanced_input::action::TriggerState;
use bevy_enhanced_input::prelude::Action;
use bevy_enhanced_input::prelude::ActionOf;
use bevy_kana::ToF32;

use super::FlameGizmo;
use super::flicker;
use crate::actor::constants::FLAME_COLOR_FLICKER_SPEED;
use crate::actor::constants::FLAME_PHASE_SPREAD;
use crate::actor::constants::FLAME_VIBRATION_AMPLITUDE;
use crate::actor::constants::FLAME_VIBRATION_SPEED;
use crate::actor::constants::THRUSTER_COLOR_FLICKER_INTENSITY;
use crate::actor::constants::THRUSTER_COLOR_ZONE_SIZE;
use crate::actor::constants::THRUSTER_CONE_HALF_ANGLE;
use crate::actor::constants::THRUSTER_LINE_COUNT;
use crate::actor::constants::THRUSTER_LINE_LENGTH_BASE;
use crate::actor::constants::THRUSTER_LINE_LENGTH_VARIANCE;
use crate::actor::constants::THRUSTER_LINE_OFFSET;
use crate::actor::constants::THRUSTER_VIBRATION_VERTICAL_PHASE_MULTIPLIER;
use crate::actor::constants::THRUSTER_VIBRATION_VERTICAL_SPEED_MULTIPLIER;
use crate::actor::spaceship::Spaceship;
use crate::input::ShipAccelerate;
use crate::input::ShipControlsContext;
use crate::state::PauseState;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct ThrusterEffect;

/// Classifies a position within a flame into temperature zones.
/// Used for thruster color gradients where center = hottest.
enum FlameZone {
    /// Outer edge of flame (coolest) - value is normalized 0-1 within zone
    Outer(f32),
    /// Middle transition zone (uses time-based flicker, not position)
    Middle,
    /// Center of flame (hottest) - value is normalized 0-1 within zone
    Center(f32),
}

impl FlameZone {
    /// Classifies a `center_factor` (0.0 = edge, 1.0 = center) into zones.
    fn from_center_factor(value: f32, zone_size: f32) -> Self {
        let middle_threshold = zone_size * 2.0;
        if value < zone_size {
            Self::Outer(value / zone_size)
        } else if value < middle_threshold {
            Self::Middle
        } else {
            Self::Center((value - middle_threshold) / (1.0 - middle_threshold))
        }
    }

    /// Computes the flame color based on zone.
    /// - Outer: red -> orange (coolest)
    /// - Middle: orange with flicker toward yellow/red
    /// - Center: orange -> yellow (hottest)
    fn color(
        &self,
        elapsed: f32,
        phase: f32,
        color_red: Color,
        color_orange: Color,
        color_yellow: Color,
    ) -> Color {
        match self {
            Self::Outer(t) => super::flicker::lerp_color(color_red, color_orange, *t),
            Self::Middle => {
                // Flicker between cooler (red) and hotter (yellow)
                let flicker = elapsed.mul_add(FLAME_COLOR_FLICKER_SPEED, phase).sin();
                if flicker > 0.0 {
                    super::flicker::lerp_color(
                        color_orange,
                        color_yellow,
                        flicker * THRUSTER_COLOR_FLICKER_INTENSITY,
                    )
                } else {
                    super::flicker::lerp_color(
                        color_orange,
                        color_red,
                        -flicker * THRUSTER_COLOR_FLICKER_INTENSITY,
                    )
                }
            },
            Self::Center(t) => super::flicker::lerp_color(color_orange, color_yellow, *t),
        }
    }
}

pub(super) fn update_thruster_effect(
    mut commands: Commands,
    ship_query: Query<(Entity, Option<&ThrusterEffect>), With<Spaceship>>,
    accelerate_query: Query<
        &TriggerState,
        (
            With<Action<ShipAccelerate>>,
            With<ActionOf<ShipControlsContext>>,
        ),
    >,
) {
    let Ok((entity, thruster_effect)) = ship_query.single() else {
        return;
    };

    let Ok(accelerate_state) = accelerate_query.single() else {
        return;
    };

    let is_accelerating = *accelerate_state != TriggerState::None;

    match (is_accelerating, thruster_effect) {
        (true, None) => {
            commands.entity(entity).insert(ThrusterEffect);
        },
        (false, Some(_)) => {
            commands.entity(entity).remove::<ThrusterEffect>();
        },
        _ => {},
    }
}

pub(super) fn draw_thruster_flames(
    mut gizmos: Gizmos<FlameGizmo>,
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    mut frozen_elapsed: Local<f32>,
    query: Query<&Transform, With<ThrusterEffect>>,
) {
    let elapsed = if *pause_state.get() == PauseState::Paused {
        *frozen_elapsed
    } else {
        *frozen_elapsed = time.elapsed_secs();
        time.elapsed_secs()
    };

    for transform in query.iter() {
        draw_exhaust_flames(&mut gizmos, transform, elapsed);
    }
}

fn draw_exhaust_flames(gizmos: &mut Gizmos<FlameGizmo>, transform: &Transform, elapsed: f32) {
    let back_direction = -transform.forward().as_vec3();
    let right = transform.right().as_vec3();
    let up = transform.up().as_vec3();

    let base_position = transform.translation + back_direction * THRUSTER_LINE_OFFSET;

    let color_yellow = Color::from(tailwind::YELLOW_400);
    let color_orange = Color::from(tailwind::ORANGE_500);
    let color_red = Color::from(tailwind::RED_600);

    let line_count_f32 = THRUSTER_LINE_COUNT.to_f32();

    for i in 0..THRUSTER_LINE_COUNT {
        let line_index = i.to_f32();
        let angle_offset = (line_index - (line_count_f32 - 1.0) / 2.0) / (line_count_f32 / 2.0)
            * THRUSTER_CONE_HALF_ANGLE;

        let phase = line_index * FLAME_PHASE_SPREAD;
        let vibration_lateral =
            elapsed.mul_add(FLAME_VIBRATION_SPEED, phase).sin() * FLAME_VIBRATION_AMPLITUDE;
        let vibration_vertical = (elapsed * FLAME_VIBRATION_SPEED)
            .mul_add(
                THRUSTER_VIBRATION_VERTICAL_SPEED_MULTIPLIER,
                phase * THRUSTER_VIBRATION_VERTICAL_PHASE_MULTIPLIER,
            )
            .cos()
            * FLAME_VIBRATION_AMPLITUDE;

        let flicker = flicker::compute_flicker(elapsed, line_index, 0.0);
        let line_length = flicker
            .length
            .mul_add(THRUSTER_LINE_LENGTH_VARIANCE, THRUSTER_LINE_LENGTH_BASE);

        let spread_direction = (back_direction + right * angle_offset.sin()).normalize();

        let start = base_position + right * vibration_lateral + up * vibration_vertical;
        let end = start + spread_direction * line_length;

        let center_factor = 1.0 - (angle_offset.abs() / THRUSTER_CONE_HALF_ANGLE);
        let zone = FlameZone::from_center_factor(center_factor, THRUSTER_COLOR_ZONE_SIZE);
        let color = zone.color(elapsed, phase, color_red, color_orange, color_yellow);

        gizmos.line(start, end, color);
    }
}
