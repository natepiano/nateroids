mod config;
mod constants;
mod game_camera;
mod lights;
mod selection;
mod star_camera;
mod star_twinkling;
mod stars;
mod zoom;

use bevy::camera::visibility::Layer;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera_ext::PanOrbitCameraExtPlugin;
pub use config::CameraConfig;
use config::CameraConfigPlugin;
pub use constants::ZOOM_MARGIN;
use game_camera::GameCameraPlugin;
use game_camera::set_fit_target_debug;
use game_camera::spawn_panorbit_camera;
use game_camera::spawn_ui_camera;
use lights::DirectionalLightsPlugin;
use selection::SelectionPlugin;
use star_camera::StarCameraPlugin;
use star_camera::spawn_star_camera;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraConfigPlugin)
            .add_plugins(PanOrbitCameraPlugin)
            .add_plugins(MeshPickingPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(StarCameraPlugin)
            .add_plugins(GameCameraPlugin)
            .add_plugins(PanOrbitCameraExtPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(StarTwinklingPlugin)
            .add_plugins(StarsPlugin)
            .add_systems(
                Startup,
                (
                    spawn_ui_camera,
                    spawn_star_camera,
                    spawn_panorbit_camera,
                    set_fit_target_debug,
                )
                    .chain(),
            );
    }
}

/// Camera rendering order. Higher order values render later (on top).
///
/// Render sequence:
/// 1. `Stars` (order 0): Background stars with bloom effect
/// 2. `Game` (order 1): Game objects (spaceships, asteroids, etc.)
/// 3. `Ui` (order 2): egui inspectors and UI overlays (must be last to appear on top)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraOrder {
    Stars,
    Game,
    Ui,
}

impl CameraOrder {
    pub const fn order(self) -> isize {
        match self {
            Self::Stars => 0,
            Self::Game => 1,
            Self::Ui => 2,
        }
    }
}

// RenderLayers don't propagate to scene children, they default to layer 0
// Stars camera (order 0) renders layer 0 (stars only) with bloom, clears with
// opaque background color. Game camera (order 1) renders layer 1 (game objects)
// without bloom, clears with transparent color (preserves stars but prevents
// motion trails)
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    Game,
    Stars,
}

// returning the array rather than just one in case we have more complex
// situations in the future that require overlapping layers
impl RenderLayer {
    pub const fn layers(self) -> &'static [Layer] {
        match self {
            Self::Game => &[1],
            Self::Stars => &[0],
        }
    }
}
