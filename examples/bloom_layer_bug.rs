//! Bloom + `RenderLayers` bug: objects disappear when second camera uses sequential `.insert()`.
//!
//! See https://github.com/bevyengine/bevy/issues/22000 - you need to have Hdr on all cameras
//! or at least apparently this seems to be the problem.
//!
//! Run: `cargo run --example bloom_layer_bug`

use bevy::camera::visibility::RenderLayers;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;

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
    let bloom_layer = RenderLayers::layer(1);
    let game_layer = RenderLayers::layer(2);
    let camera_pos = Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y);

    // Camera 1: Bloom-enabled, renders layer 1
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Bloom::NATURAL,
        bloom_layer.clone(),
        camera_pos,
    ));

    // BUG: Camera 2: Bloom layer gets hidden, renders game layer only
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        game_layer.clone(),
        camera_pos,
    ));

    // FIX: adding Hdr works
    // commands.spawn((
    //     Camera3d::default(),
    //     Camera {
    //         order: 1,
    //         clear_color: ClearColorConfig::None,
    //         ..default()
    //     },
    //     game_layer.clone(),
    //     camera_pos,
    //     Hdr,
    // ));

    // Bloom layer: emissive sphere - should glow orange, but invisible with BUG
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(50.0, 25.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(-2.5, 0.0, 0.0),
        bloom_layer.clone(),
    ));

    // Game layer: green sphere - always visible (proves rendering works)
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.8, 0.2),
            ..default()
        })),
        Transform::from_xyz(2.5, 0.0, 0.0),
        game_layer.clone(),
    ));

    // bloom layer light - doesn't make a difference if this light here or not
    // just left it in so people can see for themselves whether it makes a difference or not
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        bloom_layer,
    ));

    // Light for game layer (required to trigger bug)
    // without this light both objects are visible however you spawn them
    // with this light, the emissive sphere is only visible if spawned as a tuple
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        game_layer,
    ));
}
