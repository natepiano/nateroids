mod cameras;
mod config;
mod constants;
mod lights;
mod move_queue;
mod pan_orbit_ext;
mod star_twinkling;
mod stars;
mod zoom;

use bevy::camera::visibility::Layer;
use bevy::prelude::*;
use cameras::CamerasPlugin;
pub use cameras::Edge;
pub use cameras::ScreenSpaceBoundary;
pub use cameras::calculate_home_radius;
pub use config::CameraConfig;
use config::CameraConfigPlugin;
pub use config::ZoomConfig;
use lights::DirectionalLightsPlugin;
pub use move_queue::CameraMove;
pub use move_queue::CameraMoveList;
use move_queue::MoveQueuePlugin;
pub use pan_orbit_ext::PanOrbitCameraExt;
use star_twinkling::StarTwinklingPlugin;
use stars::StarsPlugin;
use zoom::ZoomPlugin;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraConfigPlugin)
            .add_plugins(CamerasPlugin)
            .add_plugins(DirectionalLightsPlugin)
            .add_plugins(MoveQueuePlugin)
            .add_plugins(StarTwinklingPlugin)
            .add_plugins(StarsPlugin)
            .add_plugins(ZoomPlugin);
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
