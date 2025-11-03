mod cameras;
mod config;
mod lights;
mod star_twinkling;
mod stars;

use bevy::{
    camera::visibility::Layer,
    prelude::*,
};

use cameras::CamerasPlugin;
use config::CameraConfigPlugin;
use lights::DirectionalLightsPlugin;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraConfigPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(CamerasPlugin)
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

// Bevy 0.17 fixed the RenderLayers bug, so now we can align camera order with
// render layers Stars camera (order 0) renders layer 0
// Game camera (order 1) renders layer 1
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    Both,
    Game,
    Stars,
}

// returning the array rather than just one in case we have more complex
// situations in the future that require overlapping layers
impl RenderLayer {
    pub const fn layers(self) -> &'static [Layer] {
        match self {
            RenderLayer::Both => &[0, 1],
            RenderLayer::Game => &[0],
            RenderLayer::Stars => &[1],
        }
    }
}
