use crate::{
    camera::{CameraOrder, RenderLayer, config::CameraConfig},
    global_input::{GlobalAction, just_pressed},
    playfield::Boundary,
};
use bevy::{
    core_pipeline::{bloom::Bloom, tonemapping::Tonemapping},
    prelude::*,
    render::view::RenderLayers,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, TrackpadBehavior};
use leafwing_input_manager::prelude::*;

pub struct CamerasPlugin;

impl Plugin for CamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, spawn_star_camera.before(spawn_panorbit_camera))
            .add_systems(Startup, spawn_panorbit_camera)
            .add_systems(Update, home_camera.run_if(just_pressed(GlobalAction::Home)))
            .add_systems(Update, (toggle_stars, update_bloom_settings, update_clear_color));
    }
}

pub fn home_camera(boundary: Res<Boundary>, mut camera_query: Query<(&mut PanOrbitCamera, &Projection)>) {
    if let Ok((mut pan_orbit, projection)) = camera_query.single_mut() {
        if let Projection::Perspective(perspective) = projection {
            let grid_size = boundary.scale();

            let target_radius = calculate_camera_radius(grid_size, perspective.fov, perspective.aspect_ratio);

            // Set the camera's orbit parameters
            pan_orbit.target_focus = Vec3::ZERO;
            pan_orbit.target_yaw = 0.0;
            pan_orbit.target_pitch = 0.0;
            pan_orbit.target_radius = target_radius;

            pan_orbit.force_update = true;
        }
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

#[derive(Component)]
pub struct StarsCamera;

// star camera uses bloom so it needs to be in its own layer as we don't
// want that effect on the colliders
fn spawn_star_camera(mut commands: Commands, camera_config: Res<CameraConfig>) {
    commands
        .spawn(Camera3d::default())
        .insert(Camera {
            order: CameraOrder::Stars.order(),
            hdr: true, // 1. HDR is required for bloom
            ..default()
        })
        .insert(Tonemapping::BlenderFilmic)
        .insert(RenderLayers::from_layers(RenderLayer::Stars.layers()))
        .insert(get_bloom_settings(camera_config))
        .insert(StarsCamera);
}

// propagate bloom settings back to the camera
fn update_bloom_settings(
    camera_config: Res<CameraConfig>,
    mut q_current_settings: Query<&mut Bloom, With<StarsCamera>>,
) {
    if camera_config.is_changed() {
        if let Ok(mut old_bloom_settings) = q_current_settings.single_mut() {
            *old_bloom_settings = get_bloom_settings(camera_config);
        }
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
    user_input: Res<ActionState<GlobalAction>>,
    camera_config: Res<CameraConfig>,
) {
    if let Ok(current_bloom_settings) = camera.single_mut() {
        match current_bloom_settings {
            (entity, Some(_)) => {
                if user_input.just_pressed(&GlobalAction::Stars) {
                    println!("stars off");
                    commands.entity(entity).remove::<Bloom>();
                }
            },
            (entity, None) => {
                if user_input.just_pressed(&GlobalAction::Stars) {
                    println!("stars on");
                    commands.entity(entity).insert(get_bloom_settings(camera_config));
                }
            },
        }
    }
}

pub fn spawn_panorbit_camera(
    camera_config: Res<CameraConfig>,
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
    // i tried having home_camera run on first frame using a run condition but it didn't work
    // it set the correct radius but it didn't actually move the camera - this doesn't make sense
    // hard coding the initial values here sucks but I can live with it for now
    let default_fov = 0.7853982;
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
                modifier_pan: Some(KeyCode::ShiftLeft),
                modifier_zoom: Some(KeyCode::ControlLeft),
            },
            trackpad_pinch_to_zoom_enabled: true,
            ..default()
        })
        //  .insert(transform)
        .insert(Camera {
            hdr: true,
            order: CameraOrder::Game.order(),
            clear_color: ClearColorConfig::Custom(
                camera_config.clear_color.darker(camera_config.darkening_factor),
            ),
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
        clear_color.0 = camera_config.clear_color.darker(camera_config.darkening_factor);
    }
}
