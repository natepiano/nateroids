mod actor;
mod constants;
mod portal;
mod settings;

pub use actor::ActorPortals;
use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
pub(super) use portal::Portal;
pub(super) use settings::PortalGizmo;
use settings::PortalSettings;

use crate::input::InspectPortalSwitch;
use crate::state::GameState;
use crate::state::PauseState;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(PortalInspectorEvent);

pub(super) struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PortalGizmo>()
            .init_resource::<PortalSettings>()
            .add_plugins(
                ResourceInspectorPlugin::<PortalSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectPortals)),
            )
            .add_systems(
                Update,
                (
                    settings::apply_portal_settings.run_if(in_state(GameState::InGame)),
                    actor::init_portals.run_if(in_state(PauseState::Playing)),
                    actor::update_approaching_portals.run_if(in_state(PauseState::Playing)),
                    actor::update_emerging_portals.run_if(in_state(PauseState::Playing)),
                    actor::draw_approaching_portals.run_if(in_state(GameState::InGame)),
                    actor::draw_emerging_portals.run_if(in_state(GameState::InGame)),
                )
                    .chain(),
            );
        bind_action_switch!(
            app,
            InspectPortalSwitch,
            PortalInspectorEvent,
            Switch::InspectPortals
        );
    }
}
