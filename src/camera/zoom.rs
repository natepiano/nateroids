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

use super::RenderLayer;
use super::constants::EDGE_MARKER_FONT_SIZE;
use super::constants::EDGE_MARKER_SPHERE_RADIUS;
use super::constants::FOCUS_GIZMO_COLOR;
use super::constants::FOCUS_GIZMO_DEFAULT_CAMERA_RADIUS;
use super::constants::FOCUS_GIZMO_LINE_WIDTH;
use super::constants::FOCUS_GIZMO_LINE_WIDTH_MAX;
use super::constants::FOCUS_GIZMO_LINE_WIDTH_MIN;
use super::constants::FOCUS_GIZMO_SPHERE_RADIUS_MAX;
use super::constants::FOCUS_GIZMO_SPHERE_RADIUS_MIN;
use super::constants::HOME_ANIMATION_DURATION_MS;
use super::constants::ZOOM_MARGIN;
use super::constants::ZOOM_TO_FIT_DURATION_MS;
use crate::input::BoundaryBoxSwitch;
use crate::input::CameraHome as CameraHomeShortcut;
use crate::input::InspectFocusSwitch;
use crate::input::ShowFocusSwitch;
use crate::input::ZoomToFitShortcut;
use crate::playfield::BoundaryVolume;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

/// Resource tracking the currently selected entity for zoom-to-fit.
/// When `None`, Z zooms to boundary.
#[derive(Resource, Default)]
pub(super) struct ZoomTarget(pub Option<Entity>);

event!(CameraHomeEvent);
event!(FocusInspectorEvent);
event!(ShowFocusEvent);
event!(ToggleFitTargetDebugEvent);
event!(ZoomToFitEvent);

#[derive(Default, Reflect, GizmoConfigGroup)]
struct FocusGizmo {}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct FocusSettings {
    color:         Color,
    #[inspector(
        min = FOCUS_GIZMO_LINE_WIDTH_MIN,
        max = FOCUS_GIZMO_LINE_WIDTH_MAX,
        display = NumberDisplay::Slider
    )]
    line_width:    f32,
    #[inspector(
        min = FOCUS_GIZMO_SPHERE_RADIUS_MIN,
        max = FOCUS_GIZMO_SPHERE_RADIUS_MAX,
        display = NumberDisplay::Slider
    )]
    sphere_radius: f32,
}

impl Default for FocusSettings {
    fn default() -> Self {
        Self {
            color:         FOCUS_GIZMO_COLOR,
            line_width:    FOCUS_GIZMO_LINE_WIDTH,
            sphere_radius: EDGE_MARKER_SPHERE_RADIUS,
        }
    }
}

/// Stores the calculated world-space sphere radius that maintains constant screen-space size
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
struct FocusGizmoState {
    /// World-space radius scaled to appear constant size on screen
    sphere_radius: f32,
}

/// Marker component for the focus distance label
#[derive(Component)]
struct FocusDistanceLabel;

pub(super) struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FocusGizmo>()
            .init_resource::<ZoomTarget>()
            .init_resource::<FocusSettings>()
            .init_resource::<FocusGizmoState>()
            .add_plugins(
                ResourceInspectorPlugin::<FocusSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectFocus)),
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
            InspectFocusSwitch,
            FocusInspectorEvent,
            Switch::InspectFocus
        );
        bind_action_switch!(app, ShowFocusSwitch, ShowFocusEvent, Switch::ShowFocus);
        app.add_systems(
            Update,
            apply_focus_settings.run_if(resource_changed::<FocusSettings>),
        )
        .add_systems(Update, update_focus_gizmo_state)
        .add_systems(
            Update,
            (
                draw_camera_focus_gizmo.run_if(switches::is_switch_on(Switch::ShowFocus)),
                cleanup_focus_labels.run_if(switches::is_switch_off(Switch::ShowFocus)),
            ),
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

fn apply_focus_settings(
    mut config_store: ResMut<GizmoConfigStore>,
    focus_settings: Res<FocusSettings>,
) {
    let (gizmo_config, _) = config_store.config_mut::<FocusGizmo>();
    gizmo_config.line.width = focus_settings.line_width;
    gizmo_config.render_layers = RenderLayer::Game.layers();
}

fn update_focus_gizmo_state(
    camera_query: Query<&OrbitCam, With<Camera>>,
    camera_changed: Query<(), (With<Camera>, Changed<OrbitCam>)>,
    focus_settings: Res<FocusSettings>,
    mut focus_gizmo_state: ResMut<FocusGizmoState>,
) {
    if camera_changed.is_empty() && !focus_settings.is_changed() {
        return;
    }

    if let Ok(orbit_cam) = camera_query.single() {
        let camera_radius = orbit_cam
            .radius
            .unwrap_or(FOCUS_GIZMO_DEFAULT_CAMERA_RADIUS);
        focus_gizmo_state.sphere_radius =
            focus_settings.sphere_radius * (camera_radius / FOCUS_GIZMO_DEFAULT_CAMERA_RADIUS);
    }
}

fn draw_camera_focus_gizmo(
    mut commands: Commands,
    mut gizmos: Gizmos<FocusGizmo>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrbitCam)>,
    focus_settings: Res<FocusSettings>,
    focus_gizmo_state: Res<FocusGizmoState>,
    mut label_query: Query<(&mut Text, &mut Node, &mut TextColor), With<FocusDistanceLabel>>,
) {
    if let Ok((cam, cam_transform, orbit_cam)) = camera_query.single() {
        let focus = orbit_cam.target_focus;

        gizmos.sphere(focus, focus_gizmo_state.sphere_radius, focus_settings.color);
        gizmos.arrow(Vec3::ZERO, focus, focus_settings.color);

        let distance = focus.length();
        let text = format!("{distance:.1}");

        let arrow_dir = focus.normalize_or_zero();
        let along_arrow_offset = focus_gizmo_state.sphere_radius.mul_add(2.0, 20.0);
        let label_world_pos = focus + (arrow_dir * along_arrow_offset);

        if let Ok(label_screen_pos) = cam.world_to_viewport(cam_transform, label_world_pos) {
            if let Ok((mut label_text, mut node, mut text_color)) = label_query.single_mut() {
                label_text.0.clone_from(&text);
                text_color.0 = focus_settings.color;
                node.left = Val::Px(label_screen_pos.x);
                node.top = Val::Px(label_screen_pos.y);
            } else {
                commands.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: EDGE_MARKER_FONT_SIZE,
                        ..default()
                    },
                    TextColor(focus_settings.color),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(label_screen_pos.x),
                        top: Val::Px(label_screen_pos.y),
                        ..default()
                    },
                    RenderLayer::UI.layers(),
                    FocusDistanceLabel,
                ));
            }
        }
    }
}

fn cleanup_focus_labels(
    mut commands: Commands,
    label_query: Query<Entity, With<FocusDistanceLabel>>,
) {
    for entity in &label_query {
        commands.entity(entity).despawn();
    }
}
