mod boundary;
mod boundary_face;
mod planes;
mod portals;
mod screen_boundary;
mod types;

use bevy::prelude::*;

pub use crate::playfield::boundary::Boundary;
use crate::playfield::boundary::BoundaryPlugin;
use crate::playfield::planes::PlanesPlugin;
pub use crate::playfield::portals::ActorPortals;
use crate::playfield::portals::PortalPlugin;
use crate::playfield::screen_boundary::ScreenBoundaryPlugin;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(PortalPlugin)
            .add_plugins(ScreenBoundaryPlugin);
    }
}
