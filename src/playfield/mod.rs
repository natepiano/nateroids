mod boundary;
mod boundary_face;
mod constants;
mod portals;

use bevy::prelude::*;
pub use boundary::Boundary;
use boundary::BoundaryPlugin;
pub use boundary::BoundaryVolume;
pub use boundary::GridFlash;
pub use portals::ActorPortals;
use portals::PortalPlugin;

pub(crate) struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) { app.add_plugins(BoundaryPlugin).add_plugins(PortalPlugin); }
}
