use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::ZoomConfig;

use super::constants::SCREEN_BOUNDARY_FONT_SIZE;
use super::constants::SCREEN_BOUNDARY_LINE_WIDTH;
use super::constants::SCREEN_BOUNDARY_TEXT_OFFSET;
use super::constants::SCREEN_BOUNDARY_WORLD_POS_OFFSET;
use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::playfield::Boundary;
use crate::traits::UsizeExt;

/// Boundary box edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

/// Screen-space margin information for a boundary
struct ScreenSpaceBoundary {
    /// Distance from left edge (positive = inside, negative = outside)
    left_margin:   f32,
    /// Distance from right edge (positive = inside, negative = outside)
    right_margin:  f32,
    /// Distance from top edge (positive = inside, negative = outside)
    top_margin:    f32,
    /// Distance from bottom edge (positive = inside, negative = outside)
    bottom_margin: f32,
    /// Minimum normalized x coordinate in screen space
    min_norm_x:    f32,
    /// Maximum normalized x coordinate in screen space
    max_norm_x:    f32,
    /// Minimum normalized y coordinate in screen space
    min_norm_y:    f32,
    /// Maximum normalized y coordinate in screen space
    max_norm_y:    f32,
    /// Average depth of boundary corners from camera
    avg_depth:     f32,
    /// Half tangent of vertical field of view
    half_tan_vfov: f32,
    /// Half tangent of horizontal field of view (vfov * `aspect_ratio`)
    half_tan_hfov: f32,
}

impl ScreenSpaceBoundary {
    /// Creates screen space margins from a camera's view of a boundary.
    /// Returns `None` if any boundary corner is behind the camera.
    #[allow(clippy::similar_names)] // half_tan_hfov vs half_tan_vfov are standard FOV terms
    fn from_camera_view(
        boundary: &Boundary,
        cam_global: &GlobalTransform,
        perspective: &PerspectiveProjection,
        viewport_aspect: f32,
        _zoom_multiplier: f32,
    ) -> Option<Self> {
        let half_tan_vfov = (perspective.fov * 0.5).tan();
        let half_tan_hfov = half_tan_vfov * viewport_aspect;

        // Get boundary corners
        let boundary_corners = boundary.corners();

        // Get camera basis vectors from global transform (world position, not local)
        let cam_pos = cam_global.translation();
        let cam_rot = cam_global.rotation();
        let cam_forward = cam_rot * Vec3::NEG_Z;
        let cam_right = cam_rot * Vec3::X;
        let cam_up = cam_rot * Vec3::Y;

        // Project corners to screen space
        let mut min_norm_x = f32::INFINITY;
        let mut max_norm_x = f32::NEG_INFINITY;
        let mut min_norm_y = f32::INFINITY;
        let mut max_norm_y = f32::NEG_INFINITY;
        let mut avg_depth = 0.0;

        for corner in &boundary_corners {
            let relative = *corner - cam_pos;
            let depth = relative.dot(cam_forward);

            // Check if corner is behind camera
            if depth <= 0.1 {
                return None;
            }

            let x = relative.dot(cam_right);
            let y = relative.dot(cam_up);

            let norm_x = x / depth;
            let norm_y = y / depth;

            min_norm_x = min_norm_x.min(norm_x);
            max_norm_x = max_norm_x.max(norm_x);
            min_norm_y = min_norm_y.min(norm_y);
            max_norm_y = max_norm_y.max(norm_y);
            avg_depth += depth;
        }
        avg_depth /= boundary_corners.len().to_f32();

        // Calculate margins as distance from bounds to screen edges
        // Positive = within screen, negative = outside
        let left_margin = min_norm_x - (-half_tan_hfov);
        let right_margin = half_tan_hfov - max_norm_x;
        let bottom_margin = min_norm_y - (-half_tan_vfov);
        let top_margin = half_tan_vfov - max_norm_y;

        Some(Self {
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            min_norm_x,
            max_norm_x,
            min_norm_y,
            max_norm_y,
            avg_depth,
            half_tan_vfov,
            half_tan_hfov,
        })
    }

    /// Returns true if horizontal margins are balanced (left == right)
    fn is_horizontally_balanced(&self, tolerance: f32) -> bool {
        (self.left_margin - self.right_margin).abs() < tolerance
    }

    /// Returns true if vertical margins are balanced (top == bottom)
    fn is_vertically_balanced(&self, tolerance: f32) -> bool {
        (self.top_margin - self.bottom_margin).abs() < tolerance
    }

