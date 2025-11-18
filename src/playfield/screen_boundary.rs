use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::Edge;
use crate::camera::RenderLayer;
use crate::camera::ScreenSpaceBoundary;
use crate::camera::ZoomConfig;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::playfield::Boundary;

pub struct ScreenBoundaryPlugin;

impl Plugin for ScreenBoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<ScreenBoundaryGizmo>()
            .init_resource::<ScreenBoundaryConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<ScreenBoundaryConfig>::default().run_if(toggle_active(
                    false,
                    GameAction::ScreenBoundaryConfigInspector,
                )),
            )
            .add_systems(
                Update,
                apply_screen_boundary_config.run_if(resource_changed::<ScreenBoundaryConfig>),
            )
            .add_systems(
                Update,
                draw_screen_aligned_boundary_box
                    .run_if(toggle_active(false, GameAction::BoundaryBox)),
            )
            .add_systems(
                Update,
                cleanup_margin_labels.run_if(toggle_active(true, GameAction::BoundaryBox)),
            );
    }
}

#[derive(Component, Reflect)]
struct MarginLabel {
    edge: Edge,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct ScreenBoundaryGizmo {}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct ScreenBoundaryConfig {
    rectangle_color:  Color,
    balanced_color:   Color,
    unbalanced_color: Color,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    line_width:       f32,
}

impl Default for ScreenBoundaryConfig {
    fn default() -> Self {
        Self {
            rectangle_color:  Color::from(tailwind::YELLOW_400),
            balanced_color:   Color::srgb(0.0, 1.0, 0.0),
            unbalanced_color: Color::srgb(1.0, 0.0, 0.0),
            line_width:       1.0,
        }
    }
}

fn apply_screen_boundary_config(
    mut config_store: ResMut<GizmoConfigStore>,
    config: Res<ScreenBoundaryConfig>,
) {
    let (gizmo_config, _) = config_store.config_mut::<ScreenBoundaryGizmo>();
    gizmo_config.line.width = config.line_width;
    gizmo_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

/// Calculates the color for an edge based on balance state
const fn calculate_edge_color(
    edge: Edge,
    h_balanced: bool,
    v_balanced: bool,
    config: &ScreenBoundaryConfig,
) -> Color {
    match edge {
        Edge::Left | Edge::Right => {
            if h_balanced {
                config.balanced_color
            } else {
                config.unbalanced_color
            }
        },
        Edge::Top | Edge::Bottom => {
            if v_balanced {
                config.balanced_color
            } else {
                config.unbalanced_color
            }
        },
    }
}

/// Calculates the normalized screen-space position for a label based on edge type
fn calculate_label_position(edge: Edge, margins: &ScreenSpaceBoundary) -> (f32, f32) {
    const TEXT_OFFSET: f32 = 0.01;
    match edge {
        Edge::Left => {
            let (_, screen_y) = margins.screen_edge_center(edge);
            (
                -margins.half_tan_hfov + TEXT_OFFSET,
                TEXT_OFFSET.mul_add(2.0, screen_y),
            )
        },
        Edge::Right => {
            let (_, screen_y) = margins.screen_edge_center(edge);
            (
                margins.half_tan_hfov - TEXT_OFFSET,
                TEXT_OFFSET.mul_add(2.0, screen_y),
            )
        },
        Edge::Top => {
            let (screen_x, _) = margins.screen_edge_center(edge);
            (screen_x + TEXT_OFFSET, margins.half_tan_vfov - TEXT_OFFSET)
        },
        Edge::Bottom => {
            let (screen_x, _) = margins.screen_edge_center(edge);
            (screen_x + TEXT_OFFSET, -margins.half_tan_vfov + TEXT_OFFSET)
        },
    }
}

/// Creates the 4 corners of the screen-aligned boundary rectangle in world space
fn create_screen_corners(
    margins: &ScreenSpaceBoundary,
    cam_pos: Vec3,
    cam_right: Vec3,
    cam_up: Vec3,
    cam_forward: Vec3,
) -> [Vec3; 4] {
    [
        margins.normalized_to_world(
            margins.min_norm_x,
            margins.min_norm_y,
            cam_pos,
            cam_right,
            cam_up,
            cam_forward,
        ),
        margins.normalized_to_world(
            margins.max_norm_x,
            margins.min_norm_y,
            cam_pos,
            cam_right,
            cam_up,
            cam_forward,
        ),
        margins.normalized_to_world(
            margins.max_norm_x,
            margins.max_norm_y,
            cam_pos,
            cam_right,
            cam_up,
            cam_forward,
        ),
        margins.normalized_to_world(
            margins.min_norm_x,
            margins.max_norm_y,
            cam_pos,
            cam_right,
            cam_up,
            cam_forward,
        ),
    ]
}

/// Draws the boundary rectangle outline
fn draw_rectangle(
    gizmos: &mut Gizmos<ScreenBoundaryGizmo>,
    corners: &[Vec3; 4],
    config: &ScreenBoundaryConfig,
) {
    for i in 0..4 {
        let next = (i + 1) % 4;
        gizmos.line(corners[i], corners[next], config.rectangle_color);
    }
}

/// Updates an existing margin label or creates a new one
fn update_or_create_margin_label(
    commands: &mut Commands,
    label_query: &mut Query<
        (Entity, &MarginLabel, &mut Text, &mut Node, &mut TextColor),
        Without<Camera>,
    >,
    edge: Edge,
    text: String,
    color: Color,
    screen_pos: Vec2,
    viewport_size: Vec2,
) {
    // Find or update existing label for this edge
    let mut found = false;
    for (_, label, mut label_text, mut node, mut text_color) in label_query {
        if label.edge == edge {
            label_text.0.clone_from(&text);
            text_color.0 = color;
            match edge {
                Edge::Left | Edge::Top => {
                    node.left = Val::Px(screen_pos.x);
                    node.top = Val::Px(screen_pos.y);
                    node.right = Val::Auto;
                    node.bottom = Val::Auto;
                },
                Edge::Right => {
                    node.right = Val::Px(viewport_size.x - screen_pos.x);
                    node.top = Val::Px(screen_pos.y);
                    node.left = Val::Auto;
                    node.bottom = Val::Auto;
                },
                Edge::Bottom => {
                    node.left = Val::Px(screen_pos.x);
                    node.bottom = Val::Px(viewport_size.y - screen_pos.y);
                    node.right = Val::Auto;
                    node.top = Val::Auto;
                },
            }
            found = true;
            break;
        }
    }

    if !found {
        // Create new label
        let node = match edge {
            Edge::Left | Edge::Top => Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.x),
                top: Val::Px(screen_pos.y),
                ..default()
            },
            Edge::Right => Node {
                position_type: PositionType::Absolute,
                right: Val::Px(viewport_size.x - screen_pos.x),
                top: Val::Px(screen_pos.y),
                ..default()
            },
            Edge::Bottom => Node {
                position_type: PositionType::Absolute,
                left: Val::Px(screen_pos.x),
                bottom: Val::Px(viewport_size.y - screen_pos.y),
                ..default()
            },
        };

        commands.spawn((
            Text::new(text),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(color),
            node,
            RenderLayers::from_layers(RenderLayer::Game.layers()),
            MarginLabel { edge },
        ));
    }
}

