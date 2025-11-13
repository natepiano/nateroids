use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::CameraConfig;
use crate::camera::ScreenSpaceBoundary;
use crate::camera::ZoomConfig;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::playfield::Boundary;

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

/// Debug helper that verifies boundary corners using `Camera.world_to_viewport`.
///
/// Prints viewport bounds and whether any corners fall outside the viewport.
/// Used for debugging viewport calculations during zoom-to-fit.
fn verify_viewport_corners_debug(
    camera: &Camera,
    cam_global: &GlobalTransform,
    boundary: &Boundary,
) {
    if let Some(viewport_size) = camera.logical_viewport_size() {
        let mut any_outside = false;
        let mut min_vp_x = f32::INFINITY;
        let mut max_vp_x = f32::NEG_INFINITY;
        let mut min_vp_y = f32::INFINITY;
        let mut max_vp_y = f32::NEG_INFINITY;

        let boundary_corners = boundary.corners();

        for corner in &boundary_corners {
            if let Ok(viewport_pos) = camera.world_to_viewport(cam_global, *corner) {
                min_vp_x = min_vp_x.min(viewport_pos.x);
                max_vp_x = max_vp_x.max(viewport_pos.x);
                min_vp_y = min_vp_y.min(viewport_pos.y);
                max_vp_y = max_vp_y.max(viewport_pos.y);

                if viewport_pos.x < 0.0
                    || viewport_pos.x > viewport_size.x
                    || viewport_pos.y < 0.0
                    || viewport_pos.y > viewport_size.y
                {
                    any_outside = true;
                }
            }
        }
        println!(
            "  Camera.world_to_viewport check: viewport={}x{}, bounds=[{:.1},{:.1}]x[{:.1},{:.1}], outside={}",
            viewport_size.x, viewport_size.y, min_vp_x, max_vp_x, min_vp_y, max_vp_y, any_outside
        );
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
        let boundary_center = boundary_corners.iter().sum::<Vec3>() / boundary_corners.len() as f32;
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
        "Iteration {}: center=({:.6}, {:.6}), span=({:.3}, {:.3}), bounds x:[{:.3}, {:.3}] y:[{:.3}, {:.3}]",
        zoom_state.iteration_count,
        center_x,
        center_y,
        span_x,
        span_y,
        margins.min_norm_x,
        margins.max_norm_x,
        margins.min_norm_y,
        margins.max_norm_y
    );
    println!(
        "  Screen edges: h_fov=±{:.3}, v_fov=±{:.3}, aspect={:.3}, zoom_mult={:.3}",
        half_tan_hfov,
        half_tan_vfov,
        aspect_ratio,
        zoom_config.zoom_margin_multiplier()
    );

    let h_min = margins.left_margin.min(margins.right_margin);
    let v_min = margins.top_margin.min(margins.bottom_margin);
    let (constraining_dim, current_margin, target_margin) = if h_min < v_min {
        ("H", h_min, margins.target_margin_x)
    } else {
        ("V", v_min, margins.target_margin_y)
    };

    println!(
        "  Margins: left={:.3}, right={:.3}, top={:.3}, bottom={:.3}, target_x={:.3}, target_y={:.3}, min={:.3}",
        margins.left_margin,
        margins.right_margin,
        margins.top_margin,
        margins.bottom_margin,
        margins.target_margin_x,
        margins.target_margin_y,
        margins.min_margin()
    );
    println!(
        "  Constraining: dim={}, current={:.3}, target={:.3}, ratio={:.2}",
        constraining_dim,
        current_margin,
        target_margin,
        current_margin / target_margin
    );

    verify_viewport_corners_debug(camera, cam_global, &boundary);

    println!(
        "  Balanced: h_diff={:.3}, v_diff={:.3}, is_balanced={}, is_fitted={}",
        (margins.left_margin - margins.right_margin).abs(),
        (margins.top_margin - margins.bottom_margin).abs(),
        margins.is_balanced(zoom_config.margin_tolerance),
        margins.is_fitted(zoom_config.margin_tolerance)
    );

    // Use target_radius instead of actual radius to avoid one-frame delay
    // Since we set smoothness to 0, target should equal actual, but Transform updates next frame
    let current_radius = pan_orbit.target_radius;

    println!(
        "  START OF ITERATION: pan_orbit.target_radius={:.6}, pan_orbit.target_focus=({:.3},{:.3},{:.3})",
        pan_orbit.target_radius,
        pan_orbit.target_focus.x,
        pan_orbit.target_focus.y,
        pan_orbit.target_focus.z
    );

    let target_focus =
        calculate_target_focus(pan_orbit.target_focus, current_radius, &margins, cam_global);

    // For debug output
    let focus_to_boundary_distance = pan_orbit.target_focus.length();
    let far_from_boundary_threshold = current_radius * 0.5;

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

    println!(
        "  SPAN-BASED CALC: current_span=({:.3},{:.3}), target_span=({:.3},{:.3}), ratio_x={:.3}, ratio_y={:.3}, ratio={:.3}",
        span_x, span_y, target_span_x, target_span_y, ratio_x, ratio_y, ratio
    );
    println!(
        "  RADIUS CALC: current={:.3}, target={:.3}, delta={:.3}",
        current_radius,
        target_radius,
        target_radius - current_radius
    );

    // Calculate error magnitudes
    let focus_delta = target_focus - pan_orbit.target_focus;
    let radius_delta = target_radius - current_radius;
    let focus_error = focus_delta.length();

    let focus_phase = if focus_to_boundary_distance > far_from_boundary_threshold {
        "PHASE1"
    } else {
        "PHASE2"
    };

    println!(
        "  FOCUS {}: dist_to_boundary={:.1}, threshold={:.1}, target=({:.3},{:.3},{:.3}), delta=({:.3},{:.3},{:.3}), error={:.3}",
        focus_phase,
        focus_to_boundary_distance,
        far_from_boundary_threshold,
        target_focus.x,
        target_focus.y,
        target_focus.z,
        focus_delta.x,
        focus_delta.y,
        focus_delta.z,
        focus_error
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

    println!(
        "  APPLYING: radius_delta={:.6}, radius_adjustment={:.6}, current={:.6}, new_target={:.6}",
        radius_delta, radius_adjustment, current_radius, new_target_radius
    );

    let balanced = margins.is_balanced(zoom_config.margin_tolerance);
    let fitted = margins.is_fitted(zoom_config.margin_tolerance);

    println!(
        "  Correcting: rate={:.1}%, focus_adj=({:.3},{:.3},{:.3}), radius {:.1}->{:.1}, balanced={}, fitted={}",
        rate * 100.0,
        focus_adjustment.x,
        focus_adjustment.y,
        focus_adjustment.z,
        current_radius,
        pan_orbit.target_radius,
        balanced,
        fitted
    );

    // Check completion: balanced AND fitted
    if balanced && fitted {
        println!(
            "  Zoom-to-fit complete! balanced={}, fitted={}",
            balanced, fitted
        );
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    zoom_state.iteration_count += 1;

    // Stop if we hit max iterations
    if zoom_state.iteration_count >= zoom_config.max_iterations {
        println!(
            "Zoom-to-fit stopped at max iterations! balanced={}, fitted={}",
            balanced, fitted
        );
        commands.entity(entity).remove::<ZoomToFitActive>();
    }
}
