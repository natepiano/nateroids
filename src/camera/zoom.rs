use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::CameraConfig;
use crate::camera::ScreenSpaceBoundary;
use crate::camera::ZoomConfig;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::playfield::Boundary;
use crate::traits::UsizeExt;

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            start_zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)),
        )
        .add_systems(Update, update_zoom_to_fit)
        .add_observer(on_remove_zoom_to_fit);
    }
}

#[derive(Component)]
struct ZoomToFitActive {
    iteration_count: usize,
}

/// Observer that runs whenever `ZoomToFitActive` is removed from an entity.
/// Restores camera smoothness values from config.
fn on_remove_zoom_to_fit(
    remove: On<Remove, ZoomToFitActive>,
    camera_config: Res<CameraConfig>,
    mut camera: Query<&mut PanOrbitCamera>,
) {
    let Ok(mut pan_orbit) = camera.get_mut(remove.entity) else {
        return;
    };

    pan_orbit.zoom_smoothness = camera_config.zoom_smoothness;
    pan_orbit.pan_smoothness = camera_config.pan_smoothness;

    println!(
        "ZoomToFitActive removed: restored smoothness (zoom={:.2}, pan={:.2})",
        camera_config.zoom_smoothness, camera_config.pan_smoothness
    );
}

// Start the zoom-to-fit animation
fn start_zoom_to_fit(
    mut commands: Commands,
    mut camera_query: Query<
        (Entity, &mut PanOrbitCamera, Option<&ZoomToFitActive>),
        With<PanOrbitCamera>,
    >,
) {
    if let Ok((camera_entity, mut pan_orbit, existing_zoom)) = camera_query.single_mut() {
        // Allow restart if already running
        if existing_zoom.is_some() {
            println!("Zoom-to-fit already active, restarting");
        }

        // Disable smoothing so targets apply immediately
        pan_orbit.zoom_smoothness = 0.0;
        pan_orbit.pan_smoothness = 0.0;

        commands
            .entity(camera_entity)
            .insert(ZoomToFitActive { iteration_count: 0 });
        println!("Starting zoom-to-fit animation");
    }
}

/// Calculates the target focus point using a two-phase approach.
///
/// **Phase 1** (far from boundary): When focus is more than half the camera radius away from
/// the boundary center, move directly toward `Vec3::ZERO`.
///
/// **Phase 2** (close to boundary): Use screen-space centering to fine-tune the focus position
/// by converting screen-space offsets to world-space corrections.
fn calculate_target_focus(
    current_focus: Vec3,
    current_radius: f32,
    margins: &ScreenSpaceBoundary,
    cam_global: &GlobalTransform,
) -> Vec3 {
    let focus_to_boundary_distance = current_focus.length();
    let far_from_boundary_threshold = current_radius * 0.5;

    if focus_to_boundary_distance > far_from_boundary_threshold {
        // Phase 1: Move toward boundary center
        Vec3::ZERO
    } else {
        // Phase 2: Fine-tune using screen-space centering
        let (center_x, center_y) = margins.center();
        let cam_rot = cam_global.rotation();
        let cam_right = cam_rot * Vec3::X;
        let cam_up = cam_rot * Vec3::Y;

        // Convert screen-space offset to world-space adjustment
        let world_offset_x = center_x * margins.avg_depth;
        let world_offset_y = center_y * margins.avg_depth;
        let focus_correction = cam_right * world_offset_x + cam_up * world_offset_y;

        current_focus + focus_correction
    }
}

