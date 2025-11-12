use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::ScreenSpaceMargins;
use crate::camera::ZoomConfig;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::playfield::Boundary;

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, home_camera.run_if(just_pressed(GameAction::Home)))
            .add_systems(
                Update,
                start_zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)),
            )
            .add_systems(Update, update_zoom_to_fit);
    }
}

#[derive(Component)]
struct ZoomToFitActive {
    max_iterations:       usize,
    iteration_count:      usize,
    previous_bounds:      Option<(f32, f32, f32, f32)>, // (min_x, max_x, min_y, max_y)
    original_zoom_smooth: f32,
    original_pan_smooth:  f32,
    locked_alpha:         f32, // Lock pitch during zoom-to-fit
    locked_beta:          f32, // Lock yaw during zoom-to-fit
}

const BALANCE_TOLERANCE: f32 = 0.002; // 0.2% tolerance - margins must be nearly equal
const AT_TARGET_TOLERANCE: f32 = 0.05; // 5% tolerance for "at target" check

pub fn calculate_camera_radius(grid_size: Vec3, fov: f32, aspect_ratio: f32, buffer: f32) -> f32 {
    // Calculate horizontal FOV based on aspect ratio
    let horizontal_fov = 2.0 * ((fov / 2.0).tan() * aspect_ratio).atan();

    // Calculate distances required for X and Y dimensions to fit in viewport
    let x_distance = (grid_size.x / 2.0) / (horizontal_fov / 2.0).tan();
    let y_distance = (grid_size.y / 2.0) / (fov / 2.0).tan();

    // Take the max of X and Y distances
    let xy_distance = x_distance.max(y_distance);

    // For Z dimension (depth)
    let z_half_depth = grid_size.z / 2.0;

    // Apply buffer to XY distance (for screen-space margin), then add Z depth
    // This ensures buffer represents actual screen-space margin percentage
    xy_distance * buffer + z_half_depth
}

pub fn home_camera(
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    mut camera_query: Query<(&mut PanOrbitCamera, &Projection)>,
) {
    if let Ok((mut pan_orbit, Projection::Perspective(perspective))) = camera_query.single_mut() {
        let grid_size = boundary.scale();

        let target_radius = calculate_camera_radius(
            grid_size,
            perspective.fov,
            perspective.aspect_ratio,
            zoom_config.zoom_margin_multiplier(),
        );

        // Set the camera's orbit parameters
        pan_orbit.target_focus = Vec3::ZERO;
        pan_orbit.target_yaw = 0.0;
        pan_orbit.target_pitch = 0.0;
        pan_orbit.target_radius = target_radius;

        pan_orbit.force_update = true;
    }
}

// Start the zoom-to-fit animation
fn start_zoom_to_fit(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut PanOrbitCamera), With<PanOrbitCamera>>,
) {
    if let Ok((camera_entity, mut pan_orbit)) = camera_query.single_mut() {
        // Store original smoothness values
        let original_zoom_smooth = pan_orbit.zoom_smoothness;
        let original_pan_smooth = pan_orbit.pan_smoothness;

        // Lock current pitch and yaw - only adjust focus and radius
        let locked_alpha = pan_orbit.target_pitch;
        let locked_beta = pan_orbit.target_yaw;

        // Disable smoothing so targets apply immediately
        pan_orbit.zoom_smoothness = 0.0;
        pan_orbit.pan_smoothness = 0.0;

        commands.entity(camera_entity).insert(ZoomToFitActive {
            max_iterations: 200,
            iteration_count: 0,
            previous_bounds: None,
            original_zoom_smooth,
            original_pan_smooth,
            locked_alpha,
            locked_beta,
        });
        println!(
            "Starting zoom-to-fit animation (stored smoothness: zoom={:.2}, pan={:.2})",
            original_zoom_smooth, original_pan_smooth
        );
    }
}

