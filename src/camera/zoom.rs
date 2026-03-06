use std::time::Duration;

use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::AnimateToFit;
use bevy_panorbit_camera_ext::FitVisualization;
use bevy_panorbit_camera_ext::SetFitTarget;
use bevy_panorbit_camera_ext::ZoomToFit as PanOrbitZoomToFit;

use super::constants::EDGE_MARKER_FONT_SIZE;
use super::constants::EDGE_MARKER_SPHERE_RADIUS;
use super::constants::HOME_ANIMATION_DURATION_MS;
use super::constants::ZOOM_MARGIN;
use super::constants::ZOOM_TO_FIT_DURATION_MS;
/// Resource tracking the currently selected entity for zoom-to-fit.
/// When `None`, Z zooms to boundary.
#[derive(Resource, Default)]
pub struct ZoomTarget(pub Option<Entity>);
use crate::camera::RenderLayer;
use crate::input::BoundaryBoxSwitch;
use crate::input::CameraHome as CameraHomeShortcut;
use crate::input::InspectFocusSwitch;
use crate::input::ShowFocusSwitch;
use crate::input::ZoomToFitShortcut;
use crate::playfield::BoundaryVolume;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

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
    #[inspector(min = 0.1, max = 10.0, display = NumberDisplay::Slider)]
    line_width:    f32,
    #[inspector(min = 0.1, max = 50.0, display = NumberDisplay::Slider)]
    sphere_radius: f32,
}

impl Default for FocusSettings {
    fn default() -> Self {
        Self {
            color:         Color::srgb(1.0, 0.0, 0.0),
            line_width:    2.0,
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

pub struct ZoomPlugin;

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
    camera_entity: Single<Entity, With<PanOrbitCamera>>,
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
        PanOrbitZoomToFit::new(camera_entity, target)
            .margin(ZOOM_MARGIN)
            .duration(Duration::from_millis(ZOOM_TO_FIT_DURATION_MS))
            .easing(EaseFunction::Linear),
    );
    debug!("Triggered zoom-to-fit to {target:?}");
}

fn set_fit_target_debug(
    mut commands: Commands,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
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
    camera_entity: Single<Entity, With<PanOrbitCamera>>,
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
    camera_entity: Single<Entity, With<PanOrbitCamera>>,
    viz_query: Query<(), With<FitVisualization>>,
) {
    let camera_entity = *camera_entity;
    if viz_query.get(camera_entity).is_ok() {
        commands.entity(camera_entity).remove::<FitVisualization>();
    } else {
        commands.entity(camera_entity).insert(FitVisualization);
    }
}

fn apply_focus_settings(mut config_store: ResMut<GizmoConfigStore>, config: Res<FocusSettings>) {
    let (gizmo_config, _) = config_store.config_mut::<FocusGizmo>();
    gizmo_config.line.width = config.line_width;
    gizmo_config.render_layers = RenderLayer::Game.layers();
}

fn update_focus_gizmo_state(
    camera_query: Query<&PanOrbitCamera, With<Camera>>,
    camera_changed: Query<(), (With<Camera>, Changed<PanOrbitCamera>)>,
    config: Res<FocusSettings>,
    mut state: ResMut<FocusGizmoState>,
) {
    if camera_changed.is_empty() && !config.is_changed() {
        return;
    }

    if let Ok(pan_orbit) = camera_query.single() {
        let camera_radius = pan_orbit.radius.unwrap_or(100.0);
        state.sphere_radius = config.sphere_radius * (camera_radius / 100.0);
    }
}

fn draw_camera_focus_gizmo(
    mut commands: Commands,
    mut gizmos: Gizmos<FocusGizmo>,
    camera_query: Query<(&Camera, &GlobalTransform, &PanOrbitCamera)>,
    config: Res<FocusSettings>,
    state: Res<FocusGizmoState>,
    mut label_query: Query<(&mut Text, &mut Node, &mut TextColor), With<FocusDistanceLabel>>,
) {
    if let Ok((cam, cam_transform, pan_orbit)) = camera_query.single() {
        let focus = pan_orbit.target_focus;

        gizmos.sphere(focus, state.sphere_radius, config.color);
        gizmos.arrow(Vec3::ZERO, focus, config.color);

        let distance = focus.length();
        let text = format!("{distance:.1}");

        let arrow_dir = focus.normalize_or_zero();
        let along_arrow_offset = state.sphere_radius.mul_add(2.0, 20.0);
        let label_world_pos = focus + (arrow_dir * along_arrow_offset);

        if let Ok(label_screen_pos) = cam.world_to_viewport(cam_transform, label_world_pos) {
            if let Ok((mut label_text, mut node, mut text_color)) = label_query.single_mut() {
                label_text.0.clone_from(&text);
                text_color.0 = config.color;
                node.left = Val::Px(label_screen_pos.x);
                node.top = Val::Px(label_screen_pos.y);
            } else {
                commands.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: EDGE_MARKER_FONT_SIZE,
                        ..default()
                    },
                    TextColor(config.color),
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
