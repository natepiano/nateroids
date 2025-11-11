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
            .add_systems(
                Update,
                (toggle_stars, update_bloom_settings, update_clear_color),
            );
    }
}

#[derive(Component)]
struct ZoomToFitActive {
    max_iterations:     usize,
    iteration_count:    usize,
    stable_frame_count: usize,
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
            max_iterations:     3000,
            iteration_count:    0,
            stable_frame_count: 0,
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

    let half_tan_vfov = (perspective.fov * 0.5).tan();
    let half_tan_hfov = half_tan_vfov * aspect_ratio;

    // Get boundary corners using the same method as the yellow box code
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

    // Calculate screen-space bounds using actual camera position (matches yellow box)
    // CRITICAL: Use GlobalTransform rotation (not Transform) to match yellow box calculation
    let cam_pos = cam_transform.translation;
    let cam_rot = cam_global.rotation();
    let cam_forward = cam_rot * Vec3::NEG_Z;
    let cam_right = cam_rot * Vec3::X;
    let cam_up = cam_rot * Vec3::Y;

    let mut min_norm_x = f32::INFINITY;
    let mut max_norm_x = f32::NEG_INFINITY;
    let mut min_norm_y = f32::INFINITY;
    let mut max_norm_y = f32::NEG_INFINITY;
    let mut all_in_front = true;

    for corner in &boundary_corners {
        let relative = *corner - cam_pos;
        let depth = relative.dot(cam_forward);

        // Check if corner is behind camera
        if depth <= 0.1 {
            all_in_front = false;
            break;
        }

        let x = relative.dot(cam_right);
        let y = relative.dot(cam_up);

        let norm_x = x / depth;
        let norm_y = y / depth;

        min_norm_x = min_norm_x.min(norm_x);
        max_norm_x = max_norm_x.max(norm_x);
        min_norm_y = min_norm_y.min(norm_y);
        max_norm_y = max_norm_y.max(norm_y);
    }

    // If any corner is behind camera, move camera back
    if !all_in_front {
        println!(
            "Iteration {}: Boundary behind camera, moving back",
            zoom_state.iteration_count
        );
        let boundary_center = boundary_corners.iter().sum::<Vec3>() / boundary_corners.len() as f32;
        pan_orbit.target_focus = boundary_center;
        pan_orbit.target_radius *= 1.5;
        pan_orbit.force_update = true;
        zoom_state.iteration_count += 1;
        return;
    }

    // Calculate center of bounding box in screen space
    let center_x = (min_norm_x + max_norm_x) * 0.5;
    let center_y = (min_norm_y + max_norm_y) * 0.5;

    // Calculate span of bounding box
    let span_x = max_norm_x - min_norm_x;
    let span_y = max_norm_y - min_norm_y;

    println!(
        "Iteration {}: center=({:.6}, {:.6}), span=({:.3}, {:.3}), bounds x:[{:.3}, {:.3}] y:[{:.3}, {:.3}]",
        zoom_state.iteration_count,
        center_x,
        center_y,
        span_x,
        span_y,
        min_norm_x,
        max_norm_x,
        min_norm_y,
        max_norm_y
    );

    const TOLERANCE: f32 = 0.01;

    // Check if we're done
    let centered = center_x.abs() < TOLERANCE && center_y.abs() < TOLERANCE;

    // Check if boundary fits within screen edges (with margin)
    let target_edge_x = half_tan_hfov / camera_config.zoom_multiplier();
    let target_edge_y = half_tan_vfov / camera_config.zoom_multiplier();

    // Check if we're outside viewport (need to zoom out)
    let outside_x = max_norm_x.abs() > half_tan_hfov || min_norm_x.abs() > half_tan_hfov;
    let outside_y = max_norm_y.abs() > half_tan_vfov || min_norm_y.abs() > half_tan_vfov;

    // Check if either dimension is at target (within 0.1% tolerance)
    let x_at_target =
        max_norm_x.abs() > target_edge_x * 0.999 || min_norm_x.abs() > target_edge_x * 0.999;
    let y_at_target =
        max_norm_y.abs() > target_edge_y * 0.999 || min_norm_y.abs() > target_edge_y * 0.999;

    let fitted = (x_at_target || y_at_target) && !outside_x && !outside_y;

    // Require 5 consecutive stable frames before declaring complete
    const REQUIRED_STABLE_FRAMES: usize = 5;

    if centered && fitted {
        zoom_state.stable_frame_count += 1;
        if zoom_state.stable_frame_count >= REQUIRED_STABLE_FRAMES {
            println!(
                "Zoom-to-fit complete! centered={}, fitted={}, outside=({},{}), at_target=({},{}), stable_frames={}",
                centered,
                fitted,
                outside_x,
                outside_y,
                x_at_target,
                y_at_target,
                zoom_state.stable_frame_count
            );
            commands.entity(entity).remove::<ZoomToFitActive>();
            return;
        }
    } else {
        // Reset counter if we're not centered and fitted
        zoom_state.stable_frame_count = 0;
    }

