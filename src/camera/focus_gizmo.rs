use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_lagrange::OrbitCam;

use super::RenderLayer;
use super::constants::EDGE_MARKER_FONT_SIZE;
use super::constants::EDGE_MARKER_SPHERE_RADIUS;
use super::constants::FOCUS_GIZMO_COLOR;
use super::constants::FOCUS_GIZMO_DEFAULT_CAMERA_RADIUS;
use super::constants::FOCUS_GIZMO_DISTANCE_LABEL_OFFSET;
use super::constants::FOCUS_GIZMO_LABEL_RADIUS_MULTIPLIER;
use super::constants::FOCUS_GIZMO_LINE_WIDTH;
use super::constants::FOCUS_GIZMO_LINE_WIDTH_MAX;
use super::constants::FOCUS_GIZMO_LINE_WIDTH_MIN;
use super::constants::FOCUS_GIZMO_SPHERE_RADIUS_MAX;
use super::constants::FOCUS_GIZMO_SPHERE_RADIUS_MIN;
use crate::input::InspectFocusSwitch;
use crate::input::ShowFocusSwitch;
use crate::switches;
use crate::switches::Switch;

pub(super) struct FocusGizmoPlugin;

impl Plugin for FocusGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FocusGizmo>()
            .init_resource::<FocusSettings>()
            .init_resource::<FocusGizmoState>()
            .add_plugins(
                ResourceInspectorPlugin::<FocusSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectFocus)),
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

event!(FocusInspectorEvent);
event!(ShowFocusEvent);

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

fn apply_focus_settings(
    mut gizmo_config_store: ResMut<GizmoConfigStore>,
    focus_settings: Res<FocusSettings>,
) {
    let (gizmo_config, _) = gizmo_config_store.config_mut::<FocusGizmo>();
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
    if let Ok((camera, camera_transform, orbit_cam)) = camera_query.single() {
        let focus = orbit_cam.target_focus;

        gizmos.sphere(focus, focus_gizmo_state.sphere_radius, focus_settings.color);
        gizmos.arrow(Vec3::ZERO, focus, focus_settings.color);

        let distance = focus.length();
        let text = format!("{distance:.1}");

        let arrow_direction = focus.normalize_or_zero();
        let along_arrow_offset = focus_gizmo_state.sphere_radius.mul_add(
            FOCUS_GIZMO_LABEL_RADIUS_MULTIPLIER,
            FOCUS_GIZMO_DISTANCE_LABEL_OFFSET,
        );
        let label_world_position = focus + (arrow_direction * along_arrow_offset);

        if let Ok(label_screen_position) =
            camera.world_to_viewport(camera_transform, label_world_position)
        {
            if let Ok((mut label_text, mut node, mut text_color)) = label_query.single_mut() {
                label_text.0.clone_from(&text);
                text_color.0 = focus_settings.color;
                node.left = Val::Px(label_screen_position.x);
                node.top = Val::Px(label_screen_position.y);
            } else {
                commands.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: FontSize::Px(EDGE_MARKER_FONT_SIZE),
                        ..default()
                    },
                    TextColor(focus_settings.color),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(label_screen_position.x),
                        top: Val::Px(label_screen_position.y),
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
