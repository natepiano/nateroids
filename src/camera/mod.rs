mod constants;
mod game;
mod lights;
mod rendering;
mod required_components;
mod selection;
mod star;
mod star_twinkling;
mod stars;
mod ui;
mod zoom;

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use bevy_lagrange::LagrangePlugin;
use bevy_liminal::MeshOutlinePlugin;
pub(crate) use constants::ZOOM_MARGIN;
pub(crate) use game::CameraSettings;
use game::GameCameraPlugin;
use lights::DirectionalLightsPlugin;
pub(crate) use rendering::RenderLayer;
use selection::SelectionPlugin;
use star::StarCameraPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;
use ui::spawn_ui_camera;
pub(crate) use zoom::CameraHomeEvent;
use zoom::ZoomPlugin;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_plugins(LagrangePlugin)
            .add_plugins(MeshOutlinePlugin)
            .add_plugins(GameCameraPlugin)
            .add_plugins(StarCameraPlugin)
            .add_plugins(ZoomPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(StarTwinklingPlugin)
            .add_plugins(StarsPlugin)
            .add_systems(
                Startup,
                (
                    spawn_ui_camera,
                    star::spawn_star_camera,
                    game::spawn_game_camera,
                )
                    .chain(),
            );
    }
}
