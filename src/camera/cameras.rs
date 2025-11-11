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
                zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)),
            )
            .add_systems(
                Update,
                (toggle_stars, update_bloom_settings, update_clear_color),
            );
    }
}

pub fn home_camera(
    boundary: Res<Boundary>,
    mut camera_query: Query<(&mut PanOrbitCamera, &Projection)>,
) {
    if let Ok((mut pan_orbit, Projection::Perspective(perspective))) = camera_query.single_mut() {
        let grid_size = boundary.scale();

        let target_radius =
            calculate_camera_radius(grid_size, perspective.fov, perspective.aspect_ratio);

        // Set the camera's orbit parameters
        pan_orbit.target_focus = Vec3::ZERO;
        pan_orbit.target_yaw = 0.0;
        pan_orbit.target_pitch = 0.0;
        pan_orbit.target_radius = target_radius;

        pan_orbit.force_update = true;
    }
}

pub fn zoom_to_fit(
    boundary: Res<Boundary>,
    mut camera_query: Query<(&mut PanOrbitCamera, &Projection)>,
) {
    if let Ok((mut pan_orbit, Projection::Perspective(perspective))) = camera_query.single_mut() {
        let grid_size = boundary.scale();

        let (target_radius, target_focus) = calculate_camera_radius_and_focus_for_angle(
            grid_size,
            perspective.fov,
            perspective.aspect_ratio,
            pan_orbit.target_yaw,
            pan_orbit.target_pitch,
        );

        // Keep current yaw and pitch, adjust focus to center boundary, adjust radius
        pan_orbit.target_focus = target_focus;
        pan_orbit.target_radius = target_radius;

        pan_orbit.force_update = true;
    }
}

fn calculate_camera_radius(grid_size: Vec3, fov: f32, aspect_ratio: f32) -> f32 {
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

    // Apply minimal margin
    let buffer = 1.05; // 5% margin
    total_distance * buffer
}

fn calculate_camera_radius_and_focus_for_angle(
    grid_size: Vec3,
    fov: f32,
    aspect_ratio: f32,
    yaw: f32,
    pitch: f32,
) -> (f32, Vec3) {
    // Calculate horizontal FOV
    let horizontal_fov = 2.0 * ((fov / 2.0).tan() * aspect_ratio).atan();
    let half_tan_hfov = (horizontal_fov / 2.0).tan();
    let half_tan_vfov = (fov / 2.0).tan();

    // Generate the 8 corners of the boundary box
    let half_size = grid_size / 2.0;
    let corners = [
        Vec3::new(-half_size.x, -half_size.y, -half_size.z),
        Vec3::new(half_size.x, -half_size.y, -half_size.z),
        Vec3::new(-half_size.x, half_size.y, -half_size.z),
        Vec3::new(half_size.x, half_size.y, -half_size.z),
        Vec3::new(-half_size.x, -half_size.y, half_size.z),
        Vec3::new(half_size.x, -half_size.y, half_size.z),
        Vec3::new(-half_size.x, half_size.y, half_size.z),
        Vec3::new(half_size.x, half_size.y, half_size.z),
    ];

    // Build camera rotation (yaw then pitch)
    let yaw_quat = Quat::from_rotation_y(yaw);
    let pitch_quat = Quat::from_rotation_x(pitch);
    let camera_rotation = yaw_quat * pitch_quat;
    let view_rotation = camera_rotation.inverse();

    // Transform all corners to camera view space
    let mut rotated_corners = Vec::new();
    for corner in &corners {
        rotated_corners.push(view_rotation * (*corner));
    }

    // Find 3D bounding box in view space
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    for rotated in &rotated_corners {
        min_x = min_x.min(rotated.x);
        max_x = max_x.max(rotated.x);
        min_y = min_y.min(rotated.y);
        max_y = max_y.max(rotated.y);
        min_z = min_z.min(rotated.z);
        max_z = max_z.max(rotated.z);
    }

    // Center of the 3D bounding box in view space
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let center_z = (min_z + max_z) / 2.0;

    // The offset in view space that centers the bounding box
    let offset_in_view = Vec3::new(center_x, center_y, center_z);

    // Transform back to world space
    let optimal_focus = camera_rotation * offset_in_view;

    // Now recalculate corners relative to this new focus
    // In view space, they'll be centered around (0, 0)
    let mut centered_corners = Vec::new();
    for corner in &corners {
        let relative = *corner - optimal_focus;
        centered_corners.push(view_rotation * relative);
    }

    // Calculate minimum radius needed to fit all corners
    // For each corner at (x, y, z) to be visible:
    // |x| <= (R - z) * tan(hfov/2)  =>  R >= |x|/tan(hfov/2) + z
    // |y| <= (R - z) * tan(vfov/2)  =>  R >= |y|/tan(vfov/2) + z
    let mut min_radius = 0.0f32;

    for centered in &centered_corners {
        let r_from_x = centered.x.abs() / half_tan_hfov + centered.z;
        let r_from_y = centered.y.abs() / half_tan_vfov + centered.z;
        let r_for_corner = r_from_x.max(r_from_y);
        min_radius = min_radius.max(r_for_corner);
    }

    // Small buffer for safety
    (min_radius * 1.01, optimal_focus)
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
    let initial_radius = calculate_camera_radius(grid_size, default_fov, default_aspect_ratio);

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
