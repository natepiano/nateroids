use bevy::anti_alias::smaa::Smaa;
use bevy::prelude::*;
use bevy_mesh_outline::OutlineCamera;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera::TrackpadBehavior;

use super::RenderLayer;
use super::camera_star::StarCamera;
use super::components::RequiredCameraComponents;
use super::constants::CAMERA_ZOOM_LOWER_LIMIT;
use super::constants::CAMERA_ZOOM_SENSITIVITY;
use super::lights::LightSettings;
use super::settings::CameraSettings;
use super::support::CameraOrder;
use crate::asset_loader::SceneAssets;

pub(super) struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Update, update_environment_map_intensity); }
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
            PanOrbitCamera {
                focus: Vec3::ZERO,
                target_radius: camera_settings.splash_start_radius,
                button_orbit: MouseButton::Middle,
                button_pan: MouseButton::Middle,
                modifier_pan: Some(KeyCode::ShiftLeft),
                zoom_sensitivity: CAMERA_ZOOM_SENSITIVITY,
                zoom_lower_limit: CAMERA_ZOOM_LOWER_LIMIT,
                trackpad_behavior: TrackpadBehavior::BlenderLike {
                    modifier_pan:  Some(KeyCode::ShiftLeft),
                    modifier_zoom: Some(KeyCode::ControlLeft),
                },
                trackpad_pinch_to_zoom_enabled: true,
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
                diffuse_map: scene_assets.env_diffuse_map.clone(),
                specular_map: scene_assets.env_specular_map.clone(),
                intensity: light_settings.environment_map_intensity,
                ..default()
            },
        ))
        .add_child(*stars_camera_entity);
}

fn update_environment_map_intensity(
    light_settings: Res<LightSettings>,
    mut query: Query<&mut EnvironmentMapLight, With<Camera3d>>,
) {
    if !light_settings.is_changed() {
        return;
    }

    for mut env_light in &mut query {
        env_light.intensity = light_settings.environment_map_intensity;
    }
}
