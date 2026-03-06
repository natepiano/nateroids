mod camera_game;
mod camera_star;
mod camera_ui;
mod components;
mod constants;
mod lights;
mod selection;
mod settings;
mod star_twinkling;
mod stars;
mod support;
mod zoom;

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use bevy_mesh_outline::MeshOutlinePlugin;
pub use components::RequiredCameraComponents;
pub use constants::ZOOM_MARGIN;
pub use settings::CameraSettings;
pub use support::CameraOrder;
pub use support::RenderLayer;
pub use zoom::CameraHomeEvent;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)
            .add_plugins(bevy_panorbit_camera_ext::PanOrbitCameraExtPlugin)
            .add_plugins(MeshOutlinePlugin)
            .add_plugins(camera_game::GameCameraPlugin)
            .add_plugins(camera_star::StarCameraPlugin)
            .add_plugins(zoom::ZoomPlugin)
            .add_plugins(settings::CameraSettingsPlugin)
            .add_plugins(lights::DirectionalLightsPlugin)
            .add_plugins(selection::SelectionPlugin)
            .add_plugins(star_twinkling::StarTwinklingPlugin)
            .add_plugins(stars::StarsPlugin)
            .add_systems(
                Startup,
                (
                    camera_ui::spawn_ui_camera,
                    camera_star::spawn_star_camera,
                    camera_game::spawn_game_camera,
                )
                    .chain(),
            );
    }
}
