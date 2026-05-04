//! Bloom + `RenderLayers` bug: objects disappear when second camera uses sequential `.insert()`.
//!
//! See <https://github.com/bevyengine/bevy/issues/22000> - you need to have Hdr on all cameras
//! or at least apparently this seems to be the problem.
//!
//! Run: `cargo run --example bloom_layer_bug`

use bevy::camera::visibility::RenderLayers;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;

// camera
const BLOOM_CAMERA_ORDER: isize = 0;
const CAMERA_POSITION: Vec3 = Vec3::new(0.0, 0.0, 10.0);
const GAME_CAMERA_ORDER: isize = 1;
const PRIMARY_CAMERA_CLEAR_COLOR: Color = Color::BLACK;

// colors
const BLOOM_SPHERE_EMISSIVE: LinearRgba = LinearRgba::rgb(50.0, 25.0, 0.0);
const GAME_SPHERE_COLOR: Color = Color::srgb(0.0, 0.8, 0.2);

// lighting
const DIRECTIONAL_LIGHT_ILLUMINANCE: f32 = 10_000.0;
const LIGHT_POSITION: Vec3 = Vec3::new(5.0, 5.0, 5.0);

// render layers
const BLOOM_LAYER_INDEX: usize = 1;
const GAME_LAYER_INDEX: usize = 2;

// spheres
const BLOOM_SPHERE_POSITION: Vec3 = Vec3::new(-2.5, 0.0, 0.0);
const GAME_SPHERE_POSITION: Vec3 = Vec3::new(2.5, 0.0, 0.0);
const SPHERE_RADIUS: f32 = 1.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bloom_layer = RenderLayers::layer(BLOOM_LAYER_INDEX);
    let game_layer = RenderLayers::layer(GAME_LAYER_INDEX);
    let camera_transform =
        Transform::from_translation(CAMERA_POSITION).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera 1: Bloom-enabled, renders layer 1
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: BLOOM_CAMERA_ORDER,
            clear_color: ClearColorConfig::Custom(PRIMARY_CAMERA_CLEAR_COLOR),
            ..default()
        },
        Bloom::NATURAL,
        bloom_layer.clone(),
        camera_transform,
    ));

    // BUG: Camera 2: Bloom layer gets hidden, renders game layer only
    // commands.spawn((
    //     Camera3d::default(),
    //     Camera {
    //         order: GAME_CAMERA_ORDER,
    //         clear_color: ClearColorConfig::None,
    //         ..default()
    //     },
    //     game_layer.clone(),
    //     camera_transform,
    // ));

    // FIX: adding Hdr works
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: GAME_CAMERA_ORDER,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        game_layer.clone(),
        camera_transform,
        Hdr,
    ));

    // Bloom layer: emissive sphere - should glow orange, but invisible with BUG
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(SPHERE_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: BLOOM_SPHERE_EMISSIVE,
            ..default()
        })),
        Transform::from_translation(BLOOM_SPHERE_POSITION),
        bloom_layer.clone(),
    ));

    // Game layer: green sphere - always visible (proves rendering works)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(SPHERE_RADIUS))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: GAME_SPHERE_COLOR,
            ..default()
        })),
        Transform::from_translation(GAME_SPHERE_POSITION),
        game_layer.clone(),
    ));

    // bloom layer light - doesn't make a difference if this light here or not
    // just left it in so people can see for themselves whether it makes a difference or not
    commands.spawn((
        DirectionalLight {
            illuminance: DIRECTIONAL_LIGHT_ILLUMINANCE,
            ..default()
        },
        Transform::from_translation(LIGHT_POSITION).looking_at(Vec3::ZERO, Vec3::Y),
        bloom_layer,
    ));

    // Light for game layer (required to trigger bug)
    // without this light both objects are visible however you spawn them
    // with this light, the emissive sphere is only visible if spawned as a tuple
    commands.spawn((
        DirectionalLight {
            illuminance: DIRECTIONAL_LIGHT_ILLUMINANCE,
            ..default()
        },
        Transform::from_translation(LIGHT_POSITION).looking_at(Vec3::ZERO, Vec3::Y),
        game_layer,
    ));
}
