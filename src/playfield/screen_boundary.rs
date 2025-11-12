use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
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
            .add_systems(Startup, init_screen_boundary_gizmo_config)
            .add_systems(
                Update,
                draw_screen_aligned_boundary_box
                    .run_if(toggle_active(false, GameAction::BoundaryBox)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct ScreenBoundaryGizmo {}

fn init_screen_boundary_gizmo_config(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<ScreenBoundaryGizmo>();
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

/// used to draw a yellow screen-aligned box around the boundary
/// used for troubleshooting camera movement logic
fn draw_screen_aligned_boundary_box(
    mut gizmos: Gizmos<ScreenBoundaryGizmo>,
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    camera: Query<(&Camera, &Transform, &GlobalTransform, &Projection), With<PanOrbitCamera>>,
) {
    let Ok((cam, cam_transform, cam_global, projection)) = camera.single() else {
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
    let cam_pos = cam_transform.translation;
    let cam_rot = cam_global.rotation();
    let cam_forward = cam_rot * Vec3::NEG_Z;
    let cam_right = cam_rot * Vec3::X;
    let cam_up = cam_rot * Vec3::Y;

    // Create the 4 corners using the helper method
    let rect_corners_world = [
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
    ];

    // Draw the yellow boundary rectangle
    for i in 0..4 {
        let next = (i + 1) % 4;
        gizmos.line(
            rect_corners_world[i],
            rect_corners_world[next],
            Color::from(tailwind::YELLOW_400),
        );
    }

    // Draw lines from visible boundary edges to screen edges
    // Green if margins are balanced, red otherwise
    let h_balanced = margins.is_horizontally_balanced(zoom_config.balance_tolerance);
    let v_balanced = margins.is_vertically_balanced(zoom_config.balance_tolerance);

    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        if let Some((boundary_x, boundary_y)) = margins.boundary_edge_center(edge) {
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

            // Color based on balance: green if balanced, red otherwise
            let color = match edge {
                Edge::Left | Edge::Right => {
                    if h_balanced {
                        Color::srgb(0.0, 1.0, 0.0) // Green
                    } else {
                        Color::srgb(1.0, 0.0, 0.0) // Red
                    }
                },
                Edge::Top | Edge::Bottom => {
                    if v_balanced {
                        Color::srgb(0.0, 1.0, 0.0) // Green
                    } else {
                        Color::srgb(1.0, 0.0, 0.0) // Red
                    }
                },
            };

            gizmos.line(boundary_pos, screen_pos, color);
        }
    }
}