/// used to draw a screen-aligned box around the boundary
/// used for troubleshooting camera movement logic
fn draw_screen_aligned_boundary_box(
    mut commands: Commands,
    mut gizmos: Gizmos<ScreenBoundaryGizmo>,
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    config: Res<ScreenBoundaryConfig>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<PanOrbitCamera>>,
    mut label_query: Query<
        (Entity, &MarginLabel, &mut Text, &mut Node, &mut TextColor),
        Without<Camera>,
    >,
) {
    let Ok((cam, cam_global, projection)) = camera.single() else {
        return;
    };

    let Projection::Perspective(perspective) = projection else {
        return;
    };

    // Get actual viewport aspect ratio
    let aspect_ratio = if let Some(viewport_size) = cam.logical_viewport_size() {
        viewport_size.x / viewport_size.y
    } else {
        perspective.aspect_ratio
    };

    // Calculate screen-space bounds using ScreenSpaceMargins
    let Some(margins) = ScreenSpaceBoundary::from_camera_view(
        &boundary,
        cam_global,
        perspective,
        aspect_ratio,
        zoom_config.zoom_margin_multiplier(),
    ) else {
        return; // Boundary behind camera
    };

    // Get camera basis vectors for reconstruction
    let cam_pos = cam_global.translation();
    let cam_rot = cam_global.rotation();
    let cam_forward = cam_rot * Vec3::NEG_Z;
    let cam_right = cam_rot * Vec3::X;
    let cam_up = cam_rot * Vec3::Y;

    let rect_corners_world =
        create_screen_corners(&margins, cam_pos, cam_right, cam_up, cam_forward);
    draw_rectangle(&mut gizmos, &rect_corners_world, &config);

    // Draw lines from visible boundary edges to screen edges
    // Green if margins are balanced, red otherwise
    let h_balanced = margins.is_horizontally_balanced(zoom_config.margin_tolerance);
    let v_balanced = margins.is_vertically_balanced(zoom_config.margin_tolerance);

    // Track which edges are currently visible for label cleanup
    let mut visible_edges: Vec<Edge> = Vec::new();

    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        if let Some((boundary_x, boundary_y)) = margins.boundary_edge_center(edge) {
            visible_edges.push(edge);
            let (screen_x, screen_y) = margins.screen_edge_center(edge);

            let boundary_pos = margins.normalized_to_world(
                boundary_x,
                boundary_y,
                cam_pos,
                cam_right,
                cam_up,
                cam_forward,
            );
            let screen_pos = margins.normalized_to_world(
                screen_x,
                screen_y,
                cam_pos,
                cam_right,
                cam_up,
                cam_forward,
            );

            let color = calculate_edge_color(edge, h_balanced, v_balanced, &config);
            gizmos.line(boundary_pos, screen_pos, color);

            // Add text label for this edge
            let percentage = margins.margin_percentage(edge);
            let text = format!("{percentage:.3}%");

            let (label_x, label_y) = calculate_label_position(edge, &margins);

            let mut world_pos = margins.normalized_to_world(
                label_x,
                label_y,
                cam_pos,
                cam_right,
                cam_up,
                cam_forward,
            );
            world_pos -= cam_forward * 1.0;

            // Project to screen space
            if let Ok(label_screen_pos) = cam.world_to_viewport(cam_global, world_pos) {
                // Extract viewport size - must exist if world_to_viewport succeeded
                let Some(viewport_size) = cam.logical_viewport_size() else {
                    continue;
                };

                update_or_create_margin_label(
                    &mut commands,
                    &mut label_query,
                    edge,
                    text,
                    color,
                    label_screen_pos,
                    viewport_size,
                );
            }
        }
    }

    // Remove labels for edges that are no longer visible
    for (entity, label, _, _, _) in &label_query {
        if !visible_edges.contains(&label.edge) {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_margin_labels(mut commands: Commands, label_query: Query<Entity, With<MarginLabel>>) {
    for entity in &label_query {
        commands.entity(entity).despawn();
    }
}
