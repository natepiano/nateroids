use bevy::anti_alias::smaa::Smaa;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use hana_kana::Position;
use hana_lagrange::Operation;
use hana_lagrange::OrbitCam;
use hana_lagrange::OrbitCamBlenderLikePreset;
use hana_lagrange::OrbitCamInputGain;
use hana_lagrange::OrbitCamInputMode;
use hana_lagrange::OrbitCamPreset;
use hana_lagrange::Radius;
use hana_lagrange::ScalarLimit;
use hana_liminal::OutlineCamera;

use super::RenderLayer;
use super::constants::CAMERA_BLOOM_HIGH_PASS_FREQUENCY;
use super::constants::CAMERA_BLOOM_INTENSITY;
use super::constants::CAMERA_BLOOM_LOW_FREQUENCY_BOOST;
use super::constants::CAMERA_BLOOM_MAX;
use super::constants::CAMERA_BLOOM_MIN;
use super::constants::CAMERA_INPUT_SENSITIVITY;
use super::constants::CAMERA_LINE_SCROLL_SENSITIVITY;
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
use super::constants::CAMERA_ZOOM_LOWER_LIMIT;
use super::constants::CAMERA_ZOOM_SENSITIVITY;
use super::constants::CAMERA_ZOOM_SMOOTHNESS;
use super::constants::FORWARDED_TRACKPAD_ENV;
use super::lights::LightSettings;
use super::rendering::CameraOrder;
use super::required_components::RequiredCameraComponents;
use crate::asset_loader::SceneAssets;
use crate::input::InspectCameraSwitch;
use crate::switches;
use crate::switches::Switch;

pub(super) struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraSettings>()
            .add_plugins(
                ResourceInspectorPlugin::<CameraSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectCamera)),
            )
            .add_systems(Update, update_environment_map_intensity)
            .add_systems(
                Update,
                (update_orbit_cam_smoothness, update_camera_input_mode)
                    .run_if(resource_changed::<CameraSettings>),
            );
        bind_action_switch!(
            app,
            InspectCameraSwitch,
            InspectCameraEvent,
            Switch::InspectCamera
        );
    }
}

event!(InspectCameraEvent);

#[derive(Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(InspectorOptions)]
pub(crate) struct BloomSettings {
    #[inspector(
        min = CAMERA_BLOOM_MIN,
        max = CAMERA_BLOOM_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) intensity:           f32,
    #[inspector(
        min = CAMERA_BLOOM_MIN,
        max = CAMERA_BLOOM_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) low_frequency_boost: f32,
    #[inspector(
        min = CAMERA_BLOOM_MIN,
        max = CAMERA_BLOOM_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) high_pass_frequency: f32,
}

#[derive(Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(InspectorOptions)]
pub(crate) struct SplashStart {
    /// Camera starting distance for splash screen animation.
    #[inspector(min = CAMERA_SPLASH_RADIUS_MIN, max = CAMERA_SPLASH_RADIUS_MAX)]
    pub(crate) radius: f32,
    /// Camera starting focus point for splash screen animation.
    pub(crate) focus:  Position,
    /// Camera starting pitch angle for splash screen animation.
    #[inspector(
        min = CAMERA_SPLASH_ANGLE_MIN,
        max = CAMERA_SPLASH_ANGLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) pitch:  f32,
    /// Camera starting yaw angle for splash screen animation.
    #[inspector(
        min = CAMERA_SPLASH_ANGLE_MIN,
        max = CAMERA_SPLASH_ANGLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) yaw:    f32,
}

#[derive(Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(InspectorOptions)]
pub(crate) struct SmoothnessSettings {
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    zoom:  f32,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pan:   f32,
    #[inspector(
        min = CAMERA_SMOOTHNESS_MIN,
        max = CAMERA_SMOOTHNESS_MAX,
        display = NumberDisplay::Slider
    )]
    orbit: f32,
}

