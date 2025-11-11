use std::collections::VecDeque;

use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::AmbientLight;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::TrackpadBehavior;
use leafwing_input_manager::prelude::*;

use crate::camera::CameraOrder;
use crate::camera::RenderLayer;
use crate::camera::config::CameraConfig;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::playfield::Boundary;

pub struct CamerasPlugin;

impl Plugin for CamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, spawn_star_camera.before(spawn_panorbit_camera))
            .add_systems(Startup, spawn_panorbit_camera)
            .add_systems(Update, home_camera.run_if(just_pressed(GameAction::Home)))
            .add_systems(
                Update,
                start_zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)),
            )
            .add_systems(Update, update_zoom_to_fit)
            .add_systems(Update, move_camera_system)
            .add_systems(
                Update,
                (toggle_stars, update_bloom_settings, update_clear_color),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ZoomDirection {
    In,
    Out,
}

/// Component for programmatically moving the camera to a specific position
/// Used for testing zoom-to-fit from various camera angles
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MoveMe {
    pub target_focus:  Vec3,
    pub target_radius: f32,
    pub target_yaw:    f32, // Rotation around vertical axis (in radians)
    pub target_pitch:  f32, // Rotation up/down (in radians)
    pub speed:         f32, // Interpolation speed (0.0-1.0, higher = faster)
}

#[derive(Component)]
struct ZoomToFitActive {
    max_iterations:        usize,
    iteration_count:       usize,
    previous_bounds:       Option<(f32, f32, f32, f32)>, // (min_x, max_x, min_y, max_y)
    last_zoom_direction:   Option<ZoomDirection>,
    waiting_for_stability: bool, /* True when we've stopped issuing zoom commands, waiting for
                                  * camera to settle */
    margin_history:        VecDeque<(f32, f32)>, // Track (radius, margin) pairs for prediction
}

/// Screen-space margin information for a boundary
pub struct ScreenSpaceMargins {
    /// Distance from left edge (positive = inside, negative = outside)
    pub left_margin:     f32,
    /// Distance from right edge (positive = inside, negative = outside)
    pub right_margin:    f32,
    /// Distance from top edge (positive = inside, negative = outside)
    pub top_margin:      f32,
    /// Distance from bottom edge (positive = inside, negative = outside)
    pub bottom_margin:   f32,
    /// Target margin for horizontal (in screen-space units)
    pub target_margin_x: f32,
    /// Target margin for vertical (in screen-space units)
    pub target_margin_y: f32,
    /// Minimum normalized x coordinate in screen space
    pub min_norm_x:      f32,
    /// Maximum normalized x coordinate in screen space
    pub max_norm_x:      f32,
    /// Minimum normalized y coordinate in screen space
    pub min_norm_y:      f32,
    /// Maximum normalized y coordinate in screen space
    pub max_norm_y:      f32,
    /// Average depth of boundary corners from camera
    pub avg_depth:       f32,
}

impl ScreenSpaceMargins {
    /// Creates screen space margins from a camera's view of a boundary.
    /// Returns `None` if any boundary corner is behind the camera.
    pub fn from_camera_view(
        boundary: &Boundary,
        cam_transform: &Transform,
        cam_global: &GlobalTransform,
        perspective: &PerspectiveProjection,
        viewport_aspect: f32,
        zoom_multiplier: f32,
    ) -> Option<Self> {
        let half_tan_vfov = (perspective.fov * 0.5).tan();
        let half_tan_hfov = half_tan_vfov * viewport_aspect;

        // Get boundary corners
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

        // Get camera basis vectors
        let cam_pos = cam_transform.translation;
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
        avg_depth /= boundary_corners.len() as f32;

        // Screen edges are at ±half_tan_hfov and ±half_tan_vfov
        // Target edges (with margin) are at ±(half_tan_hfov / zoom_multiplier)
        let target_edge_x = half_tan_hfov / zoom_multiplier;
        let target_edge_y = half_tan_vfov / zoom_multiplier;

        // Calculate margins as distance from bounds to screen edges
        // Positive = within screen, negative = outside
        let left_margin = min_norm_x - (-half_tan_hfov);
        let right_margin = half_tan_hfov - max_norm_x;
        let bottom_margin = min_norm_y - (-half_tan_vfov);
        let top_margin = half_tan_vfov - max_norm_y;

        // Target margins are the difference between screen edge and target edge
        let target_margin_x = half_tan_hfov - target_edge_x;
        let target_margin_y = half_tan_vfov - target_edge_y;

        Some(Self {
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            target_margin_x,
            target_margin_y,
            min_norm_x,
            max_norm_x,
            min_norm_y,
            max_norm_y,
            avg_depth,
        })
    }

    /// Returns the minimum margin across all sides
    fn min_margin(&self) -> f32 {
        self.left_margin
            .min(self.right_margin)
            .min(self.top_margin)
            .min(self.bottom_margin)
    }

    /// Returns true if the margins are balanced (opposite sides are equal)
    fn is_balanced(&self, tolerance: f32) -> bool {
        let horizontal_balanced = (self.left_margin - self.right_margin).abs() < tolerance;
        let vertical_balanced = (self.top_margin - self.bottom_margin).abs() < tolerance;
        horizontal_balanced && vertical_balanced
    }

    /// Returns true if properly fitted with correct margins
    /// Requirements:
    /// - All margins >= target (no margin below target)
    /// - One dimension at target, other dimension >= target
    /// - Margins are balanced (left==right, top==bottom)
    fn is_fitted(&self, balance_tolerance: f32, at_target_tolerance: f32) -> bool {
        // All margins must be >= their respective targets (can't have any below)
        let h_min = self.left_margin.min(self.right_margin);
        let v_min = self.top_margin.min(self.bottom_margin);
        let all_margins_sufficient =
            h_min >= self.target_margin_x * 0.99 && v_min >= self.target_margin_y * 0.99;

        // Check if horizontal dimension is at target (use looser tolerance to avoid oscillation)
        let h_at_target = (h_min - self.target_margin_x).abs() < at_target_tolerance;

        // Check if vertical dimension is at target
        let v_at_target = (v_min - self.target_margin_y).abs() < at_target_tolerance;

        // At least one dimension should be at target
        let one_dimension_at_target = h_at_target || v_at_target;

        // Must be balanced (use tight tolerance)
        let balanced = self.is_balanced(balance_tolerance);

        all_margins_sufficient && one_dimension_at_target && balanced
    }
}

pub fn home_camera(
    boundary: Res<Boundary>,
    camera_config: Res<CameraConfig>,
    mut camera_query: Query<(&mut PanOrbitCamera, &Projection)>,
) {
    if let Ok((mut pan_orbit, Projection::Perspective(perspective))) = camera_query.single_mut() {
        let grid_size = boundary.scale();

        let target_radius = calculate_camera_radius(
            grid_size,
            perspective.fov,
            perspective.aspect_ratio,
            camera_config.zoom_multiplier(),
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
fn start_zoom_to_fit(mut commands: Commands, camera_query: Query<Entity, With<PanOrbitCamera>>) {
    if let Ok(camera_entity) = camera_query.single() {
        commands.entity(camera_entity).insert(ZoomToFitActive {
            max_iterations:        3000,
            iteration_count:       0,
            previous_bounds:       None,
            last_zoom_direction:   None,
            waiting_for_stability: false,
            margin_history:        VecDeque::new(),
        });
        println!("Starting zoom-to-fit animation");
    }
}

// Update zoom-to-fit each frame
fn update_zoom_to_fit(
    mut commands: Commands,
    boundary: Res<Boundary>,
    camera_config: Res<CameraConfig>,
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
        camera_config.zoom_multiplier(),
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
        camera_config.zoom_multiplier()
    );

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

    const BALANCE_TOLERANCE: f32 = 0.001; // Tight tolerance for margin balance (left==right, top==bottom)
    const AT_TARGET_TOLERANCE: f32 = 0.003; // Looser tolerance for "at target" check to avoid oscillation

    println!(
        "  Balanced: h_diff={:.3}, v_diff={:.3}, is_balanced={}, is_fitted={}",
        (margins.left_margin - margins.right_margin).abs(),
        (margins.top_margin - margins.bottom_margin).abs(),
        margins.is_balanced(BALANCE_TOLERANCE),
        margins.is_fitted(BALANCE_TOLERANCE, AT_TARGET_TOLERANCE)
    );

    // Use margins struct for proper centered and fitted checks
    let balanced = margins.is_balanced(BALANCE_TOLERANCE);
    let fitted = margins.is_fitted(BALANCE_TOLERANCE, AT_TARGET_TOLERANCE);

    // Check if bounds have actually changed since last frame (detect camera stabilization)
    const BOUNDS_TOLERANCE: f32 = 0.0001;
    let current_bounds = (
        margins.min_norm_x,
        margins.max_norm_x,
        margins.min_norm_y,
        margins.max_norm_y,
    );
    let bounds_changed = if let Some(prev) = zoom_state.previous_bounds {
        (margins.min_norm_x - prev.0).abs() > BOUNDS_TOLERANCE
            || (margins.max_norm_x - prev.1).abs() > BOUNDS_TOLERANCE
            || (margins.min_norm_y - prev.2).abs() > BOUNDS_TOLERANCE
            || (margins.max_norm_y - prev.3).abs() > BOUNDS_TOLERANCE
    } else {
        true // First frame always counts as "changed"
    };
    zoom_state.previous_bounds = Some(current_bounds);

    // Once bounds are stable (unchanged), check if we're done
    // If waiting for stability, bounds stable means camera finished interpolating
    // Otherwise, need to be balanced AND fitted too
    if !bounds_changed && (zoom_state.waiting_for_stability || (balanced && fitted)) {
        println!(
            "  Camera stabilized: bounds unchanged, balanced={}, fitted={}",
            balanced, fitted
        );
        println!(
            "Zoom-to-fit complete! balanced={}, fitted={}",
            balanced, fitted
        );
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    // Stop if we hit max iterations
    if zoom_state.iteration_count >= zoom_state.max_iterations {
        println!(
            "Zoom-to-fit stopped at max iterations! balanced={}, fitted={}",
            balanced, fitted
        );
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    // Phase 1: Balance first (center the boundary)
    // Adjust focus until left==right and top==bottom
    if !balanced {
        // Move focus in screen-space direction with damping to prevent overshoot
        const FOCUS_DAMPING: f32 = 0.5; // Only apply 50% of correction each frame
        let current_radius = pan_orbit.target_radius;
        let offset_x = center_x * current_radius * half_tan_hfov * FOCUS_DAMPING;
        let offset_y = center_y * current_radius * half_tan_vfov * FOCUS_DAMPING;

        // Get camera basis vectors for world space offset calculation
        let cam_rot = cam_global.rotation();
        let cam_right = cam_rot * Vec3::X;
        let cam_up = cam_rot * Vec3::Y;
        let offset_world = cam_right * offset_x + cam_up * offset_y;

        pan_orbit.target_focus += offset_world;
        println!(
            "  Adjusting focus by ({:.3}, {:.3}) in world space",
            offset_world.x, offset_world.y
        );
    }
    // Phase 2: Once balanced, adjust zoom to fit
    // Zoom until one dimension hits target margin
    else if !fitted {
        // Check each dimension against its own target
        let h_min = margins.left_margin.min(margins.right_margin);
        let v_min = margins.top_margin.min(margins.bottom_margin);

        // Calculate margin ratios to find the most constrained dimension
        // Lower ratio = more constrained (closer to minimum required margin)
        let h_ratio = h_min / margins.target_margin_x;
        let v_ratio = v_min / margins.target_margin_y;

        // Determine which dimension is most constrained
        let (constrained_margin, constrained_target, dimension_name) = if h_ratio < v_ratio {
            (h_min, margins.target_margin_x, "horizontal")
        } else {
            (v_min, margins.target_margin_y, "vertical")
        };

        // Calculate error based on the most constrained dimension
        let error = (constrained_margin - constrained_target).abs();
        let error_pct = (error / constrained_target) * 100.0;

        // Track margin history for prediction (keep exactly 3 samples)
        const HISTORY_SIZE: usize = 3;
        let current_radius = pan_orbit.radius.unwrap_or(pan_orbit.target_radius);
        zoom_state
            .margin_history
            .push_back((current_radius, constrained_margin));
        if zoom_state.margin_history.len() > HISTORY_SIZE {
            zoom_state.margin_history.pop_front();
        }

        // Predict final margin when camera reaches target_radius
        let mut predicted_final_margin = None;
        if zoom_state.margin_history.len() == HISTORY_SIZE {
            // Calculate empirical rate: dMargin/dRadius from history
            let (r1, m1) = zoom_state.margin_history[0];
            let (r2, m2) = zoom_state.margin_history[HISTORY_SIZE - 1];
            let delta_radius = r2 - r1;
            if delta_radius.abs() > 0.01 {
                let d_margin_d_radius = (m2 - m1) / delta_radius;
                let remaining_delta = pan_orbit.target_radius - current_radius;
                predicted_final_margin =
                    Some(constrained_margin + (remaining_delta * d_margin_d_radius));

                println!(
                    "  Prediction: current_r={:.1}, target_r={:.1}, delta_r={:.1}, dM/dR={:.4}, predicted_margin={:.3}",
                    current_radius,
                    pan_orbit.target_radius,
                    remaining_delta,
                    d_margin_d_radius,
                    predicted_final_margin.unwrap()
                );
            }
        }

        // If we're waiting for stability, don't issue new zoom commands
        // Just continue logging and checking if bounds have stabilized
        if zoom_state.waiting_for_stability {
            println!(
                "  Waiting for stability ({} constrained: {:.3}/{:.3}, error={:.2}%)",
                dimension_name, constrained_margin, constrained_target, error_pct
            );
            pan_orbit.force_update = true;
            zoom_state.iteration_count += 1;
            return;
        }

        // Use prediction to decide when to stop issuing new zoom commands
        // If predicted final margin is within acceptable range (±20% of target), stop
        if let Some(predicted_margin) = predicted_final_margin {
            let predicted_error =
                ((predicted_margin - constrained_target).abs() / constrained_target) * 100.0;

            if predicted_error <= 20.0 {
                println!(
                    "  Predicted final margin acceptable: {:.3}/{:.3} (error={:.1}%), stopping new zoom commands",
                    predicted_margin, constrained_target, predicted_error
                );
                zoom_state.waiting_for_stability = true;
                pan_orbit.force_update = true;
                zoom_state.iteration_count += 1;
                return;
            }
        }

        let zoom_rate = if error_pct > 20.0 {
            0.02 // 2% when far from target
        } else if error_pct > 5.0 {
            0.01 // 1% when medium distance
        } else if error_pct > 2.0 {
            0.005 // 0.5% when close
        } else {
            0.0025 // 0.25% when very close
        };

        // Determine zoom direction
        let desired_direction = if constrained_margin < constrained_target * 0.99 {
            ZoomDirection::Out
        } else {
            ZoomDirection::In
        };

        // Detect oscillation: if direction changed, we're overshooting due to interpolation
        // Stop issuing new zoom commands but keep logging until camera settles
        if let Some(last_direction) = zoom_state.last_zoom_direction {
            if last_direction != desired_direction {
                println!(
                    "  Oscillation detected: last={:?}, desired={:?} ({} constrained: \
                     {:.3}/{:.3}, h={:.3}, v={:.3})",
                    last_direction,
                    desired_direction,
                    dimension_name,
                    constrained_margin,
                    constrained_target,
                    h_min,
                    v_min
                );
                println!("  Stopping zoom commands, waiting for camera to stabilize...");
                zoom_state.waiting_for_stability = true;
                pan_orbit.force_update = true;
                zoom_state.iteration_count += 1;
                return;
            }
        }

        // Record this zoom direction
        zoom_state.last_zoom_direction = Some(desired_direction);

        // Perform the zoom
        let (radius_adjustment, direction_str) = match desired_direction {
            ZoomDirection::Out => (1.0 + zoom_rate, "OUT"),
            ZoomDirection::In => (1.0 - zoom_rate, "IN"),
        };

        pan_orbit.target_radius = current_radius * radius_adjustment;
        let percentage_change = (radius_adjustment - 1.0).abs() * 100.0;

        println!(
            "  Zooming {} by {:.2}% to {:.1} ({} constrained: {:.3}/{:.3}, h={:.3}, v={:.3}, \
             error={:.1}%)",
            direction_str,
            percentage_change,
            current_radius * radius_adjustment,
            dimension_name,
            constrained_margin,
            constrained_target,
            h_min,
            v_min,
            error_pct
        );
    }

    pan_orbit.force_update = true;
    zoom_state.iteration_count += 1;
}

fn move_camera_system(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut PanOrbitCamera, &MoveMe), With<Camera>>,
) {
    for (entity, mut pan_orbit, move_me) in camera_query.iter_mut() {
        // Interpolate towards target values
        let focus_diff = move_me.target_focus - pan_orbit.target_focus;
        let radius_diff = move_me.target_radius - pan_orbit.target_radius;
        let yaw_diff = move_me.target_yaw - pan_orbit.target_yaw;
        let pitch_diff = move_me.target_pitch - pan_orbit.target_pitch;

        // Check if we're close enough to target
        let close_enough = focus_diff.length() < 0.001
            && radius_diff.abs() < 0.1
            && yaw_diff.abs() < 0.001
            && pitch_diff.abs() < 0.001;

        if close_enough {
            // Snap to exact target
            pan_orbit.target_focus = move_me.target_focus;
            pan_orbit.target_radius = move_me.target_radius;
            pan_orbit.target_yaw = move_me.target_yaw;
            pan_orbit.target_pitch = move_me.target_pitch;
            pan_orbit.force_update = true;

            println!(
                "Camera reached target position: focus={:?}, radius={:.1}, yaw={:.2}, pitch={:.2}",
                move_me.target_focus,
                move_me.target_radius,
                move_me.target_yaw,
                move_me.target_pitch
            );

            // Remove MoveMe component
            commands.entity(entity).remove::<MoveMe>();
        } else {
            // Interpolate towards target
            pan_orbit.target_focus += focus_diff * move_me.speed;
            pan_orbit.target_radius += radius_diff * move_me.speed;
            pan_orbit.target_yaw += yaw_diff * move_me.speed;
            pan_orbit.target_pitch += pitch_diff * move_me.speed;
            pan_orbit.force_update = true;
        }
    }
}

fn calculate_camera_radius(grid_size: Vec3, fov: f32, aspect_ratio: f32, buffer: f32) -> f32 {
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

#[derive(Component, Reflect)]
pub struct StarsCamera;

// star camera uses bloom so it needs to be in its own layer as we don't
// want that effect on the colliders
fn spawn_star_camera(mut commands: Commands, camera_config: Res<CameraConfig>) {
    commands
        .spawn(Camera3d::default())
        .insert(Camera {
            order: CameraOrder::Stars.order(),
            clear_color: ClearColorConfig::Default,
            ..default()
        })
        .insert(Tonemapping::BlenderFilmic)
        .insert(RenderLayers::from_layers(RenderLayer::Stars.layers()))
        .insert(get_bloom_settings(camera_config))
        // CRITICAL: Adding an `AmbientLight` component to the stars camera overrides the
        // global `AmbientLight` resource for this camera only. Without this, the global
        // ambient light (used for lighting game objects) washes out the stars completely,
        // making the background appear black. The brightness value doesn't matter since
        // stars are emissive (self-illuminating), but the component must be present.
        .insert(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        .insert(StarsCamera);
}

// propagate bloom settings back to the camera
fn update_bloom_settings(
    camera_config: Res<CameraConfig>,
    mut q_current_settings: Query<&mut Bloom, With<StarsCamera>>,
) {
    if camera_config.is_changed()
        && let Ok(mut old_bloom_settings) = q_current_settings.single_mut()
    {
        *old_bloom_settings = get_bloom_settings(camera_config);
    }
}

fn get_bloom_settings(camera_config: Res<CameraConfig>) -> Bloom {
    let mut new_bloom_settings = Bloom::NATURAL;

    new_bloom_settings.intensity = camera_config.bloom_intensity;
    new_bloom_settings.low_frequency_boost = camera_config.bloom_low_frequency_boost;
    new_bloom_settings.high_pass_frequency = camera_config.bloom_high_pass_frequency;
    new_bloom_settings.clone()
}

// remove and insert BloomSettings to toggle them off and on
// this can probably be removed now that bloom is pretty well working...
fn toggle_stars(
    mut commands: Commands,
    mut camera: Query<(Entity, Option<&mut Bloom>), With<StarsCamera>>,
    user_input: Res<ActionState<GameAction>>,
    camera_config: Res<CameraConfig>,
) {
    if let Ok(current_bloom_settings) = camera.single_mut() {
        match current_bloom_settings {
            (entity, Some(_)) => {
                if user_input.just_pressed(&GameAction::Stars) {
                    println!("stars off");
                    commands.entity(entity).remove::<Bloom>();
                }
            },
            (entity, None) => {
                if user_input.just_pressed(&GameAction::Stars) {
                    println!("stars on");
                    commands
                        .entity(entity)
                        .insert(get_bloom_settings(camera_config));
                }
            },
        }
    }
}

pub fn spawn_panorbit_camera(
    config: Res<Boundary>,
    camera_config: Res<CameraConfig>,
    mut commands: Commands,
    mut q_stars_camera: Query<Entity, With<StarsCamera>>,
) {
    // we know we have one because we spawn the stars camera prior to this system
    // we're going to attach it to the primary as a child so it always has the same
    // view as the primary camera but can show the stars with bloom while the
    // primary shows everything else
    let stars_camera_entity = q_stars_camera
        .single_mut()
        .expect("why in god's name is there no star's camera?");

    // Use default FOV and aspect ratio values since the camera doesn't exist yet
    // values determined from home_camera
    // i tried having home_camera run on first frame using a run condition but it
    // didn't work it set the correct radius but it didn't actually move the
    // camera - this doesn't make sense hard coding the initial values here
    // sucks but I can live with it for now
    let default_fov = std::f32::consts::FRAC_PI_4;
    let default_aspect_ratio = 1.7777778;
    let grid_size = config.scale();
    let initial_radius = calculate_camera_radius(
        grid_size,
        default_fov,
        default_aspect_ratio,
        camera_config.zoom_multiplier(),
    );

    commands
        .spawn(PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: Some(initial_radius), // Some(config.scale().z * 2.),
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Middle,
            modifier_pan: Some(KeyCode::ShiftLeft),
            zoom_sensitivity: 0.1,
            trackpad_behavior: TrackpadBehavior::BlenderLike {
                modifier_pan:  Some(KeyCode::ShiftLeft),
                modifier_zoom: Some(KeyCode::ControlLeft),
            },
            trackpad_pinch_to_zoom_enabled: true,
            ..default()
        })
        //  .insert(transform)
        .insert(Camera {
            order: CameraOrder::Game.order(),
            // transparent because the game sits on top of the stars
            // this (speculative) clears the depth buffer of bloom information still - allowing
            // the game entities to render correctly without bloom
            clear_color: ClearColorConfig::Custom(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.0))),
            ..default()
        })
        .insert(Tonemapping::TonyMcMapface)
        .insert(RenderLayers::from_layers(RenderLayer::Game.layers()))
        .add_child(stars_camera_entity);
}

// this allows us to use Inspector reflection to manually update ClearColor to
// different values while the game is running from the ui_for_resources provided
// by bevy_inspector_egui
fn update_clear_color(camera_config: Res<CameraConfig>, mut clear_color: ResMut<ClearColor>) {
    if camera_config.is_changed() {
        clear_color.0 = camera_config
            .clear_color
            .darker(camera_config.darkening_factor);
    }
}
