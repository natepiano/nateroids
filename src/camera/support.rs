use bevy::camera::visibility::Layer;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

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
    UI,
}

// returning the array rather than just one in case we have more complex
// situations in the future that require overlapping layers
impl RenderLayer {
    pub const fn layer_ids(self) -> &'static [Layer] {
        match self {
            Self::UI => &[2],
            Self::Game => &[1],
            Self::Stars => &[0],
        }
    }

    pub fn layers(self) -> RenderLayers { RenderLayers::from_layers(self.layer_ids()) }
}
