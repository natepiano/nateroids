mod boundary;
mod boundary_face;
mod constants;
mod portals;

use bevy::prelude::*;
pub(crate) use boundary::Boundary;
use boundary::BoundaryPlugin;
pub(crate) use boundary::BoundaryVolume;
pub(crate) use boundary::GridFlash;
pub(crate) use portals::ActorPortals;
use portals::PortalPlugin;

pub(crate) struct PlayfieldPlugin;

impl Plugin for PlayfieldPlugin {
    fn build(&self, app: &mut App) { app.add_plugins(BoundaryPlugin).add_plugins(PortalPlugin); }
}
