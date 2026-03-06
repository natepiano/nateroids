use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

use super::settings::CameraSettings;
use crate::camera::CameraOrder;
use crate::camera::RenderLayer;
use crate::camera::RequiredCameraComponents;

#[derive(Component, Reflect)]
pub struct StarCamera;

pub struct StarCameraPlugin;

impl Plugin for StarCameraPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Update, update_bloom_settings); }
}

pub fn spawn_star_camera(mut commands: Commands, camera_config: Res<CameraSettings>) {
    commands.spawn((
        RequiredCameraComponents,
        Camera3d::default(),
        Camera {
            order: CameraOrder::Stars.order(),
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        StarCamera,
        get_bloom_settings(camera_config),
        RenderLayer::Stars.layers(),
    ));
}

fn update_bloom_settings(
    camera_config: Res<CameraSettings>,
    mut q_current_settings: Query<&mut Bloom, With<StarCamera>>,
) {
    if camera_config.is_changed()
        && let Ok(mut old_bloom_settings) = q_current_settings.single_mut()
    {
        *old_bloom_settings = get_bloom_settings(camera_config);
    }
}

fn get_bloom_settings(camera_config: Res<CameraSettings>) -> Bloom {
    let mut new_bloom_settings = Bloom::NATURAL;
    new_bloom_settings.intensity = camera_config.bloom_intensity;
    new_bloom_settings.low_frequency_boost = camera_config.bloom_low_frequency_boost;
    new_bloom_settings.high_pass_frequency = camera_config.bloom_high_pass_frequency;
    new_bloom_settings
}
