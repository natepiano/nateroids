use std::time::Duration;

use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_lagrange::AnimateToFit;
use bevy_lagrange::FitOverlay;
use bevy_lagrange::OrbitCam;
use bevy_lagrange::SetFitTarget;
use bevy_lagrange::ZoomToFit;

use super::constants::HOME_ANIMATION_DURATION_MS;
use super::constants::ZOOM_CONVERGENCE_RATE;
use super::constants::ZOOM_CONVERGENCE_RATE_MAX;
use super::constants::ZOOM_CONVERGENCE_RATE_MIN;
use super::constants::ZOOM_MARGIN;
use super::constants::ZOOM_MARGIN_MAX;
use super::constants::ZOOM_MARGIN_MIN;
use super::constants::ZOOM_MARGIN_TOLERANCE;
use super::constants::ZOOM_MARGIN_TOLERANCE_MAX;
use super::constants::ZOOM_MARGIN_TOLERANCE_MIN;
use super::constants::ZOOM_MAX_ITERATIONS;
use super::constants::ZOOM_MAX_ITERATIONS_MAX;
use super::constants::ZOOM_MAX_ITERATIONS_MIN;
use super::constants::ZOOM_SETTINGS_MARGIN;
use super::constants::ZOOM_TO_FIT_DURATION_MS;
use crate::input::BoundaryBoxSwitch;
use crate::input::CameraHome as CameraHomeShortcut;
use crate::input::InspectZoomSwitch;
use crate::input::ZoomToFitShortcut;
use crate::playfield::BoundaryVolume;
use crate::switches;
use crate::switches::Switch;

/// Resource tracking the currently selected entity for zoom-to-fit.
/// When `None`, Z zooms to boundary.
#[derive(Resource, Default)]
pub(super) struct ZoomTarget(pub Option<Entity>);

event!(CameraHomeEvent);
event!(InspectZoomEvent);
event!(ToggleFitTargetDebugEvent);
event!(ZoomToFitEvent);

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct ZoomSettings {
    /// Maximum iterations before giving up.
    #[inspector(min = ZOOM_MAX_ITERATIONS_MIN, max = ZOOM_MAX_ITERATIONS_MAX)]
    pub(super) max_iterations:   usize,
    #[inspector(
        min = ZOOM_MARGIN_MIN,
        max = ZOOM_MARGIN_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) margin:           f32,
    /// Margin tolerance for convergence detection (0.001 = 0.1% tolerance).
    /// Used for both balance and fit checks.
    #[inspector(
        min = ZOOM_MARGIN_TOLERANCE_MIN,
        max = ZOOM_MARGIN_TOLERANCE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) margin_tolerance: f32,
    // Zoom-to-fit convergence parameters
    /// Convergence rate for zoom-to-fit adjustments (0.18 = 18% per frame).
    #[inspector(
        min = ZOOM_CONVERGENCE_RATE_MIN,
        max = ZOOM_CONVERGENCE_RATE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) convergence_rate: f32,
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

pub(super) struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomTarget>()
            .init_resource::<ZoomSettings>()
            .add_plugins(
                ResourceInspectorPlugin::<ZoomSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectZoom)),
            )
            .add_systems(Startup, set_fit_target_debug);
        bind_action_system!(app, ZoomToFitShortcut, ZoomToFitEvent, zoom_to_fit_command);
        bind_action_system!(
            app,
            CameraHomeShortcut,
            CameraHomeEvent,
            camera_home_command
        );
        bind_action_system!(
            app,
            BoundaryBoxSwitch,
            ToggleFitTargetDebugEvent,
            toggle_fit_target_debug_command
        );
        bind_action_switch!(
            app,
            InspectZoomSwitch,
            InspectZoomEvent,
            Switch::InspectZoom
        );
    }
}

/// Reusable on-demand command for zoom-to-fit.
///
/// Uses the selected target when present; otherwise falls back to `BoundaryVolume`.
fn zoom_to_fit_command(
    mut commands: Commands,
    zoom_target: Res<ZoomTarget>,
    boundary_volume: Query<Entity, With<BoundaryVolume>>,
    camera_entity: Single<Entity, With<OrbitCam>>,
) {
    let camera_entity = *camera_entity;

    let target = if let Some(selected) = zoom_target.0 {
        selected
    } else {
        let Ok(boundary) = boundary_volume.single() else {
            warn!("No BoundaryVolume entity found");
            return;
        };
        boundary
    };

    commands.trigger(
        ZoomToFit::new(camera_entity, target)
            .margin(ZOOM_MARGIN)
            .duration(Duration::from_millis(ZOOM_TO_FIT_DURATION_MS))
            .easing(EaseFunction::Linear),
    );
    debug!("Triggered zoom-to-fit to {target:?}");
}

fn set_fit_target_debug(
    mut commands: Commands,
    camera_query: Query<Entity, With<OrbitCam>>,
    boundary_volume_query: Query<Entity, With<BoundaryVolume>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume_query.single() else {
        warn!("No BoundaryVolume entity found for fit target debug");
        return;
    };

    commands.trigger(SetFitTarget::new(camera_entity, boundary_entity));
}

/// Reusable on-demand command for camera "home" animation.
fn camera_home_command(
    mut commands: Commands,
    boundary_volume_query: Query<Entity, With<BoundaryVolume>>,
    camera_entity: Single<Entity, With<OrbitCam>>,
) {
    let camera_entity = *camera_entity;

    let Ok(boundary_entity) = boundary_volume_query.single() else {
        warn!("No BoundaryVolume entity found");
        return;
    };

    commands.trigger(
        AnimateToFit::new(camera_entity, boundary_entity)
            .yaw(0.0)
            .pitch(0.0)
            .margin(ZOOM_MARGIN)
            .duration(Duration::from_millis(HOME_ANIMATION_DURATION_MS))
            .easing(EaseFunction::QuadraticOut),
    );
}

/// Reusable on-demand command for toggling fit-target visualization.
fn toggle_fit_target_debug_command(
    mut commands: Commands,
    camera_entity: Single<Entity, With<OrbitCam>>,
    viz_query: Query<(), With<FitOverlay>>,
) {
    let camera_entity = *camera_entity;
    if viz_query.get(camera_entity).is_ok() {
        commands.entity(camera_entity).remove::<FitOverlay>();
    } else {
        commands.entity(camera_entity).insert(FitOverlay);
    }
}
