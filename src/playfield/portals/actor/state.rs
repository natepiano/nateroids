use bevy::prelude::*;

use crate::playfield::portals::Portal;

#[derive(Component, Default)]
pub(crate) struct ActorPortals {
    pub(super) approaching: Option<Portal>,
    pub(super) emerging:    Option<Portal>,
}
