use bevy::anti_alias::smaa::Smaa;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_kana::Position;
use bevy_lagrange::InputControl;
use bevy_lagrange::OrbitCam;
use bevy_lagrange::TrackpadInput;
use bevy_liminal::OutlineCamera;

use super::RenderLayer;
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
use super::constants::CAMERA_TRACKPAD_SENSITIVITY;
use super::constants::CAMERA_ZOOM_LOWER_LIMIT;
use super::constants::CAMERA_ZOOM_SENSITIVITY;
use super::constants::CAMERA_ZOOM_SMOOTHNESS;
use super::lights::LightSettings;
use super::rendering::CameraOrder;
use super::required_components::RequiredCameraComponents;
use super::star::StarCamera;
use crate::asset_loader::SceneAssets;
use crate::input::InspectCameraSwitch;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(InspectCameraEvent);

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
                update_orbit_cam_smoothness.run_if(resource_changed::<CameraSettings>),
            );
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
pub(crate) struct CameraSettings {
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

pub(super) fn spawn_game_camera(
    camera_settings: Res<CameraSettings>,
    scene_assets: Res<SceneAssets>,
    light_settings: Res<LightSettings>,
    mut commands: Commands,
    stars_camera_entity: Single<Entity, With<StarCamera>>,
) {
    commands
        .spawn((
            RequiredCameraComponents,
            OutlineCamera,
            OrbitCam {
                focus: Vec3::ZERO,
                target_radius: camera_settings.splash_start.radius,
                button_orbit: MouseButton::Middle,
                button_pan: MouseButton::Middle,
                modifier_pan: Some(KeyCode::ShiftLeft),
                zoom_sensitivity: CAMERA_ZOOM_SENSITIVITY,
                zoom_lower_limit: CAMERA_ZOOM_LOWER_LIMIT,
                input_control: Some(InputControl {
                    trackpad: Some(TrackpadInput {
                        sensitivity: CAMERA_TRACKPAD_SENSITIVITY,
                        ..TrackpadInput::blender_default()
                    }),
                    ..default()
                }),
                ..default()
            },
            Camera {
                order: CameraOrder::Game.order(),
                // can't obscure the star camera with this on
                clear_color: ClearColorConfig::None,
                ..default()
            },
            RenderLayer::Game.layers(),
            Smaa::default(),
            EnvironmentMapLight {
                diffuse_map: scene_assets.environment_diffuse_map.clone(),
                specular_map: scene_assets.environment_specular_map.clone(),
                intensity: light_settings.environment_map_intensity,
                ..default()
            },
        ))
        .add_child(*stars_camera_entity);
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
        camera.zoom_smoothness = camera_settings.smoothness.zoom;
        camera.pan_smoothness = camera_settings.smoothness.pan;
        camera.orbit_smoothness = camera_settings.smoothness.orbit;
    }
}