/// Convergence algorithm for zoom-to-fit animation using iterative adjustments.
///
/// **Convergence Rate**: Applies `convergence_rate` to both focus and radius adjustments each
/// frame, moving the camera gradually toward the target configuration.
///
/// **Convergence Detection**: Stops when both `is_fitted(margin_tolerance)` and
/// `is_balanced(margin_tolerance)` are true.
fn update_zoom_to_fit(
    mut commands: Commands,
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    mut camera_query: Query<(
        Entity,
        &GlobalTransform,
        &mut PanOrbitCamera,
        &Projection,
        &Camera,
        &mut ZoomToFitActive,
    )>,
) {
    let Ok((entity, cam_global, mut pan_orbit, projection, camera, mut zoom_state)) =
        camera_query.single_mut()
    else {
        return;
    };

    let Projection::Perspective(perspective) = projection else {
        return;
    };

    // Get actual viewport aspect ratio
    let aspect_ratio = if let Some(viewport_size) = camera.logical_viewport_size() {
        viewport_size.x / viewport_size.y
    } else {
        perspective.aspect_ratio
    };

    // Calculate screen-space bounds and margins
    let Some(margins) = ScreenSpaceBoundary::from_camera_view(
        &boundary,
        cam_global,
        perspective,
        aspect_ratio,
        zoom_config.zoom_margin_multiplier(),
    ) else {
        // Boundary behind camera, move camera back
        println!(
            "Iteration {}: Boundary behind camera, moving back",
            zoom_state.iteration_count
        );
        let boundary_corners = boundary.corners();
        let boundary_center =
            boundary_corners.iter().sum::<Vec3>() / boundary_corners.len().to_f32();
        pan_orbit.target_focus = boundary_center;
        pan_orbit.target_radius *= 1.5;
        pan_orbit.force_update = true;
        zoom_state.iteration_count += 1;
        return;
    };

    // Use FOV tangent values from margins (already calculated in from_camera_view)
    let half_tan_vfov = margins.half_tan_vfov;
    let half_tan_hfov = margins.half_tan_hfov;

    // Calculate center and span for debug printing
    let (center_x, center_y) = margins.center();
    let (span_x, span_y) = margins.span();

    println!(
        "Iteration {}: center=({:.3},{:.3}), span=({:.3},{:.3})",
        zoom_state.iteration_count, center_x, center_y, span_x, span_y
    );

    let h_min = margins.left_margin.min(margins.right_margin);
    let v_min = margins.top_margin.min(margins.bottom_margin);
    let (constraining_dim, current_margin, target_margin) = if h_min < v_min {
        ("H", h_min, margins.target_margin_x)
    } else {
        ("V", v_min, margins.target_margin_y)
    };

    println!(
        "  Margins: L={:.3} R={:.3} T={:.3} B={:.3}, target=({:.3},{:.3})",
        margins.left_margin,
        margins.right_margin,
        margins.top_margin,
        margins.bottom_margin,
        margins.target_margin_x,
        margins.target_margin_y
    );
    println!(
        "  Constraining: {}, margin={:.3}/{:.3} (ratio={:.2})",
        constraining_dim,
        current_margin,
        target_margin,
        current_margin / target_margin
    );

    // Use target_radius instead of actual radius to avoid one-frame delay
    // Since we set smoothness to 0, target should equal actual, but Transform updates next frame
    let current_radius = pan_orbit.target_radius;

    let target_focus =
        calculate_target_focus(pan_orbit.target_focus, current_radius, &margins, cam_global);

    // Calculate target radius using span ratios
    // Physics: At distance R, object has span S. Closer = larger span.
    // Relationship: S * R = constant, so target_R = current_R * (current_S / target_S)

    // Target spans with proper margins
    let target_span_x = 2.0 * half_tan_hfov / zoom_config.zoom_margin_multiplier();
    let target_span_y = 2.0 * half_tan_vfov / zoom_config.zoom_margin_multiplier();

    // Calculate ratios for each dimension
    let ratio_x = span_x / target_span_x;
    let ratio_y = span_y / target_span_y;

    // Use the larger ratio (constraining dimension) to ensure both fit
    let ratio = ratio_x.max(ratio_y);

    // Calculate target radius from current radius and span ratio
    let target_radius = current_radius * ratio;

    // Calculate error magnitudes
    let focus_delta = target_focus - pan_orbit.target_focus;
    let radius_delta = target_radius - current_radius;

    println!(
        "  Focus: adj=({:.3},{:.3},{:.3})",
        focus_delta.x, focus_delta.y, focus_delta.z
    );

    // Apply convergence rate to both focus and radius
    let rate = zoom_config.convergence_rate;
    let focus_adjustment = focus_delta * rate;
    let radius_adjustment = radius_delta * rate;

    // Apply adjustments
    pan_orbit.target_focus += focus_adjustment;
    let new_target_radius = current_radius + radius_adjustment;
    pan_orbit.target_radius = new_target_radius;
    pan_orbit.force_update = true;

    let balanced = margins.is_balanced(zoom_config.margin_tolerance);
    let fitted = margins.is_fitted(zoom_config.margin_tolerance);

    println!(
        "  Radius: {:.1}→{:.1} (Δ={:.3}, rate={:.0}%)",
        current_radius,
        new_target_radius,
        radius_delta,
        rate * 100.0
    );
    println!("  Status: balanced={}, fitted={}", balanced, fitted);

    // Check completion: balanced AND fitted
    if balanced && fitted {
        println!("  → CONVERGED");
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    zoom_state.iteration_count += 1;

    // Stop if we hit max iterations
    if zoom_state.iteration_count >= zoom_config.max_iterations {
        println!("  → MAX ITERATIONS REACHED (not converged)");
        commands.entity(entity).remove::<ZoomToFitActive>();
    }
}
