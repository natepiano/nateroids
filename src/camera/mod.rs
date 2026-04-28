mod camera_game;
mod camera_star;
mod camera_ui;
mod constants;
mod lights;
mod rendering;
mod required_camera_components;
mod selection;
mod settings;
mod star_twinkling;
mod stars;
mod zoom;

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use bevy_liminal::MeshOutlinePlugin;
use camera_game::GameCameraPlugin;
use camera_star::StarCameraPlugin;
pub(crate) use constants::ZOOM_MARGIN;
use lights::DirectionalLightsPlugin;
pub(crate) use rendering::RenderLayer;
use selection::SelectionPlugin;
pub(crate) use settings::CameraSettings;
use settings::CameraSettingsPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;
pub(crate) use zoom::CameraHomeEvent;
use zoom::ZoomPlugin;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_plugins(bevy_lagrange::LagrangePlugin)
            .add_plugins(MeshOutlinePlugin)
            .add_plugins(GameCameraPlugin)
            .add_plugins(StarCameraPlugin)
            .add_plugins(ZoomPlugin)
            .add_plugins(CameraSettingsPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(StarTwinklingPlugin)
            .add_plugins(StarsPlugin)
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
