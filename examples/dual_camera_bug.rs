//! Minimal reproduction of entity ID hash collision bug affecting dual camera rendering.
//!
//! ## Bug Description
//! When spawning a SpotLight after setting up two Camera3d cameras:
//! - WITHOUT the workaround: Stars camera's render phases get cleared after frame 1
//! - WITH the workaround: Stars render correctly every frame
//!
//! ## Expected Behavior
//! Both cameras should continue rendering every frame regardless of what entities
//! are spawned afterward.
//!
//! ## Actual Behavior
//! The first camera (order 0) stops rendering after frame 1 unless we spawn an
//! extra entity to offset entity ID allocation.
//!
//! ## To Test
//! 1. Run with workaround enabled (default): `cargo run --example dual_camera_spotlight_bug`
//!    - You should see green cubes continuously
//! 2. Comment out the `commands.spawn_empty()` line (line ~95)
//! 3. Rebuild and run
//!    - Green cubes will flash for 1 frame then disappear

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

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
    // Spawn some emissive "stars" on render layer 0
    for i in 0..10 {
        let x = (i as f32 - 4.5) * 100.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(10.0, 10.0, 10.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                emissive: LinearRgba::rgb(0.0, 10.0, 0.0),
                ..default()
            })),
            Transform::from_xyz(x, 0.0, -500.0),
            RenderLayers::layer(0),
        ));
    }

    // Spawn a game object on render layer 1
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(50.0, 50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(1),
    ));

    // Camera 1: "Stars camera" - renders layer 0 only, order 0
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(0),
    ));

    // Camera 2: "Game camera" - renders layer 1 only, order 1
    // Transparent clear color preserves stars underneath
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
    ));

    // Spawn a SpotLight on layer 1
    commands.spawn((
        SpotLight {
            intensity: 1_000_000.0,
            range: 1_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::layer(1),
    ));

    // WORKAROUND: Spawn an empty entity to offset entity ID allocation
    // Comment this line out to see the bug (stars will flash then disappear)
    commands.spawn_empty();

    info!("Setup complete. Green cubes should be visible continuously.");
    info!("If cubes flash then disappear, the bug is reproduced.");
}
