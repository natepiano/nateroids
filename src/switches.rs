use std::collections::HashMap;

use bevy::prelude::*;

use crate::constants::INSPECTOR_SWITCHES;

/// Wires an input action to a switch toggle through an intermediate event.
///
/// Registers two observers:
/// 1. `On<Start<Action>>` → triggers `Event`
/// 2. `On<Event>` → toggles the switch
///
/// The intermediate event decouples the keyboard input from the switch toggle,
/// making switches BRP-triggerable via `world.trigger_event`.
///
/// Use with `action!` and `event!` to generate the action and event structs.
///
/// ```rust
/// bind_action_switch!(app, MySwitch, MySwitchEvent, Switch::MySwitch);
/// ```
macro_rules! bind_action_switch {
    ($app:expr, $action:ty, $event:ty, $switch:expr) => {
        $app.add_observer(
            |_: On<bevy_enhanced_input::action::events::Start<$action>>, mut commands: Commands| {
                commands.trigger(<$event>::default());
            },
        );
        let switch = $switch;
        $app.add_observer(move |_: On<$event>, mut switches: ResMut<Switches>| {
            switches.toggle_switch(switch);
        });
    };
}

pub(crate) struct SwitchesPlugin;

impl Plugin for SwitchesPlugin {
    fn build(&self, app: &mut App) { app.init_resource::<Switches>(); }
}

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub(crate) struct Switches {
    map: HashMap<Switch, ToggleState>,
}

#[derive(Default, Copy, Clone, Debug, Reflect, PartialEq, Eq, Hash)]
#[reflect(Debug, PartialEq, Hash)]
pub(crate) enum ToggleState {
    On,
    #[default]
    Off,
}

impl Switches {
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
        INSPECTOR_SWITCHES.iter().any(|switch| self.is_on(*switch))
    }

    /// Turn off every active inspector. Returns `true` if any were closed.
    pub(crate) fn close_all_active_inspectors(&mut self) -> bool {
        if !self.is_any_inspector_active() {
            return false;
        }
        for inspector_switch in &INSPECTOR_SWITCHES {
            if self.is_on(*inspector_switch) {
                self.toggle(*inspector_switch);
            }
        }
        true
    }

    pub(crate) fn toggle_switch(&mut self, switch: Switch) { self.toggle(switch); }

    pub(crate) fn is_switch_on(&self, switch: Switch) -> bool { self.is_on(switch) }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reflect(Debug, PartialEq, Hash)]
pub(crate) enum Switch {
    ShowAabbs,
    ShowPhysicsDebug,
    InspectAabb,
    InspectBoundary,
    InspectCamera,
    InspectFocus,
    ShowFocus,
    InspectLights,
    InspectMissile,
    InspectNateroid,
    InspectOutline,
    InspectPortals,
    InspectSpaceship,
    InspectSpaceshipControl,
    InspectStar,
    InspectZoom,
}

pub(crate) fn is_switch_on(switch: Switch) -> impl Fn(Res<Switches>) -> bool + Clone {
    move |switches: Res<Switches>| switches.is_on(switch)
}

pub(crate) fn is_switch_off(switch: Switch) -> impl Fn(Res<Switches>) -> bool + Clone {
    move |switches: Res<Switches>| !switches.is_on(switch)
}
