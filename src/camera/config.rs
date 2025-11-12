use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::game_input::GameAction;
use crate::game_input::toggle_active;

pub struct CameraConfigPlugin;

impl Plugin for CameraConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<CameraConfig>::default()
                .run_if(toggle_active(false, GameAction::CameraConfigInspector)),
        )
        .init_resource::<CameraConfig>()
        .add_plugins(
            ResourceInspectorPlugin::<ZoomConfig>::default()
                .run_if(toggle_active(false, GameAction::ZoomConfigInspector)),
        )
        .init_resource::<ZoomConfig>();
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraConfig {
    pub clear_color:               Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub darkening_factor:          f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_intensity:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_low_frequency_boost: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_high_pass_frequency: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:               Color::from(tailwind::SLATE_900),
            darkening_factor:          0.002,
            bloom_intensity:           0.5,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
        }
    }
}

impl CameraConfig {
    pub const fn darkening_multiplier(&self) -> f32 { 1.0 - self.darkening_factor }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct ZoomConfig {
    /// Maximum iterations before giving up
    #[inspector(min = 50, max = 500)]
    pub max_iterations:                     usize,
    #[inspector(min = 0.0, max = 0.5, display = NumberDisplay::Slider)]
    pub margin:                             f32,
    // Zoom-to-fit convergence parameters
    /// Convergence rate during fitting phase (0.12 = 12% per frame).
    /// Careful rate to avoid overshooting target margins when zooming in/out.
    #[inspector(min = 0.01, max = 0.5, display = NumberDisplay::Slider)]
    pub convergence_rate_fitting:           f32,
    /// Convergence rate during balancing phase (0.50 = 50% per frame).
    /// Faster rate since we're only centering - the fit can't be lost by adjusting focus.
    #[inspector(min = 0.01, max = 1.0, display = NumberDisplay::Slider)]
    pub convergence_rate_balancing:         f32,
    /// Convergence threshold for stopping when dimension flip detected (0.05 = 5% tolerance)
    #[inspector(min = 0.01, max = 0.2, display = NumberDisplay::Slider)]
    pub stop_on_dimension_flip_threshold:   f32,
    /// Damping factor when dimension flip detected but not converged (0.30 = 30% speed)
    #[inspector(min = 0.1, max = 1.0, display = NumberDisplay::Slider)]
    pub damping_on_dimension_flip_detected: f32,
    /// Minimum ratio clamp to prevent huge jumps (0.5 = max 50% shrink per frame)
    #[inspector(min = 0.1, max = 0.9, display = NumberDisplay::Slider)]
    pub min_ratio_clamp:                    f32,
    /// Maximum ratio clamp to prevent huge jumps (1.5 = max 50% grow per frame)
    #[inspector(min = 1.1, max = 3.0, display = NumberDisplay::Slider)]
    pub max_ratio_clamp:                    f32,
    /// Strict balance tolerance for final convergence (0.002 = 0.2% tolerance)
    #[inspector(min = 0.0001, max = 0.01, display = NumberDisplay::Slider)]
    pub balance_tolerance:                  f32,
    /// Good-enough tolerance for early exit to prevent oscillation (0.01 = 1% tolerance)
    #[inspector(min = 0.001, max = 0.05, display = NumberDisplay::Slider)]
    pub early_exit_tolerance:               f32,
    /// Maximum relative error to allow early exit (0.5 = 50% error)
    #[inspector(min = 0.1, max = 1.0, display = NumberDisplay::Slider)]
    pub max_error_for_exit:                 f32,
    /// Minimum margin value for division safety (0.001)
    #[inspector(min = 0.0001, max = 0.01, display = NumberDisplay::Slider)]
    pub min_margin_divisor:                 f32,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            margin:                             0.08,
            convergence_rate_fitting:           0.12,
            convergence_rate_balancing:         0.50,
            stop_on_dimension_flip_threshold:   0.05,
            damping_on_dimension_flip_detected: 0.30,
            min_ratio_clamp:                    0.5,
            max_ratio_clamp:                    1.5,
            balance_tolerance:                  0.002,
            early_exit_tolerance:               0.01,
            max_error_for_exit:                 0.5,
            max_iterations:                     200,
            min_margin_divisor:                 0.001,
        }
    }
}

impl ZoomConfig {
    /// Returns the zoom margin multiplier (1.0 + margin)
    /// For example, a margin of 0.08 returns 1.08 (8% margin)
    pub const fn zoom_margin_multiplier(&self) -> f32 { 1.0 + self.margin }
}