    // Stop if we hit max iterations
    if zoom_state.iteration_count >= zoom_state.max_iterations {
        println!(
            "Zoom-to-fit stopped at max iterations! centered={}, fitted={}, outside=({},{}), at_target=({},{})",
            centered, fitted, outside_x, outside_y, x_at_target, y_at_target
        );
        commands.entity(entity).remove::<ZoomToFitActive>();
        return;
    }

    // Adjust focus to center the boundary
    if !centered {
        // Move focus in screen-space direction with damping to prevent overshoot
        const FOCUS_DAMPING: f32 = 0.5; // Only apply 50% of correction each frame
        let current_radius = pan_orbit.target_radius;
        let offset_x = center_x * current_radius * half_tan_hfov * FOCUS_DAMPING;
        let offset_y = center_y * current_radius * half_tan_vfov * FOCUS_DAMPING;
        let offset_world = cam_right * offset_x + cam_up * offset_y;

        pan_orbit.target_focus += offset_world;
        println!(
            "  Adjusting focus by ({:.3}, {:.3}) in world space",
            offset_world.x, offset_world.y
        );
    }

    // Adjust radius to fit the boundary
    if !fitted {
        let current_radius = pan_orbit.target_radius;

        // Use symmetric 0.25% zoom rates for smooth convergence
        if outside_x || outside_y {
            let radius_adjustment = 1.0025; // 0.25% out
            pan_orbit.target_radius = current_radius * radius_adjustment;
            println!(
                "  Zooming OUT by {:.2}% to {:.1}",
                (radius_adjustment - 1.0) * 100.0,
                current_radius * radius_adjustment
            );
        }
        // If inside viewport and not at target, zoom in
        else if !x_at_target && !y_at_target {
            let radius_adjustment = 0.9975; // 0.25% in (symmetric)
            pan_orbit.target_radius = current_radius * radius_adjustment;
            println!(
                "  Zooming IN by {:.2}% to {:.1}",
                (1.0 - radius_adjustment) * 100.0,
                current_radius * radius_adjustment
            );
        }
    }

    pan_orbit.force_update = true;
    zoom_state.iteration_count += 1;
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

    // The total required distance
    let total_distance = xy_distance + z_half_depth;

    // Apply configured margin
    total_distance * buffer
}

// Helper function that matches the yellow box calculation exactly
fn compute_screen_space_bounds(
    boundary_corners: &[Vec3; 8],
    camera_pos: Vec3,
    camera_rotation: Quat,
) -> (f32, f32, f32, f32) {
    // Get camera basis vectors (matches yellow box code)
    let cam_forward = camera_rotation * Vec3::NEG_Z;
    let cam_right = camera_rotation * Vec3::X;
    let cam_up = camera_rotation * Vec3::Y;

    let mut min_norm_x = f32::INFINITY;
    let mut max_norm_x = f32::NEG_INFINITY;
    let mut min_norm_y = f32::INFINITY;
    let mut max_norm_y = f32::NEG_INFINITY;

    for corner in boundary_corners {
        let relative = *corner - camera_pos;
        let depth = relative.dot(cam_forward).max(0.1);
        let x = relative.dot(cam_right);
        let y = relative.dot(cam_up);

        let norm_x = x / depth;
        let norm_y = y / depth;

        min_norm_x = min_norm_x.min(norm_x);
        max_norm_x = max_norm_x.max(norm_x);
        min_norm_y = min_norm_y.min(norm_y);
        max_norm_y = max_norm_y.max(norm_y);
    }

    (min_norm_x, max_norm_x, min_norm_y, max_norm_y)
}

