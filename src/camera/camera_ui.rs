use bevy::prelude::*;

use super::CameraOrder;
use super::RenderLayer;
use super::RequiredCameraComponents;

pub(super) fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((
        RequiredCameraComponents,
        Camera2d,
        Camera {
            order: CameraOrder::Ui.order(),
            // can't obscure game/star cameras with this on
            clear_color: ClearColorConfig::None,
            ..default()
        },
        RenderLayer::UI.layers(),
    ));
}