/// Convergence algorithm for zoom-to-fit animation using predictive overshoot/flip detection.
///
/// The algorithm adjusts camera focus and radius at a constant 12% rate per frame, using
/// forward simulation to detect and handle two types of problems before they occur:
///
/// 1. **Overshoot Prevention**: Simulates the next frame's margin to detect if we'd cross the
///    target, then scales adjustment to land exactly on target.
///
/// 2. **Dimension Flip Detection**: The "constraining dimension" (H or V) is whichever has the
///    larger screen-space span. Even with locked yaw/pitch, parallax from camera position changes
///    can shift relative spans, potentially causing H↔V flips.
///
///    When a flip is detected in simulation:
///    - If both dimensions are within 5% of target: **Stop immediately** - the flip is a
///      convergence signal indicating both dimensions are equally close to target.
///    - Otherwise: Apply 30% damping to prevent oscillation from premature flip.
///
/// This maintains smooth constant-rate motion throughout, stopping cleanly when converged
/// rather than fighting natural convergence with artificial damping.
fn update_zoom_to_fit(
    mut commands: Commands,
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    mut camera_query: Query<(
        Entity,
        &Transform,
        &GlobalTransform,
        &mut PanOrbitCamera,
        &Projection,
        &Camera,
        &mut ZoomToFitActive,
    )>,
) {
    let Ok((entity, cam_transform, cam_global, mut pan_orbit, projection, camera, mut zoom_state)) =
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
    let Some(margins) = ScreenSpaceMargins::from_camera_view(
        &boundary,
        cam_transform,
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
        let grid_size = boundary.scale();
        let half_size = grid_size / 2.0;
        let boundary_corners = [
            Vec3::new(-half_size.x, -half_size.y, -half_size.z),
            Vec3::new(half_size.x, -half_size.y, -half_size.z),
            Vec3::new(-half_size.x, half_size.y, -half_size.z),
            Vec3::new(half_size.x, half_size.y, -half_size.z),
            Vec3::new(-half_size.x, -half_size.y, half_size.z),
            Vec3::new(half_size.x, -half_size.y, half_size.z),
            Vec3::new(-half_size.x, half_size.y, half_size.z),
            Vec3::new(half_size.x, half_size.y, half_size.z),
        ];
        let boundary_center = boundary_corners.iter().sum::<Vec3>() / boundary_corners.len() as f32;
        pan_orbit.target_focus = boundary_center;
        pan_orbit.target_radius *= 1.5;
        pan_orbit.force_update = true;
        zoom_state.iteration_count += 1;
        return;
    };

    // Calculate center and span for debug printing
    let center_x = (margins.min_norm_x + margins.max_norm_x) * 0.5;
    let center_y = (margins.min_norm_y + margins.max_norm_y) * 0.5;
    let span_x = margins.max_norm_x - margins.min_norm_x;
    let span_y = margins.max_norm_y - margins.min_norm_y;

    let half_tan_vfov = (perspective.fov * 0.5).tan();
    let half_tan_hfov = half_tan_vfov * aspect_ratio;

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

    // Use Camera's official world_to_viewport to verify if any corners are outside
    if let Some(viewport_size) = camera.logical_viewport_size() {
        let mut any_outside = false;
        let mut min_vp_x = f32::INFINITY;
        let mut max_vp_x = f32::NEG_INFINITY;
        let mut min_vp_y = f32::INFINITY;
        let mut max_vp_y = f32::NEG_INFINITY;

        let grid_size = boundary.scale();
        let half_size = grid_size / 2.0;
        let boundary_corners = [
            Vec3::new(-half_size.x, -half_size.y, -half_size.z),
            Vec3::new(half_size.x, -half_size.y, -half_size.z),
            Vec3::new(-half_size.x, half_size.y, -half_size.z),
            Vec3::new(half_size.x, half_size.y, -half_size.z),
            Vec3::new(-half_size.x, -half_size.y, half_size.z),
            Vec3::new(half_size.x, -half_size.y, half_size.z),
            Vec3::new(-half_size.x, half_size.y, half_size.z),
            Vec3::new(half_size.x, half_size.y, half_size.z),
        ];

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

    println!(
        "  Balanced: h_diff={:.3}, v_diff={:.3}, is_balanced={}, is_fitted={}",
        (margins.left_margin - margins.right_margin).abs(),
        (margins.top_margin - margins.bottom_margin).abs(),
        margins.is_balanced(BALANCE_TOLERANCE),
        margins.is_fitted(AT_TARGET_TOLERANCE)
    );

    // Check if bounds have actually changed since last frame (detect camera stabilization)
    // Track bounds for debugging
    let current_bounds = (
        margins.min_norm_x,
        margins.max_norm_x,
        margins.min_norm_y,
        margins.max_norm_y,
    );
    zoom_state.previous_bounds = Some(current_bounds);

    let current_radius = pan_orbit.radius.unwrap_or(pan_orbit.target_radius);
    let cam_rot = cam_global.rotation();
    let cam_right = cam_rot * Vec3::X;
    let cam_up = cam_rot * Vec3::Y;

    // Calculate correction to center the boundary
    let offset_x = center_x * current_radius * half_tan_hfov;
    let offset_y = center_y * current_radius * half_tan_vfov;
    let offset_world = cam_right * offset_x + cam_up * offset_y;
    let target_focus = pan_orbit.target_focus + offset_world;

    // Calculate radius to achieve target margins
    let h_min = margins.left_margin.min(margins.right_margin);
    let v_min = margins.top_margin.min(margins.bottom_margin);
    let (current_margin_val, target_margin_val) = if h_min < v_min {
        (h_min, margins.target_margin_x)
    } else {
        (v_min, margins.target_margin_y)
    };


    let target_radius =  {
        // Cap the ratio to prevent huge jumps
        let ratio = (target_margin_val / current_margin_val.max(0.001)).clamp(
            zoom_config.min_ratio_clamp,
            zoom_config.max_ratio_clamp,
        );
        current_radius * ratio
    };

    // Calculate error magnitudes
    let focus_delta = target_focus - pan_orbit.target_focus;
    let radius_delta = target_radius - current_radius;
    let focus_error = focus_delta.length();
    let radius_error = radius_delta.abs();

    // Calculate relative errors
    let focus_rel_error = focus_error / current_radius.max(1.0);
    let radius_rel_error = radius_error / current_radius.max(1.0);
    let max_rel_error = focus_rel_error.max(radius_rel_error);

    // Check if we're already fitted (constraining dimension at target)
    let already_fitted = margins.is_fitted(AT_TARGET_TOLERANCE);

    // Use a single unified rate for both focus and radius
    // This avoids creating oscillations from unbalanced adjustments
    // Use faster rate when already fitted - we're just balancing/centering
    let base_rate = if already_fitted {
        zoom_config.balancing_rate
    } else {
        zoom_config.fitting_rate
    };

    let rate_label = if already_fitted {
        "balancing_rate"
    } else {
        "fitting_rate"
    };

    let mut focus_adjustment = focus_delta * base_rate;
    let mut radius_adjustment = radius_delta * base_rate;
    let mut effective_rate = base_rate;

    // Simulate what the margin would be after applying this adjustment
    let proposed_focus = pan_orbit.target_focus + focus_adjustment;
    let proposed_radius = current_radius + radius_adjustment;

    // We need to simulate the camera transform with the proposed values
    // The camera looks from (focus + offset_from_angles) towards focus
    // For simplicity, create a transform by offsetting along the current camera direction
    let cam_forward = cam_global.forward();
    let proposed_cam_pos = proposed_focus - cam_forward.as_vec3() * proposed_radius;

    // Create hypothetical transform for prediction
    let mut hypothetical_transform = *cam_transform;
    hypothetical_transform.translation = proposed_cam_pos;

    // Recalculate margins with proposed transform
    if let Some(proposed_margins) = ScreenSpaceMargins::from_camera_view(
        &boundary,
        &hypothetical_transform,
        cam_global,
        perspective,
        aspect_ratio,
        zoom_config.zoom_margin_multiplier(),
    ) {
        let proposed_h_min = proposed_margins
            .left_margin
            .min(proposed_margins.right_margin);
        let proposed_v_min = proposed_margins
            .top_margin
            .min(proposed_margins.bottom_margin);
        let proposed_min_margin = proposed_h_min.min(proposed_v_min);

        // Check if constraining dimension would flip
        let current_constraining_is_h = margins.is_horizontal_constraining();
        let proposed_constraining_is_h = proposed_margins.is_horizontal_constraining();
        let would_flip_dimension = current_constraining_is_h != proposed_constraining_is_h;

        if would_flip_dimension {
            // Dimension flip detected - check if both dimensions are already close to target
            // If so, the flip means we're done rather than oscillating
            let h_close = (h_min - margins.target_margin_x).abs()
                < margins.target_margin_x * zoom_config.convergence_threshold;
            let v_close = (v_min - margins.target_margin_y).abs()
                < margins.target_margin_y * zoom_config.convergence_threshold;

            if h_close && v_close {
                // Both dimensions near target - flip is a convergence signal, stop here
                println!(
                    "  PREDICTION: Would flip dimension ({}→{}) but BOTH close to target! h_err={:.1}%, v_err={:.1}% - STOPPING",
                    if current_constraining_is_h { "H" } else { "V" },
                    if proposed_constraining_is_h { "H" } else { "V" },
                    ((h_min - margins.target_margin_x) / margins.target_margin_x * 100.0).abs(),
                    ((v_min - margins.target_margin_y) / margins.target_margin_y * 100.0).abs()
                );
                pan_orbit.zoom_smoothness = zoom_state.original_zoom_smooth;
                pan_orbit.pan_smoothness = zoom_state.original_pan_smooth;
                commands.entity(entity).remove::<ZoomToFitActive>();
                return;
            } else {
                // Real oscillation problem - apply damping
                let damping = zoom_config.flip_damping;
                focus_adjustment *= damping;
                radius_adjustment *= damping;
                effective_rate *= damping;
                println!(
                    "  PREDICTION: Would flip dimension ({}→{}) and NOT converged! Damping {rate_label} from {:.1}% to {:.1}% ({:.1}% damping factor)",
                    if current_constraining_is_h { "H" } else { "V" },
                    if proposed_constraining_is_h { "H" } else { "V" },
                    base_rate * 100.0,
                    effective_rate * 100.0,
                    damping * 100.0
                );
            }
        } else {
            // Same dimension - check for overshoot
            let would_overshoot = if current_margin_val < target_margin_val {
                // Currently too small, growing towards target
                proposed_min_margin > target_margin_val
            } else {
                // Currently too large, shrinking towards target
                proposed_min_margin < target_margin_val
            };

            if would_overshoot {
                // Calculate exact scale factor to reach target precisely
                let margin_delta = proposed_min_margin - current_margin_val;
                let target_delta = target_margin_val - current_margin_val;

                let scale_factor = if margin_delta.abs() > 0.001 {
                    (target_delta / margin_delta).clamp(0.0, 1.0)
                } else {
                    0.5 // Fallback if margin barely changed
                };

                effective_rate = base_rate * scale_factor;
                focus_adjustment *= scale_factor;
                radius_adjustment *= scale_factor;
                println!(
                    "  PREDICTION: Would overshoot margins! Scaling {rate_label} from {:.1}% to {:.1}% ({:.1}% scale factor)",
                    base_rate * 100.0,
                    effective_rate * 100.0,
                    scale_factor * 100.0
                );
            } else {
                println!(
                    "  PREDICTION: Safe to apply {rate_label} {:.1}%",
                    base_rate * 100.0
                );
            }
        }
    }

    // Apply adjustments
    pan_orbit.target_focus += focus_adjustment;
    pan_orbit.target_radius = current_radius + radius_adjustment;

    // Lock pitch and yaw
    pan_orbit.target_pitch = zoom_state.locked_alpha;
    pan_orbit.target_yaw = zoom_state.locked_beta;
    pan_orbit.force_update = true;

    let balanced = margins.is_balanced(BALANCE_TOLERANCE);
    let fitted = margins.is_fitted(AT_TARGET_TOLERANCE);

    println!(
        "  Correcting: err={:.3}, effective_rate={:.1}%, focus_adj=({:.3},{:.3},{:.3}), radius {:.1}->{:.1}, balanced={}, fitted={}",
        max_rel_error,
        effective_rate * 100.0,
        focus_adjustment.x,
        focus_adjustment.y,
        focus_adjustment.z,
        current_radius,
        pan_orbit.target_radius,
        balanced,
        fitted
    );

    // Early exit if error is negligible to prevent jitter from tiny adjustments
    // When fitted and error < 1%, further iteration just causes oscillation
    if already_fitted && max_rel_error < 0.01 {
        println!(
            "  Zoom-to-fit complete! fitted={}, error negligible ({:.3}%), balanced={}",
            fitted,
            max_rel_error * 100.0,
            balanced
        );
        pan_orbit.zoom_smoothness = zoom_state.original_zoom_smooth;
        pan_orbit.pan_smoothness = zoom_state.original_pan_smooth;
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    // Check completion: balanced AND fitted
    if balanced && fitted {
        println!(
            "  Zoom-to-fit complete! balanced={}, fitted={}",
            balanced, fitted
        );
        pan_orbit.zoom_smoothness = zoom_state.original_zoom_smooth;
        pan_orbit.pan_smoothness = zoom_state.original_pan_smooth;
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    zoom_state.iteration_count += 1;

    // Stop if we hit max iterations
    if zoom_state.iteration_count >= zoom_state.max_iterations {
        println!(
            "Zoom-to-fit stopped at max iterations! balanced={}, fitted={}",
            balanced, fitted
        );
        pan_orbit.zoom_smoothness = zoom_state.original_zoom_smooth;
        pan_orbit.pan_smoothness = zoom_state.original_pan_smooth;
        commands.entity(entity).remove::<ZoomToFitActive>();
    }
}
