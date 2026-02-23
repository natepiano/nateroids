use super::constants::EDGE_MARKER_FONT_SIZE;
use super::constants::EDGE_MARKER_SPHERE_RADIUS;
use super::constants::HOME_ANIMATION_DURATION_MS;
use super::constants::ZOOM_MARGIN;
use super::constants::ZOOM_TO_FIT_DURATION_MS;
use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_enhanced_input::action::events as input_events;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::AnimateToFit;
use bevy_panorbit_camera_ext::FitTargetGizmo;
use bevy_panorbit_camera_ext::FitTargetVisualizationPlugin;
use bevy_panorbit_camera_ext::SetFitTarget;
use bevy_panorbit_camera_ext::ZoomToFit as ZoomToFitEvent;
/// Resource tracking the currently selected entity for zoom-to-fit.
/// When `None`, Z zooms to boundary.
#[derive(Resource, Default)]
pub struct ZoomTarget(pub Option<Entity>);
use crate::camera::RenderLayer;
use crate::input::BoundaryBoxToggle;
use crate::input::CameraHome;
use crate::input::FocusConfigInspectorToggle;
use crate::input::ShowFocusToggle;
use crate::input::ZoomToFitShortcut;
use crate::playfield::BoundaryVolume;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct FocusGizmo {}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct FocusConfig {
    color: Color,
    #[inspector(min = 0.1, max = 10.0, display = NumberDisplay::Slider)]
    line_width: f32,
    #[inspector(min = 0.1, max = 50.0, display = NumberDisplay::Slider)]
    sphere_radius: f32,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            color: Color::srgb(1.0, 0.0, 0.0),
            line_width: 2.0,
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
        app.add_plugins(FitTargetVisualizationPlugin)
            .init_gizmo_group::<FocusGizmo>()
            .init_resource::<ZoomTarget>()
            .init_resource::<FocusConfig>()
            .init_resource::<FocusGizmoState>()
            .add_plugins(
                ResourceInspectorPlugin::<FocusConfig>::default()
                    .run_if(switches::is_switch_on(Switch::InspectFocusConfig)),
            )
            .add_systems(Startup, set_fit_target_debug)
            .add_observer(on_camera_home_input)
            .add_observer(on_zoom_to_fit_input)
            .add_observer(on_toggle_fit_target_debug_input)
            .add_observer(on_toggle_focus_config_inspector_input)
            .add_observer(on_toggle_show_focus_input)
            .add_systems(
                Update,
                apply_focus_config.run_if(resource_changed::<FocusConfig>),
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

/// System that triggers zoom-to-fit when the user presses the zoom action.
/// Zooms to the selected entity if one exists, otherwise to the `BoundaryVolume`.
fn on_zoom_to_fit_input(
    _trigger: On<input_events::Start<ZoomToFitShortcut>>,
    mut commands: Commands,
    zoom_target: Res<ZoomTarget>,
    boundary_volume: Query<Entity, With<BoundaryVolume>>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let target = if let Some(selected) = zoom_target.0 {
        selected
    } else {
        let Ok(boundary) = boundary_volume.single() else {
            warn!("No BoundaryVolume entity found");
            return;
        };
        boundary
    };

    commands.trigger(ZoomToFitEvent::new(
        camera_entity,
        target,
        ZOOM_MARGIN,
        ZOOM_TO_FIT_DURATION_MS,
        EaseFunction::Linear,
    ));
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

fn on_camera_home_input(
    _trigger: On<input_events::Start<CameraHome>>,
    mut commands: Commands,
    boundary_volume_query: Query<Entity, With<BoundaryVolume>>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume_query.single() else {
        warn!("No BoundaryVolume entity found");
        return;
    };

    commands.trigger(AnimateToFit::new(
        camera_entity,
        boundary_entity,
        0.0,
        0.0,
        ZOOM_MARGIN,
        HOME_ANIMATION_DURATION_MS,
        EaseFunction::QuadraticOut,
    ));
}

fn on_toggle_fit_target_debug_input(
    _trigger: On<input_events::Start<BoundaryBoxToggle>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = config_store.config_mut::<FitTargetGizmo>();
    config.enabled = !config.enabled;
    info!("Fit target visualization: {}", config.enabled);
}

fn apply_focus_config(mut config_store: ResMut<GizmoConfigStore>, config: Res<FocusConfig>) {
    let (gizmo_config, _) = config_store.config_mut::<FocusGizmo>();
    gizmo_config.line.width = config.line_width;
    gizmo_config.render_layers = RenderLayer::Game.layers();
}

fn update_focus_gizmo_state(
    camera_query: Query<&PanOrbitCamera, With<Camera>>,
    camera_changed: Query<(), (With<Camera>, Changed<PanOrbitCamera>)>,
    config: Res<FocusConfig>,
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
    config: Res<FocusConfig>,
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

fn on_toggle_focus_config_inspector_input(
    _trigger: On<input_events::Start<FocusConfigInspectorToggle>>,
    mut switches: ResMut<Switches>,
) {
    switches.toggle_switch(Switch::InspectFocusConfig);
}

fn on_toggle_show_focus_input(
    _trigger: On<input_events::Start<ShowFocusToggle>>,
    mut switches: ResMut<Switches>,
) {
    switches.toggle_switch(Switch::ShowFocus);
}
