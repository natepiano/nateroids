use bevy::camera::visibility::Layer;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use super::constants::CAMERA_ORDER_GAME;
use super::constants::CAMERA_ORDER_STARS;
use super::constants::CAMERA_ORDER_UI;
use super::constants::RENDER_LAYER_GAME;
use super::constants::RENDER_LAYER_STARS;
use super::constants::RENDER_LAYER_UI;

/// Camera rendering order. Higher order values render later (on top).
///
/// Render sequence:
/// 1. `Stars` (order 0): Background stars with bloom effect
/// 2. `Game` (order 1): Game objects (spaceships, asteroids, etc.)
/// 3. `Ui` (order 2): egui inspectors and UI overlays (must be last to appear on top)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CameraOrder {
    Stars,
    Game,
    Ui,
}

impl CameraOrder {
    pub(super) const fn order(self) -> isize {
        match self {
            Self::Stars => CAMERA_ORDER_STARS,
            Self::Game => CAMERA_ORDER_GAME,
            Self::Ui => CAMERA_ORDER_UI,
        }
    }
}

// RenderLayers don't propagate to scene children, they default to layer 0
// Stars camera (order 0) renders layer 0 (stars only) with bloom, clears with
// opaque background color. Game camera (order 1) renders layer 1 (game objects)
// without bloom, clears with transparent color (preserves stars but prevents
// motion trails)
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderLayer {
    Game,
    Stars,
    UI,
}

// `RenderLayer::layer_ids` returns a slice so `RenderLayers::from_layers` can
// represent future overlapping `Layer` sets.
impl RenderLayer {
    pub(crate) const fn layer_ids(self) -> &'static [Layer] {
        match self {
            Self::UI => &[RENDER_LAYER_UI],
            Self::Game => &[RENDER_LAYER_GAME],
            Self::Stars => &[RENDER_LAYER_STARS],
        }
    }

    pub(crate) fn layers(self) -> RenderLayers { RenderLayers::from_layers(self.layer_ids()) }
}
