use std::collections::HashMap;

use bevy::prelude::*;

pub struct SwitchesPlugin;

impl Plugin for SwitchesPlugin {
    fn build(&self, app: &mut App) { app.init_resource::<Switches>(); }
}

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct Switches {
    map: HashMap<Switch, ToggleState>,
}

#[derive(Default, Copy, Clone, Debug, Reflect, PartialEq, Eq, Hash)]
#[reflect(Debug, PartialEq, Hash)]
pub enum ToggleState {
    On,
    #[default]
    Off,
}

#[allow(dead_code, reason = "Kept for parity with hana switch model; not all helpers are used yet")]
impl Switches {
    const INSPECTOR_SWITCHES: [Switch; 14] = [
        Switch::InspectAabbConfig,
        Switch::InspectBoundary,
        Switch::InspectCameraConfig,
        Switch::InspectFocusConfig,
        Switch::InspectLights,
        Switch::InspectMissile,
        Switch::InspectNateroid,
        Switch::InspectOutline,
        Switch::InspectPlanes,
        Switch::InspectPortals,
        Switch::InspectSpaceship,
        Switch::InspectSpaceshipControl,
        Switch::InspectStarConfig,
        Switch::InspectZoomConfig,
    ];

    fn is_on(&self, switch: Switch) -> bool {
        matches!(self.map.get(&switch), Some(ToggleState::On))
    }

    fn toggle(&mut self, switch: Switch) {
        let toggle_state = match self.map.get(&switch).unwrap_or(&ToggleState::Off) {
            ToggleState::On => ToggleState::Off,
            ToggleState::Off => ToggleState::On,
        };
        self.map.insert(switch, toggle_state);
    }

    fn is_any_inspector_active(&self) -> bool {
        Self::INSPECTOR_SWITCHES
            .iter()
            .any(|switch| self.is_on(*switch))
    }

    pub fn close_all_active_inspectors(&mut self) -> bool {
        if !self.is_any_inspector_active() {
            return false;
        }

        for inspector_switch in &Self::INSPECTOR_SWITCHES {
            if self.is_on(*inspector_switch) {
                self.toggle(*inspector_switch);
            }
        }

        true
    }

    pub fn has_any_inspector_active(&self) -> bool { self.is_any_inspector_active() }

    pub fn toggle_switch(&mut self, switch: Switch) { self.toggle(switch); }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reflect(Debug, PartialEq, Hash)]
pub enum Switch {
    ShowAabbs,
    InspectAabbConfig,
    InspectBoundary,
    InspectCameraConfig,
    InspectFocusConfig,
    ShowFocus,
    InspectLights,
    InspectMissile,
    InspectNateroid,
    InspectOutline,
    InspectPlanes,
    InspectPortals,
    InspectSpaceship,
    InspectSpaceshipControl,
    InspectStarConfig,
    InspectZoomConfig,
}

#[allow(dead_code, reason = "Kept for parity with hana switch model; not used yet in nateroids")]
pub fn any_inspector_active() -> impl Fn(Res<Switches>) -> bool + Clone {
    move |switches: Res<Switches>| switches.is_any_inspector_active()
}

pub fn is_switch_on(switch: Switch) -> impl Fn(Res<Switches>) -> bool + Clone {
    move |switches: Res<Switches>| switches.is_on(switch)
}

pub fn is_switch_off(switch: Switch) -> impl Fn(Res<Switches>) -> bool + Clone {
    move |switches: Res<Switches>| !switches.is_on(switch)
}
