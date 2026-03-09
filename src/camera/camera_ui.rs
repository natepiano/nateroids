use bevy::prelude::*;

use super::components::RequiredCameraComponents;
use super::support::CameraOrder;
use super::RenderLayer;

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
