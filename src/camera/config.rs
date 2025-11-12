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
        .init_resource::<CameraConfig>();
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraConfig {
    pub clear_color:                       Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub darkening_factor:                  f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_intensity:                   f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_low_frequency_boost:         f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_high_pass_frequency:         f32,
    #[inspector(min = 0.0, max = 0.5, display = NumberDisplay::Slider)]
    pub zoom_to_fit_margin:                f32,
    // Zoom-to-fit convergence parameters
    /// Convergence rate during fitting phase (0.12 = 12% per frame).
    /// Careful rate to avoid overshooting target margins when zooming in/out.
    #[inspector(min = 0.01, max = 0.5, display = NumberDisplay::Slider)]
    pub zoom_to_fit_fitting_rate:          f32,
    /// Convergence rate during balancing phase (0.50 = 50% per frame).
    /// Faster rate since we're only centering - the fit can't be lost by adjusting focus.
    #[inspector(min = 0.01, max = 1.0, display = NumberDisplay::Slider)]
    pub zoom_to_fit_balancing_rate:        f32,
    /// Convergence threshold for stopping when dimension flip detected (0.05 = 5% tolerance)
    #[inspector(min = 0.01, max = 0.2, display = NumberDisplay::Slider)]
    pub zoom_to_fit_convergence_threshold: f32,
    /// Damping factor when dimension flip detected but not converged (0.30 = 30% speed)
    #[inspector(min = 0.1, max = 1.0, display = NumberDisplay::Slider)]
    pub zoom_to_fit_flip_damping:          f32,
    /// Minimum ratio clamp to prevent huge jumps (0.5 = max 50% shrink per frame)
    #[inspector(min = 0.1, max = 0.9, display = NumberDisplay::Slider)]
    pub zoom_to_fit_min_ratio:             f32,
    /// Maximum ratio clamp to prevent huge jumps (1.5 = max 50% grow per frame)
    #[inspector(min = 1.1, max = 3.0, display = NumberDisplay::Slider)]
    pub zoom_to_fit_max_ratio:             f32,
    /// Emergency zoom out multiplier when content outside view (1.25 = 25% zoom out)
    #[inspector(min = 1.1, max = 2.0, display = NumberDisplay::Slider)]
    pub zoom_to_fit_emergency_zoom_out:    f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:                       Color::from(tailwind::SLATE_900),
            darkening_factor:                  0.002,
            bloom_intensity:                   0.5,
            bloom_low_frequency_boost:         0.5,
            bloom_high_pass_frequency:         0.5,
            zoom_to_fit_margin:                0.08,
            zoom_to_fit_fitting_rate:          0.12,
            zoom_to_fit_balancing_rate:        0.50,
            zoom_to_fit_convergence_threshold: 0.05,
            zoom_to_fit_flip_damping:          0.30,
            zoom_to_fit_min_ratio:             0.5,
            zoom_to_fit_max_ratio:             1.5,
            zoom_to_fit_emergency_zoom_out:    1.25,
        }
    }
}

impl CameraConfig {
    /// Returns the zoom buffer multiplier (1.0 + buffer)
    /// For example, a buffer of 0.05 returns 1.05 (5% margin)
    pub const fn zoom_multiplier(&self) -> f32 { 1.0 + self.zoom_to_fit_margin }
}
