mod constants;
mod focus_gizmo;
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
use bevy_liminal::LiminalPlugin;
pub(crate) use constants::ZOOM_MARGIN;
use focus_gizmo::FocusGizmoPlugin;
pub(crate) use game::CameraSettings;
use game::GameCameraPlugin;
use lights::DirectionalLightsPlugin;
use lights::LightSettings;
pub(crate) use rendering::RenderLayer;
use selection::SelectionPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;
pub(crate) use zoom::CameraHomeEvent;
use zoom::ZoomPlugin;

use crate::asset_loader::SceneAssets;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_plugins(LagrangePlugin)
            .add_plugins(LiminalPlugin)
            .add_plugins(GameCameraPlugin)
            .add_plugins(ZoomPlugin)
            .add_plugins(FocusGizmoPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(StarTwinklingPlugin)
            .add_plugins(StarsPlugin)
            .add_systems(Startup, spawn_camera_group)
            .add_systems(Update, star::update_bloom_settings);
    }
}

fn spawn_camera_group(
    mut commands: Commands,
    camera_settings: Res<CameraSettings>,
    scene_assets: Res<SceneAssets>,
    light_settings: Res<LightSettings>,
) {
    commands.spawn_scene_list(bsn_list![
        (
            game::game_camera(&camera_settings, &scene_assets, &light_settings)
            Children [star::star_camera(&camera_settings)]
        ),
        ui::ui_camera(),
    ]);
}
