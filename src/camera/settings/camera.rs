use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_kana::Position;

use crate::camera::constants::CAMERA_BLOOM_HIGH_PASS_FREQUENCY;
use crate::camera::constants::CAMERA_BLOOM_INTENSITY;
use crate::camera::constants::CAMERA_BLOOM_LOW_FREQUENCY_BOOST;
use crate::camera::constants::CAMERA_BLOOM_MAX;
use crate::camera::constants::CAMERA_BLOOM_MIN;
use crate::camera::constants::CAMERA_ORBIT_SMOOTHNESS;
use crate::camera::constants::CAMERA_PAN_SMOOTHNESS;
use crate::camera::constants::CAMERA_SMOOTHNESS_MAX;
use crate::camera::constants::CAMERA_SMOOTHNESS_MIN;
use crate::camera::constants::CAMERA_SPLASH_ANGLE_MAX;
use crate::camera::constants::CAMERA_SPLASH_ANGLE_MIN;
use crate::camera::constants::CAMERA_SPLASH_RADIUS_MAX;
use crate::camera::constants::CAMERA_SPLASH_RADIUS_MIN;
use crate::camera::constants::CAMERA_SPLASH_START_FOCUS;
use crate::camera::constants::CAMERA_SPLASH_START_PITCH;
use crate::camera::constants::CAMERA_SPLASH_START_RADIUS;
use crate::camera::constants::CAMERA_SPLASH_START_YAW;
use crate::camera::constants::CAMERA_ZOOM_SMOOTHNESS;
use crate::input::InspectCameraSwitch;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(InspectCameraEvent);

pub(super) struct CameraSettingsInspectorPlugin;

impl Plugin for CameraSettingsInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<CameraSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectCamera)),
        )
        .init_resource::<CameraSettings>();
        bind_action_switch!(
            app,
            InspectCameraSwitch,
            InspectCameraEvent,
            Switch::InspectCamera
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
    /// Camera starting distance for splash screen animation.
    #[inspector(min = CAMERA_SPLASH_RADIUS_MIN, max = CAMERA_SPLASH_RADIUS_MAX)]
    pub radius: f32,
    /// Camera starting focus point for splash screen animation.
    pub focus:  Position,
    /// Camera starting pitch angle for splash screen animation.
    #[inspector(
        min = CAMERA_SPLASH_ANGLE_MIN,
        max = CAMERA_SPLASH_ANGLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub pitch:  f32,
    /// Camera starting yaw angle for splash screen animation.
    #[inspector(
        min = CAMERA_SPLASH_ANGLE_MIN,
        max = CAMERA_SPLASH_ANGLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub yaw:    f32,
}

#[derive(Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct SmoothnessSettings {
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub zoom:  f32,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub pan:   f32,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub orbit: f32,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraSettings {
    pub bloom:        BloomSettings,
    pub smoothness:   SmoothnessSettings,
    pub splash_start: SplashStart,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            bloom:        BloomSettings {
                intensity:           CAMERA_BLOOM_INTENSITY,
                low_frequency_boost: CAMERA_BLOOM_LOW_FREQUENCY_BOOST,
                high_pass_frequency: CAMERA_BLOOM_HIGH_PASS_FREQUENCY,
            },
            smoothness:   SmoothnessSettings {
                zoom:  CAMERA_ZOOM_SMOOTHNESS,
                pan:   CAMERA_PAN_SMOOTHNESS,
                orbit: CAMERA_ORBIT_SMOOTHNESS,
            },
            splash_start: SplashStart {
                radius: CAMERA_SPLASH_START_RADIUS,
                focus:  CAMERA_SPLASH_START_FOCUS,
                pitch:  CAMERA_SPLASH_START_PITCH,
                yaw:    CAMERA_SPLASH_START_YAW,
            },
        }
    }
}
