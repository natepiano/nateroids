use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use super::constants::CAMERA_SPLASH_START_FOCUS;
use super::constants::CAMERA_SPLASH_START_PITCH;
use super::constants::CAMERA_SPLASH_START_RADIUS;
use super::constants::CAMERA_SPLASH_START_YAW;
use crate::game_input::toggle_active;
use crate::game_input::GameAction;

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
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub zoom_smoothness:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub pan_smoothness:            f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub orbit_smoothness:          f32,
    /// Camera starting distance for splash screen animation
    #[inspector(min = 1000.0, max = 50000.0)]
    pub splash_start_radius:       f32,
    /// Camera starting focus point for splash screen animation
    pub splash_start_focus:        Vec3,
    /// Camera starting pitch angle for splash screen animation
    #[inspector(min = -std::f32::consts::PI, max = std::f32::consts::PI, display = NumberDisplay::Slider)]
    pub splash_start_pitch:        f32,
    /// Camera starting yaw angle for splash screen animation
    #[inspector(min = -std::f32::consts::PI, max = std::f32::consts::PI, display = NumberDisplay::Slider)]
    pub splash_start_yaw:          f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:               Color::from(tailwind::SLATE_900),
            darkening_factor:          0.002,
            bloom_intensity:           0.5,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
            zoom_smoothness:           0.10,
            orbit_smoothness:          0.02,
            pan_smoothness:            0.02,
            splash_start_radius:       CAMERA_SPLASH_START_RADIUS,
            splash_start_focus:        CAMERA_SPLASH_START_FOCUS,
            splash_start_pitch:        CAMERA_SPLASH_START_PITCH,
            splash_start_yaw:          CAMERA_SPLASH_START_YAW,
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
    pub max_iterations:   usize,
    #[inspector(min = 0.0, max = 0.5, display = NumberDisplay::Slider)]
    pub margin:           f32,
    /// Margin tolerance for convergence detection (0.001 = 0.1% tolerance).
    /// Used for both balance and fit checks.
    #[inspector(min = 0.00001, max = 0.01, display = NumberDisplay::Slider)]
    pub margin_tolerance: f32,
    // Zoom-to-fit convergence parameters
    /// Convergence rate for zoom-to-fit adjustments (0.18 = 18% per frame).
    #[inspector(min = 0.01, max = 0.5, display = NumberDisplay::Slider)]
    pub convergence_rate: f32,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            max_iterations:   200,
            margin:           0.1, //percent of screen
            margin_tolerance: 0.00001,
            convergence_rate: 0.30,
        }
    }
}

impl ZoomConfig {
    /// Returns the zoom margin multiplier (1.0 + margin)
    /// For example, a margin of 0.08 returns 1.08 (8% margin)
    pub const fn zoom_margin_multiplier(&self) -> f32 { 1.0 / (1.0 - self.margin) }
}