#[derive(Default, Reflect, Debug, PartialEq, Clone, Copy)]
enum CameraScrollMode {
    #[default]
    MouseWheel,
    ForwardedTrackpad,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct CameraSettings {
    pub(super) bloom_settings: BloomSettings,
    smoothness_settings:       SmoothnessSettings,
    pub(crate) splash_start:   SplashStart,
    scroll_mode:               CameraScrollMode,
    #[inspector(min = 0.0)]
    line_scroll_sensitivity:   f32,
}

impl CameraSettings {
    fn input_mode(&self) -> OrbitCamInputMode {
        let line_scroll_input_gain = match self.scroll_mode {
            CameraScrollMode::MouseWheel => None,
            CameraScrollMode::ForwardedTrackpad => {
                Some(OrbitCamInputGain::uniform(self.line_scroll_sensitivity))
            },
        };
        OrbitCamInputMode::Preset(OrbitCamPreset::from(
            OrbitCamBlenderLikePreset::default()
                .mouse_input_gain(OrbitCamInputGain::uniform(CAMERA_INPUT_SENSITIVITY))
                .line_scroll_input_gain(line_scroll_input_gain),
        ))
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            scroll_mode:             if std::env::var(FORWARDED_TRACKPAD_ENV)
                .is_ok_and(|value| value == "1")
            {
                CameraScrollMode::ForwardedTrackpad
            } else {
                CameraScrollMode::MouseWheel
            },
            line_scroll_sensitivity: CAMERA_LINE_SCROLL_SENSITIVITY,
            bloom_settings:          BloomSettings {
                intensity:           CAMERA_BLOOM_INTENSITY,
                low_frequency_boost: CAMERA_BLOOM_LOW_FREQUENCY_BOOST,
                high_pass_frequency: CAMERA_BLOOM_HIGH_PASS_FREQUENCY,
            },
            smoothness_settings:     SmoothnessSettings {
                zoom:  CAMERA_ZOOM_SMOOTHNESS,
                pan:   CAMERA_PAN_SMOOTHNESS,
                orbit: CAMERA_ORBIT_SMOOTHNESS,
            },
            splash_start:            SplashStart {
                radius: CAMERA_SPLASH_START_RADIUS,
                focus:  CAMERA_SPLASH_START_FOCUS,
                pitch:  CAMERA_SPLASH_START_PITCH,
                yaw:    CAMERA_SPLASH_START_YAW,
            },
        }
    }
}

pub(super) fn game_camera(
    camera_settings: &CameraSettings,
    scene_assets: &SceneAssets,
    light_settings: &LightSettings,
) -> impl Scene {
    let input_mode = camera_settings.input_mode();

    bsn! {
        template(|_| Ok(OutlineCamera))
        RequiredCameraComponents
        OrbitCam {
            zoom: {Operation::new(
                Radius(camera_settings.splash_start.radius),
                CAMERA_ZOOM_SENSITIVITY,
                CAMERA_ZOOM_SMOOTHNESS,
                ScalarLimit::Clamp {
                    min: CAMERA_ZOOM_LOWER_LIMIT,
                    max: f32::INFINITY,
                },
            )},
        }
        // Middle-drag orbit, Shift+middle-drag pan, Blender-style trackpad.
        input_mode
        Camera {
            order: {CameraOrder::Game.order()},
            // can't obscure the star camera with this on
            clear_color: ClearColorConfig::None,
        }
        RenderLayer::Game.layers()
        Smaa
        EnvironmentMapLight {
            diffuse_map: {scene_assets.environment_diffuse_map.clone()},
            specular_map: {scene_assets.environment_specular_map.clone()},
            intensity: {light_settings.environment_map_intensity},
        }
    }
}

fn update_environment_map_intensity(
    light_settings: Res<LightSettings>,
    mut environment_map_light_query: Query<&mut EnvironmentMapLight, With<Camera3d>>,
) {
    if !light_settings.is_changed() {
        return;
    }

    for mut environment_light in &mut environment_map_light_query {
        environment_light.intensity = light_settings.environment_map_intensity;
    }
}

fn update_orbit_cam_smoothness(
    camera_settings: Res<CameraSettings>,
    mut orbit_cam_query: Query<&mut OrbitCam>,
) {
    for mut camera in &mut orbit_cam_query {
        camera
            .zoom
            .set_damping(camera_settings.smoothness_settings.zoom);
        camera
            .pan
            .set_damping(camera_settings.smoothness_settings.pan);
        camera
            .orbit
            .set_damping(camera_settings.smoothness_settings.orbit);
    }
}

fn update_camera_input_mode(
    settings: Res<CameraSettings>,
    mut cameras: Query<&mut OrbitCamInputMode, With<OrbitCam>>,
) {
    let mode = settings.input_mode();
    for mut current in &mut cameras {
        if *current != mode {
            *current = mode.clone();
        }
    }
}