    /// Returns the screen edges in normalized space (left, right, top, bottom)
    fn screen_edges_normalized(&self) -> (f32, f32, f32, f32) {
        (
            -self.half_tan_hfov,
            self.half_tan_hfov,
            self.half_tan_vfov,
            -self.half_tan_vfov,
        )
    }

    /// Returns the center of a boundary edge in normalized space, clipped to visible portion
    /// Returns None if the edge is not visible (entirely off-screen)
    fn boundary_edge_center(&self, edge: Edge) -> Option<(f32, f32)> {
        let (left_edge, right_edge, top_edge, bottom_edge) = self.screen_edges_normalized();

        match edge {
            Edge::Left if self.min_norm_x > left_edge => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                Some((self.min_norm_x, y))
            },
            Edge::Right if self.max_norm_x < right_edge => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                Some((self.max_norm_x, y))
            },
            Edge::Top if self.max_norm_y < top_edge => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                Some((x, self.max_norm_y))
            },
            Edge::Bottom if self.min_norm_y > bottom_edge => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                Some((x, self.min_norm_y))
            },
            _ => None,
        }
    }

    /// Returns the center of a screen edge in normalized space, clipped to visible boundary portion
    fn screen_edge_center(&self, edge: Edge) -> (f32, f32) {
        let (left_edge, right_edge, top_edge, bottom_edge) = self.screen_edges_normalized();

        match edge {
            Edge::Left => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                (left_edge, y)
            },
            Edge::Right => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                (right_edge, y)
            },
            Edge::Top => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                (x, top_edge)
            },
            Edge::Bottom => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                (x, bottom_edge)
            },
        }
    }

    /// Converts normalized screen-space coordinates to world space
    fn normalized_to_world(
        &self,
        norm_x: f32,
        norm_y: f32,
        cam_pos: Vec3,
        cam_right: Vec3,
        cam_up: Vec3,
        cam_forward: Vec3,
    ) -> Vec3 {
        let world_x = norm_x * self.avg_depth;
        let world_y = norm_y * self.avg_depth;
        cam_pos + cam_right * world_x + cam_up * world_y + cam_forward * self.avg_depth
    }

    /// Returns the margin percentage for a given edge.
    /// Percentage represents how much of the screen width/height is margin.
    fn margin_percentage(&self, edge: Edge) -> f32 {
        let screen_width = 2.0 * self.half_tan_hfov;
        let screen_height = 2.0 * self.half_tan_vfov;

        match edge {
            Edge::Left => (self.left_margin / screen_width) * 100.0,
            Edge::Right => (self.right_margin / screen_width) * 100.0,
            Edge::Top => (self.top_margin / screen_height) * 100.0,
            Edge::Bottom => (self.bottom_margin / screen_height) * 100.0,
        }
    }
}

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
            line_width:       SCREEN_BOUNDARY_LINE_WIDTH,
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
    match edge {
        Edge::Left => {
            let (_, screen_y) = margins.screen_edge_center(edge);
            (
                -margins.half_tan_hfov + SCREEN_BOUNDARY_TEXT_OFFSET,
                SCREEN_BOUNDARY_TEXT_OFFSET.mul_add(2.0, screen_y),
            )
        },
        Edge::Right => {
            let (_, screen_y) = margins.screen_edge_center(edge);
            (
                margins.half_tan_hfov - SCREEN_BOUNDARY_TEXT_OFFSET,
                SCREEN_BOUNDARY_TEXT_OFFSET.mul_add(2.0, screen_y),
            )
        },
        Edge::Top => {
            let (screen_x, _) = margins.screen_edge_center(edge);
            (
                screen_x + SCREEN_BOUNDARY_TEXT_OFFSET,
                margins.half_tan_vfov - SCREEN_BOUNDARY_TEXT_OFFSET,
            )
        },
        Edge::Bottom => {
            let (screen_x, _) = margins.screen_edge_center(edge);
            (
                screen_x + SCREEN_BOUNDARY_TEXT_OFFSET,
                -margins.half_tan_vfov + SCREEN_BOUNDARY_TEXT_OFFSET,
            )
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
                font_size: SCREEN_BOUNDARY_FONT_SIZE,
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
            world_pos -= cam_forward * SCREEN_BOUNDARY_WORLD_POS_OFFSET;

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
