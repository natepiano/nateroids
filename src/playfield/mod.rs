mod boundary;
mod boundary_face;
mod constants;
mod planes;
mod portals;
mod types;

use bevy::prelude::*;
pub use boundary::Boundary;
use boundary::BoundaryPlugin;
pub use boundary::BoundaryVolume;
use planes::PlanesPlugin;
pub use portals::ActorPortals;
use portals::PortalPlugin;
pub use types::GridFlash;

pub struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BoundaryPlugin)
            .add_plugins(PlanesPlugin)
            .add_plugins(PortalPlugin);
    }
}
