use crate::{
    camera::{config::CameraConfig, CameraOrder, RenderLayer},
    global_input::{just_pressed, GlobalAction},
    orientation::CameraOrientation,
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
            .add_systems(Startup, spawn_star_camera.before(spawn_primary_camera))
            .add_systems(Startup, spawn_primary_camera)
            .add_systems(Update, home_camera.run_if(just_pressed(GlobalAction::Home)))
            .add_systems(Update, (toggle_stars, update_bloom_settings, update_clear_color));
    }
}

fn home_camera(
    orientation: Res<CameraOrientation>,
    mut camera_transform: Query<&mut Transform, With<PrimaryCamera>>,
) {
    if let Ok(mut transform) = camera_transform.get_single_mut() {
        *transform = orientation.config.locus;
    }
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
        if let Ok(mut old_bloom_settings) = q_current_settings.get_single_mut() {
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
    let current_bloom_settings = camera.single_mut();

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

#[derive(Component, Debug)]
pub struct PrimaryCamera;

pub fn spawn_primary_camera(
    camera_config: Res<CameraConfig>,
    config: Res<Boundary>,
    mut commands: Commands,
    mut orientation: ResMut<CameraOrientation>,
    mut q_stars_camera: Query<Entity, With<StarsCamera>>,
) {
    // we know we have one because we spawn the stars camera prior to this system
    // we're going to attach it to the primary as a child so it always has the same
    // view as the primary camera but can show the stars with bloom while the
    // primary shows everything else
    let stars_camera_entity = q_stars_camera
        .get_single_mut()
        .expect("why in god's name is there no star's camera?");

    let transform = Transform::from_xyz(0.0, 0.0, config.scale().z * 2.)
        .looking_at(orientation.config.nexus, orientation.config.axis_mundi);

    orientation.config.locus = transform;

    commands
        .spawn(PanOrbitCamera {
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
        .insert(transform)
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
        //   .insert(InputManagerBundle::with_map(CameraControl::camera_input_map()))
        .add_child(stars_camera_entity)
        .insert(PrimaryCamera);
}

// this allows us to use Inspector reflection to manually update ClearColor to
// different values while the game is running from the ui_for_resources provided
// by bevy_inspector_egui
fn update_clear_color(camera_config: Res<CameraConfig>, mut clear_color: ResMut<ClearColor>) {
    if camera_config.is_changed() {
        clear_color.0 = camera_config.clear_color.darker(camera_config.darkening_factor);
    }
}
