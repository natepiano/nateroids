mod boundary;
mod boundary_face;
mod planes;
mod portals;

use bevy::prelude::*;

pub use crate::playfield::boundary::Boundary;
use crate::playfield::boundary::BoundaryPlugin;
use crate::playfield::planes::PlanesPlugin;
pub use crate::playfield::portals::ActorPortals;
use crate::playfield::portals::PortalPlugin;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(PortalPlugin);
    }
}
