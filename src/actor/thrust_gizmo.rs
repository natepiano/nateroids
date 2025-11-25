use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

use super::constants::THRUST_CONE_HALF_ANGLE;
use super::constants::THRUST_LINE_COUNT;
use super::constants::THRUST_LINE_LENGTH_BASE;
use super::constants::THRUST_LINE_LENGTH_VARIANCE;
use super::constants::THRUST_LINE_OFFSET;
use super::constants::THRUST_LINE_WIDTH;
use super::constants::THRUST_VIBRATION_AMPLITUDE;
use super::constants::THRUST_VIBRATION_SPEED;
use super::spaceship::Spaceship;
use super::spaceship_control::SpaceshipControl;
use crate::camera::RenderLayer;
use crate::state::PlayingGame;

pub struct ThrustGizmoPlugin;

impl Plugin for ThrustGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<ThrustGizmo>()
            .add_systems(Startup, configure_thrust_gizmo)
            .add_systems(Update, draw_thrust_flames.run_if(in_state(PlayingGame)));
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct ThrustGizmo {}

fn configure_thrust_gizmo(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<ThrustGizmo>();
    config.line.width = THRUST_LINE_WIDTH;
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

fn draw_thrust_flames(
    mut gizmos: Gizmos<ThrustGizmo>,
    time: Res<Time>,
    spaceship_query: Query<(&Transform, &ActionState<SpaceshipControl>), With<Spaceship>>,
) {
    let Ok((transform, controls)) = spaceship_query.single() else {
        return;
    };

    if !controls.pressed(&SpaceshipControl::Accelerate) {
        return;
    }

    let elapsed = time.elapsed_secs();
    let back_direction = -transform.forward().as_vec3();
    let right = transform.right().as_vec3();
    let up = transform.up().as_vec3();

    // Base position slightly behind the spaceship
    let base_position = transform.translation + back_direction * THRUST_LINE_OFFSET;

    // Fire colors for lerping
    let color_yellow = Color::from(tailwind::YELLOW_400);
    let color_orange = Color::from(tailwind::ORANGE_500);
    let color_red = Color::from(tailwind::RED_600);

    for i in 0..THRUST_LINE_COUNT {
        let line_index = i as f32;
        let angle_offset = (line_index - (THRUST_LINE_COUNT as f32 - 1.0) / 2.0)
            / (THRUST_LINE_COUNT as f32 / 2.0)
            * THRUST_CONE_HALF_ANGLE;

        // Vibration offset unique to each line
        let phase = line_index * 1.7;
        let vibration_lateral =
            (elapsed * THRUST_VIBRATION_SPEED + phase).sin() * THRUST_VIBRATION_AMPLITUDE;
        let vibration_vertical = (elapsed * THRUST_VIBRATION_SPEED * 1.3 + phase * 0.7).cos()
            * THRUST_VIBRATION_AMPLITUDE;

        // Length variation per line with time-based flicker
        let length_flicker = (elapsed * 15.0 + phase * 2.3).sin() * 0.5 + 0.5;
        let line_length = THRUST_LINE_LENGTH_BASE + length_flicker * THRUST_LINE_LENGTH_VARIANCE;

        // Spread direction based on cone angle
        let spread_direction = (back_direction + right * angle_offset.sin()).normalize();

        // Apply vibration perpendicular to thrust direction
        let start = base_position + right * vibration_lateral + up * vibration_vertical;
        let end = start + spread_direction * line_length;

        // Color: 3-zone gradient (red -> orange -> yellow) based on distance from center
        // center_factor: 0.0 = outermost, 1.0 = center
        let center_factor = 1.0 - (angle_offset.abs() / THRUST_CONE_HALF_ANGLE);
        let time_flicker = (elapsed * 12.0 + phase).sin() * 0.5 + 0.5;

        // Map center_factor to 3 color zones with smooth transitions
        // 0.0-0.33: red to orange, 0.33-0.66: orange, 0.66-1.0: orange to yellow
        let color = if center_factor < 0.33 {
            // Outer zone: red to orange
            let t = (center_factor / 0.33) * (0.7 + time_flicker * 0.3);
            lerp_color(color_red, color_orange, t)
        } else if center_factor < 0.66 {
            // Middle zone: orange with flicker toward red or yellow
            let flicker_bias = (elapsed * 8.0 + phase * 1.3).sin();
            if flicker_bias > 0.0 {
                lerp_color(color_orange, color_yellow, flicker_bias * 0.4)
            } else {
                lerp_color(color_orange, color_red, -flicker_bias * 0.3)
            }
        } else {
            // Center zone: orange to yellow (hottest)
            let t = ((center_factor - 0.66) / 0.34) * (0.6 + time_flicker * 0.4);
            lerp_color(color_orange, color_yellow, t)
        };

        gizmos.line(start, end, color);
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let a_linear = a.to_linear();
    let b_linear = b.to_linear();
    let t = t.clamp(0.0, 1.0);

    Color::linear_rgba(
        a_linear.red + (b_linear.red - a_linear.red) * t,
        a_linear.green + (b_linear.green - a_linear.green) * t,
        a_linear.blue + (b_linear.blue - a_linear.blue) * t,
        a_linear.alpha + (b_linear.alpha - a_linear.alpha) * t,
    )
}
