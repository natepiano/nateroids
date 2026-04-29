use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::camera::constants::ZOOM_CONVERGENCE_RATE;
use crate::camera::constants::ZOOM_CONVERGENCE_RATE_MAX;
use crate::camera::constants::ZOOM_CONVERGENCE_RATE_MIN;
use crate::camera::constants::ZOOM_MARGIN_MAX;
use crate::camera::constants::ZOOM_MARGIN_MIN;
use crate::camera::constants::ZOOM_MARGIN_TOLERANCE;
use crate::camera::constants::ZOOM_MARGIN_TOLERANCE_MAX;
use crate::camera::constants::ZOOM_MARGIN_TOLERANCE_MIN;
use crate::camera::constants::ZOOM_MAX_ITERATIONS;
use crate::camera::constants::ZOOM_MAX_ITERATIONS_MAX;
use crate::camera::constants::ZOOM_MAX_ITERATIONS_MIN;
use crate::camera::constants::ZOOM_SETTINGS_MARGIN;
use crate::input::InspectZoomSwitch;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(InspectZoomEvent);

pub(super) struct ZoomSettingsInspectorPlugin;

impl Plugin for ZoomSettingsInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<ZoomSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectZoom)),
        )
        .init_resource::<ZoomSettings>();
        bind_action_switch!(
            app,
            InspectZoomSwitch,
            InspectZoomEvent,
            Switch::InspectZoom
        );
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct ZoomSettings {
    /// Maximum iterations before giving up.
    #[inspector(min = ZOOM_MAX_ITERATIONS_MIN, max = ZOOM_MAX_ITERATIONS_MAX)]
    pub max_iterations:   usize,
    #[inspector(
        min = ZOOM_MARGIN_MIN,
        max = ZOOM_MARGIN_MAX,
        display = NumberDisplay::Slider
    )]
    pub margin:           f32,
    /// Margin tolerance for convergence detection (0.001 = 0.1% tolerance).
    /// Used for both balance and fit checks.
    #[inspector(
        min = ZOOM_MARGIN_TOLERANCE_MIN,
        max = ZOOM_MARGIN_TOLERANCE_MAX,
        display = NumberDisplay::Slider
    )]
    pub margin_tolerance: f32,
    // Zoom-to-fit convergence parameters
    /// Convergence rate for zoom-to-fit adjustments (0.18 = 18% per frame).
    #[inspector(
        min = ZOOM_CONVERGENCE_RATE_MIN,
        max = ZOOM_CONVERGENCE_RATE_MAX,
        display = NumberDisplay::Slider
    )]
    pub convergence_rate: f32,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            max_iterations:   ZOOM_MAX_ITERATIONS,
            margin:           ZOOM_SETTINGS_MARGIN,
            margin_tolerance: ZOOM_MARGIN_TOLERANCE,
            convergence_rate: ZOOM_CONVERGENCE_RATE,
        }
    }
}
