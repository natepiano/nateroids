use std::ops::Range;

use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_kana::Position;

use super::constants::CAMERA_BLOOM_HIGH_PASS_FREQUENCY;
use super::constants::CAMERA_BLOOM_INTENSITY;
use super::constants::CAMERA_BLOOM_LOW_FREQUENCY_BOOST;
use super::constants::CAMERA_BLOOM_MAX;
use super::constants::CAMERA_BLOOM_MIN;
use super::constants::CAMERA_ORBIT_SMOOTHNESS;
use super::constants::CAMERA_PAN_SMOOTHNESS;
use super::constants::CAMERA_SMOOTHNESS_MAX;
use super::constants::CAMERA_SMOOTHNESS_MIN;
use super::constants::CAMERA_SPLASH_ANGLE_MAX;
use super::constants::CAMERA_SPLASH_ANGLE_MIN;
use super::constants::CAMERA_SPLASH_RADIUS_MAX;
use super::constants::CAMERA_SPLASH_RADIUS_MIN;
use super::constants::CAMERA_SPLASH_START_FOCUS;
use super::constants::CAMERA_SPLASH_START_PITCH;
use super::constants::CAMERA_SPLASH_START_RADIUS;
use super::constants::CAMERA_SPLASH_START_YAW;
use super::constants::CAMERA_ZOOM_SMOOTHNESS;
use super::constants::STAR_BATCH_SIZE_REPLACE;
use super::constants::STAR_COLOR_RANGE_MAX;
use super::constants::STAR_COLOR_RANGE_MIN;
use super::constants::STAR_COLOR_WHITE_PROBABILITY;
use super::constants::STAR_COLOR_WHITE_START_RATIO;
use super::constants::STAR_COUNT;
use super::constants::STAR_DURATION_REPLACE_TIMER;
use super::constants::STAR_FIELD_DIAMETER;
use super::constants::STAR_RADIUS;
use super::constants::STAR_ROTATION_CYCLE_MAX;
use super::constants::STAR_ROTATION_CYCLE_MINIMUM_MINUTES;
use super::constants::STAR_ROTATION_CYCLE_MINUTES;
use super::constants::STAR_TWINKLE_CHOOSE_MULTIPLE_COUNT;
use super::constants::STAR_TWINKLE_DURATION_MAX;
use super::constants::STAR_TWINKLE_DURATION_MIN;
use super::constants::STAR_TWINKLE_INTENSITY_MAX;
use super::constants::STAR_TWINKLE_INTENSITY_MIN;
use super::constants::STAR_TWINKLING_DELAY;
use super::constants::ZOOM_CONVERGENCE_RATE;
use super::constants::ZOOM_CONVERGENCE_RATE_MAX;
use super::constants::ZOOM_CONVERGENCE_RATE_MIN;
use super::constants::ZOOM_MARGIN_MAX;
use super::constants::ZOOM_MARGIN_MIN;
use super::constants::ZOOM_MARGIN_TOLERANCE;
use super::constants::ZOOM_MARGIN_TOLERANCE_MAX;
use super::constants::ZOOM_MARGIN_TOLERANCE_MIN;
use super::constants::ZOOM_MAX_ITERATIONS;
use super::constants::ZOOM_MAX_ITERATIONS_MAX;
use super::constants::ZOOM_MAX_ITERATIONS_MIN;
use super::constants::ZOOM_SETTINGS_MARGIN;
use crate::input::InspectCameraSwitch;
use crate::input::InspectStarSwitch;
use crate::input::InspectZoomSwitch;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(InspectCameraEvent);
event!(InspectStarEvent);
event!(InspectZoomEvent);

pub(super) struct CameraSettingsPlugin;

impl Plugin for CameraSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<CameraSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectCamera)),
        )
        .add_plugins(
            ResourceInspectorPlugin::<StarSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectStar)),
        )
        .add_plugins(
            ResourceInspectorPlugin::<ZoomSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectZoom)),
        )
        .init_resource::<CameraSettings>()
        .init_resource::<StarSettings>()
        .init_resource::<ZoomSettings>();
        bind_action_switch!(
            app,
            InspectCameraSwitch,
            InspectCameraEvent,
            Switch::InspectCamera
        );
        bind_action_switch!(
            app,
            InspectStarSwitch,
            InspectStarEvent,
            Switch::InspectStar
        );
        bind_action_switch!(
            app,
            InspectZoomSwitch,
            InspectZoomEvent,
            Switch::InspectZoom
        );
    }
}

#[derive(Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct BloomSettings {
    #[inspector(
        min = CAMERA_BLOOM_MIN,
        max = CAMERA_BLOOM_MAX,
        display = NumberDisplay::Slider
    )]
    pub intensity:           f32,
    #[inspector(
        min = CAMERA_BLOOM_MIN,
        max = CAMERA_BLOOM_MAX,
        display = NumberDisplay::Slider
    )]
    pub low_frequency_boost: f32,
    #[inspector(
        min = CAMERA_BLOOM_MIN,
        max = CAMERA_BLOOM_MAX,
        display = NumberDisplay::Slider
    )]
    pub high_pass_frequency: f32,
}

