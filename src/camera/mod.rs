mod cameras;
mod config;
mod lights;
mod star_twinkling;
mod stars;
mod zoom;

use bevy::camera::visibility::Layer;
use bevy::prelude::*;
use cameras::CamerasPlugin;
pub use cameras::ScreenSpaceBoundaryMargins;
use config::CameraConfigPlugin;
pub use config::ZoomConfig;
use lights::DirectionalLightsPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;
use zoom::ZoomPlugin;
pub use zoom::calculate_camera_radius;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraConfigPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(CamerasPlugin)
            .add_plugins(ZoomPlugin)
            .add_plugins(StarsPlugin)
            .add_plugins(StarTwinklingPlugin);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraOrder {
    Game,
    Stars,
}

impl CameraOrder {
    pub const fn order(self) -> isize {
        match self {
            CameraOrder::Game => 1,
            CameraOrder::Stars => 0,
        }
    }
}

// RenderLayers don't propagate to scene children, they default to layer 0
// Stars camera (order 0) renders layer 1 (stars only) with bloom, clears with
// opaque background color. Game camera (order 1) renders layer 0 (game objects)
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
            RenderLayer::Game => &[1],
            RenderLayer::Stars => &[0],
        }
    }
}
