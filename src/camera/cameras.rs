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
    max_iterations:       usize,
    iteration_count:      usize,
    previous_bounds:      Option<(f32, f32, f32, f32)>, // (min_x, max_x, min_y, max_y)
    original_zoom_smooth: f32,
    original_pan_smooth:  f32,
    locked_alpha:         f32, // Lock pitch during zoom-to-fit
    locked_beta:          f32, // Lock yaw during zoom-to-fit
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

        // At least one dimension should be at target (the constraining dimension)
        // The other dimension will have >= target margin due to aspect ratio differences
        let one_dimension_at_target = h_at_target || v_at_target;

        // Must be balanced (use tight tolerance)
        let balanced = self.is_balanced(balance_tolerance);

        all_margins_sufficient && one_dimension_at_target && balanced
    }

    /// Returns true if horizontal is the constraining dimension
    /// The constraining dimension is the one with larger span (takes up more screen space)
    fn is_horizontal_constraining(&self) -> bool {
        let h_span = self.max_norm_x - self.min_norm_x;
        let v_span = self.max_norm_y - self.min_norm_y;
        h_span > v_span
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
            max_iterations: 3000,
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

    const BALANCE_TOLERANCE: f32 = 0.002; // 0.2% tolerance - margins must be nearly equal
    const AT_TARGET_TOLERANCE: f32 = 0.05; // 5% tolerance for "at target" check

    println!(
        "  Balanced: h_diff={:.3}, v_diff={:.3}, is_balanced={}, is_fitted={}",
        (margins.left_margin - margins.right_margin).abs(),
        (margins.top_margin - margins.bottom_margin).abs(),
        margins.is_balanced(BALANCE_TOLERANCE),
        margins.is_fitted(BALANCE_TOLERANCE, AT_TARGET_TOLERANCE)
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

    let target_radius = if current_margin_val < 0.0 {
        // Content outside view - zoom out by 25%
        current_radius * 1.25
    } else {
        // Cap the ratio to prevent huge jumps (max 50% change per iteration)
        let ratio = (target_margin_val / current_margin_val.max(0.001)).clamp(0.5, 1.5);
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

    // Check if constraining dimension is already at target (within 5%)
    // If so, we're just centering - no need for prediction
    let margin_at_target =
        (current_margin_val - target_margin_val).abs() / target_margin_val < 0.05;

    // Predictive approach: Start with aggressive rate, but validate it won't overshoot
    let base_rate = if margin_at_target {
        0.20 // Very aggressive when just centering
    } else {
        0.12 // Normal aggressive rate
    };

    let mut focus_adjustment = focus_delta * base_rate;
    let mut radius_adjustment = radius_delta * base_rate;

    // Only do prediction if we're not at target yet (if at target, we're just centering)
    if !margin_at_target {
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
            camera_config.zoom_multiplier(),
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
                // Dimension flip would cause oscillation - apply moderate damping
                let damping = 0.50; // 50% of the aggressive rate
                focus_adjustment *= damping;
                radius_adjustment *= damping;
                println!(
                    "  PREDICTION: Would flip dimension ({}→{})! Damping to {:.1}%",
                    if current_constraining_is_h { "H" } else { "V" },
                    if proposed_constraining_is_h { "H" } else { "V" },
                    damping * 100.0
                );
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

                    focus_adjustment *= scale_factor;
                    radius_adjustment *= scale_factor;
                    println!(
                        "  PREDICTION: Would overshoot! current={:.3}, proposed={:.3}, target={:.3}, scaling to {:.1}% (exact)",
                        current_margin_val,
                        proposed_min_margin,
                        target_margin_val,
                        scale_factor * 100.0
                    );
                } else {
                    println!(
                        "  PREDICTION: Safe to apply aggressive rate {:.1}%",
                        base_rate * 100.0
                    );
                }
            }
        }
    } else {
        println!(
            "  PREDICTION: At target margin - using aggressive centering rate {:.1}%",
            base_rate * 100.0
        );
    }

    // Apply adjustments
    pan_orbit.target_focus += focus_adjustment;
    pan_orbit.target_radius = current_radius + radius_adjustment;

    // Lock pitch and yaw
    pan_orbit.target_pitch = zoom_state.locked_alpha;
    pan_orbit.target_yaw = zoom_state.locked_beta;
    pan_orbit.force_update = true;

    let balanced = margins.is_balanced(BALANCE_TOLERANCE);
    let fitted = margins.is_fitted(BALANCE_TOLERANCE, AT_TARGET_TOLERANCE);

    println!(
        "  Correcting: err={:.3}, rate={:.1}%, focus_adj=({:.3},{:.3},{:.3}), radius {:.1}->{:.1}, balanced={}, fitted={}",
        max_rel_error,
        base_rate * 100.0,
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
            zoom_sensitivity: 0.2,
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
