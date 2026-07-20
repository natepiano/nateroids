use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

use super::RenderLayer;
use super::game::CameraSettings;
use super::rendering::CameraOrder;
use super::required_components::RequiredCameraComponents;

#[derive(Component, Reflect, Default, Clone)]
pub(super) struct StarCamera;

pub(super) fn scene(camera_settings: &CameraSettings) -> impl Scene {
    bsn! {
        RequiredCameraComponents
        Camera3d
        Camera {
            order: {CameraOrder::Stars.order()},
            clear_color: ClearColorConfig::Custom(Color::BLACK),
        }
        StarCamera
        template_value(get_bloom_settings(camera_settings))
        template_value(RenderLayer::Stars.layers())
    }
}

pub(super) fn update_bloom_settings(
    camera_settings: Res<CameraSettings>,
    mut bloom_query: Query<&mut Bloom, With<StarCamera>>,
) {
    if camera_settings.is_changed()
        && let Ok(mut old_bloom_settings) = bloom_query.single_mut()
    {
        *old_bloom_settings = get_bloom_settings(&camera_settings);
    }
}

const fn get_bloom_settings(camera_settings: &CameraSettings) -> Bloom {
    let mut new_bloom_settings = Bloom::NATURAL;
    new_bloom_settings.intensity = camera_settings.bloom_settings.intensity;
    new_bloom_settings.low_frequency_boost = camera_settings.bloom_settings.low_frequency_boost;
    new_bloom_settings.high_pass_frequency = camera_settings.bloom_settings.high_pass_frequency;
    new_bloom_settings
}
