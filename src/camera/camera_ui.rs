use bevy::prelude::*;

use crate::camera::CameraOrder;
use crate::camera::RenderLayer;
use crate::camera::RequiredCameraComponents;

pub fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: CameraOrder::Ui.order(),
            ..default()
        },
        RenderLayer::UI.layers(),
        RequiredCameraComponents,
    ));
}