#[derive(Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct SplashStart {
    /// Camera starting distance for splash screen animation
    #[inspector(min = CAMERA_SPLASH_RADIUS_MIN, max = CAMERA_SPLASH_RADIUS_MAX)]
    pub radius: f32,
    /// Camera starting focus point for splash screen animation
    pub focus:  Position,
    /// Camera starting pitch angle for splash screen animation
    #[inspector(
        min = CAMERA_SPLASH_ANGLE_MIN,
        max = CAMERA_SPLASH_ANGLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub pitch:  f32,
    /// Camera starting yaw angle for splash screen animation
    #[inspector(
        min = CAMERA_SPLASH_ANGLE_MIN,
        max = CAMERA_SPLASH_ANGLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub yaw:    f32,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraSettings {
    pub bloom:            BloomSettings,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub zoom_smoothness:  f32,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub pan_smoothness:   f32,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub orbit_smoothness: f32,
    pub splash_start:     SplashStart,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            bloom:            BloomSettings {
                intensity:           CAMERA_BLOOM_INTENSITY,
                low_frequency_boost: CAMERA_BLOOM_LOW_FREQUENCY_BOOST,
                high_pass_frequency: CAMERA_BLOOM_HIGH_PASS_FREQUENCY,
            },
            zoom_smoothness:  CAMERA_ZOOM_SMOOTHNESS,
            orbit_smoothness: CAMERA_ORBIT_SMOOTHNESS,
            pan_smoothness:   CAMERA_PAN_SMOOTHNESS,
            splash_start:     SplashStart {
                radius: CAMERA_SPLASH_START_RADIUS,
                focus:  CAMERA_SPLASH_START_FOCUS,
                pitch:  CAMERA_SPLASH_START_PITCH,
                yaw:    CAMERA_SPLASH_START_YAW,
            },
        }
    }
}

#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(super) struct StarColorSettings {
    pub range:             Range<f32>,
    pub white_probability: f32,
    pub white_start_ratio: f32,
}

#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(super) struct StarTwinkleSettings {
    pub delay:                 f32,
    pub duration:              Range<f32>,
    pub intensity:             Range<f32>,
    pub choose_multiple_count: usize,
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct StarSettings {
    pub batch_size_replace:     usize,
    pub duration_replace_timer: f32,
    pub color:                  StarColorSettings,
    pub count:                  usize,
    pub radius:                 Range<f32>,
    pub field_diameter:         Range<f32>,
    pub twinkle:                StarTwinkleSettings,
    #[inspector(
        min = STAR_ROTATION_CYCLE_MINIMUM_MINUTES,
        max = STAR_ROTATION_CYCLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub rotation_cycle_minutes: f32,
    pub rotation_axis:          Vec3,
}

impl Default for StarSettings {
    fn default() -> Self {
        Self {
            batch_size_replace:     STAR_BATCH_SIZE_REPLACE,
            duration_replace_timer: STAR_DURATION_REPLACE_TIMER,
            count:                  STAR_COUNT,
            color:                  StarColorSettings {
                range:             STAR_COLOR_RANGE_MIN..STAR_COLOR_RANGE_MAX,
                white_probability: STAR_COLOR_WHITE_PROBABILITY,
                white_start_ratio: STAR_COLOR_WHITE_START_RATIO,
            },
            radius:                 STAR_RADIUS,
            field_diameter:         STAR_FIELD_DIAMETER,
            twinkle:                StarTwinkleSettings {
                delay:                 STAR_TWINKLING_DELAY,
                duration:              STAR_TWINKLE_DURATION_MIN..STAR_TWINKLE_DURATION_MAX,
                intensity:             STAR_TWINKLE_INTENSITY_MIN..STAR_TWINKLE_INTENSITY_MAX,
                choose_multiple_count: STAR_TWINKLE_CHOOSE_MULTIPLE_COUNT,
            },
            rotation_cycle_minutes: STAR_ROTATION_CYCLE_MINUTES,
            rotation_axis:          Vec3::Y,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct ZoomSettings {
    /// Maximum iterations before giving up
    #[inspector(min = ZOOM_MAX_ITERATIONS_MIN, max = ZOOM_MAX_ITERATIONS_MAX)]
    pub max_iterations:   usize,
    #[inspector(
        min = ZOOM_MARGIN_MIN,
        max = ZOOM_MARGIN_MAX,
        display = NumberDisplay::Slider
    )]
    pub margin:           f32,
    /// Margin tolerance for convergence detection (0.001 = 0.1% tolerance).
    /// Used for both balance and fit checks.
    #[inspector(
        min = ZOOM_MARGIN_TOLERANCE_MIN,
        max = ZOOM_MARGIN_TOLERANCE_MAX,
        display = NumberDisplay::Slider
    )]
    pub margin_tolerance: f32,
    // Zoom-to-fit convergence parameters
    /// Convergence rate for zoom-to-fit adjustments (0.18 = 18% per frame).
    #[inspector(
        min = ZOOM_CONVERGENCE_RATE_MIN,
        max = ZOOM_CONVERGENCE_RATE_MAX,
        display = NumberDisplay::Slider
    )]
    pub convergence_rate: f32,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            max_iterations:   ZOOM_MAX_ITERATIONS,
            margin:           ZOOM_SETTINGS_MARGIN,
            margin_tolerance: ZOOM_MARGIN_TOLERANCE,
            convergence_rate: ZOOM_CONVERGENCE_RATE,
        }
    }
}