fn calculate_camera_radius_and_focus_for_angle(
    boundary: &Boundary,
    fov: f32,
    aspect_ratio: f32,
    yaw: f32,
    pitch: f32,
    buffer: f32,
) -> (f32, Vec3) {
    // Calculate half-angle tangents
    let half_tan_vfov = (fov * 0.5).tan();
    let half_tan_hfov = half_tan_vfov * aspect_ratio;

    // Get boundary corners using the same method as the yellow box code
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

    // Compute geometric center of boundary
    let mut boundary_center = Vec3::ZERO;
    for corner in &boundary_corners {
        boundary_center += corner;
    }
    boundary_center /= boundary_corners.len() as f32;

    // Build camera rotation (yaw then pitch)
    let yaw_quat = Quat::from_rotation_y(yaw);
    let pitch_quat = Quat::from_rotation_x(pitch);
    let camera_rotation = yaw_quat * pitch_quat;
    let view_rotation = camera_rotation.inverse();

    // Find the maximum distance of any corner from the boundary center in view space
    // to ensure we start from a position where all corners are in front of camera
    let mut max_corner_distance = 0.0f32;
    for corner in &boundary_corners {
        let corner_in_view = view_rotation * (corner - boundary_center);
        // We care about the z-distance (depth)
        max_corner_distance = max_corner_distance.max(corner_in_view.z.abs());
    }

    // Start from boundary center, then offset backward along view direction
    // by the max corner distance plus margin to ensure all corners are in front
    let view_forward = camera_rotation * Vec3::NEG_Z;
    let initial_focus = boundary_center - view_forward * (max_corner_distance * 1.5);

    // Iteratively center the boundary in screen space
    let mut optimal_focus = initial_focus;
    const MAX_ITERATIONS: usize = 10;
    const TOLERANCE: f32 = 0.001;

    // We need to iterate adjusting both focus and radius together
    let mut current_radius = max_corner_distance * 2.0; // Initial estimate

    for iteration in 0..MAX_ITERATIONS {
        // Camera position in orbit camera: focus + (rotation * Vec3::Z * radius)
        // Camera looks down -Z, so it's at +Z from focus
        let camera_pos = optimal_focus + (camera_rotation * Vec3::Z * current_radius);

        // Project corners to view space relative to camera position (matches yellow box)
        let view_corners: Vec<Vec3> = boundary_corners
            .iter()
            .map(|corner| view_rotation * (corner - camera_pos))
            .collect();

        // Compute screen-space bounding box using same projection as yellow box
        // norm_x = x / depth, norm_y = y / depth (no FOV division yet)
        let mut min_norm_x = f32::INFINITY;
        let mut max_norm_x = f32::NEG_INFINITY;
        let mut min_norm_y = f32::INFINITY;
        let mut max_norm_y = f32::NEG_INFINITY;
        let mut sum_depth = 0.0;
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for v in &view_corners {
            let depth = -v.z; // positive depth
            let norm_x = v.x / depth;
            let norm_y = v.y / depth;

            min_norm_x = min_norm_x.min(norm_x);
            max_norm_x = max_norm_x.max(norm_x);
            min_norm_y = min_norm_y.min(norm_y);
            max_norm_y = max_norm_y.max(norm_y);
            sum_depth += depth;
            min_z = min_z.min(v.z);
            max_z = max_z.max(v.z);
        }

        let avg_depth = sum_depth / view_corners.len() as f32;

        // Center of screen-space bounding box
        let center_x = (min_norm_x + max_norm_x) * 0.5;
        let center_y = (min_norm_y + max_norm_y) * 0.5;

        println!(
            "Iteration {iteration}: screen-space center = ({center_x:.6}, {center_y:.6}), bounds = \
             x:[{min_norm_x:.3}, {max_norm_x:.3}] y:[{min_norm_y:.3}, {max_norm_y:.3}], \
             z:[{min_z:.1}, {max_z:.1}], avg_depth={avg_depth:.1}"
        );

        // Check if we're centered enough
        if center_x.abs() < TOLERANCE && center_y.abs() < TOLERANCE {
            println!("Converged! Final center: ({center_x:.6}, {center_y:.6})");
            break;
        }

        // Convert screen-space offset to view-space shift with FOV scaling
        // Positive center_x means boundary is right of screen center, so move focus right
        let offset_x_view = center_x * avg_depth * half_tan_hfov;
        let offset_y_view = center_y * avg_depth * half_tan_vfov;
        let offset_world = camera_rotation * Vec3::new(offset_x_view, offset_y_view, 0.0);

        println!("  Applying offset: view_space=({offset_x_view:.3}, {offset_y_view:.3})");

        // Update focus
        optimal_focus += offset_world;

        // Recalculate radius for next iteration based on new focus
        let mut next_radius = 0.0f32;
        for corner in &boundary_corners {
            let p = view_rotation * (corner - optimal_focus);
            let depth = -p.z;
            let norm_x = p.x / depth;
            let norm_y = p.y / depth;
            let scale = (norm_x.abs() / half_tan_hfov).max(norm_y.abs() / half_tan_vfov);
            next_radius = next_radius.max(depth * scale);
        }
        current_radius = next_radius;
    }

    // Final convergence check using actual camera position
    let final_camera_pos = optimal_focus + (camera_rotation * Vec3::Z * current_radius);
    let final_view_corners: Vec<Vec3> = boundary_corners
        .iter()
        .map(|corner| view_rotation * (corner - final_camera_pos))
        .collect();

    let mut final_min_x = f32::INFINITY;
    let mut final_max_x = f32::NEG_INFINITY;
    let mut final_min_y = f32::INFINITY;
    let mut final_max_y = f32::NEG_INFINITY;

    for v in &final_view_corners {
        let depth = -v.z;
        let norm_x = v.x / depth;
        let norm_y = v.y / depth;
        final_min_x = final_min_x.min(norm_x);
        final_max_x = final_max_x.max(norm_x);
        final_min_y = final_min_y.min(norm_y);
        final_max_y = final_max_y.max(norm_y);
    }

    let final_center_x = (final_min_x + final_max_x) * 0.5;
    let final_center_y = (final_min_y + final_max_y) * 0.5;
    println!("Final: center = ({final_center_x:.6}, {final_center_y:.6})");

    // Use the radius calculated during the last iteration, with buffer applied
    let final_radius = current_radius * buffer;

    (final_radius, optimal_focus)
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
