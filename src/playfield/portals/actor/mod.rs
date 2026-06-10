mod boundary_geometry;
mod state;
mod visual_lifecycle;

use bevy::prelude::*;
pub(crate) use state::ActorPortals;

use super::settings;
use crate::state::GameState;
use crate::state::PauseState;

pub(super) struct ActorPortalPlugin;

impl Plugin for ActorPortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                settings::apply_portal_settings.run_if(in_state(GameState::InGame)),
                visual_lifecycle::init_portals.run_if(in_state(PauseState::Playing)),
                visual_lifecycle::update_approaching_portals.run_if(in_state(PauseState::Playing)),
                visual_lifecycle::update_emerging_portals.run_if(in_state(PauseState::Playing)),
                visual_lifecycle::draw_approaching_portals.run_if(in_state(GameState::InGame)),
                visual_lifecycle::draw_emerging_portals.run_if(in_state(GameState::InGame)),
            )
                .chain(),
        );
    }
}
