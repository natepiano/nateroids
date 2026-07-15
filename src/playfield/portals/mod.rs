mod actor;
mod constants;
mod portal;
mod settings;

use actor::ActorPortalPlugin;
pub(crate) use actor::ActorPortals;
use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
pub(crate) use portal::Portal;
pub(crate) use settings::PortalGizmo;
use settings::PortalSettings;

use crate::input::InspectPortalSwitch;
use crate::switches;
use crate::switches::Switch;

pub(super) struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PortalGizmo>()
            .init_resource::<PortalSettings>()
            .add_plugins((
                ResourceInspectorPlugin::<PortalSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectPortals)),
                ActorPortalPlugin,
            ));
        bind_action_switch!(
            app,
            InspectPortalSwitch,
            PortalInspectorEvent,
            Switch::InspectPortals
        );
    }
}

event!(PortalInspectorEvent);
