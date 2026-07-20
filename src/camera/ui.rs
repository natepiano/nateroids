use bevy::prelude::*;

use super::RenderLayer;
use super::rendering::CameraOrder;
use super::required_components::RequiredCameraComponents;

pub(super) fn scene() -> impl Scene {
    bsn! {
        RequiredCameraComponents
        Camera2d
        Camera {
            order: {CameraOrder::Ui.order()},
            // can't obscure game/star cameras with this on
            clear_color: ClearColorConfig::None,
        }
        template_value(RenderLayer::UI.layers())
    }
}
