// exclude when targeting wasm - this breaks in the browser right now
mod actor;
mod asset_loader;
mod camera;
mod despawn;
mod global_input;
mod orientation;
mod physics;
mod playfield;
mod schedule;
mod splash;
mod state;
mod traits;

use bevy::gltf::GltfPlugin;
use bevy::prelude::*;
#[cfg(target_arch = "wasm32")]
use bevy::window::PresentMode;
#[cfg(target_arch = "wasm32")]
use bevy::window::WindowMode;
use bevy_brp_extras::BrpExtrasPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;

use crate::actor::ActorPlugin;
use crate::asset_loader::AssetLoaderPlugin;
use crate::camera::CameraPlugin;
use crate::despawn::DespawnPlugin;
use crate::global_input::InputPlugin;
use crate::orientation::OrientationPlugin;
use crate::physics::PhysicsPlugin;
use crate::playfield::PlayfieldPlugin;
use crate::schedule::SchedulePlugin;
use crate::splash::SplashPlugin;
use crate::state::StatePlugin;

fn main() {
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins.set(GltfPlugin {
        use_model_forward_direction: true,
        ..default()
    }));

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
            .set(GltfPlugin {
                use_model_forward_direction: true,
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync, // Reduces input lag.
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins((
        EguiPlugin::default(),
        ActorPlugin,
        AssetLoaderPlugin,
        BrpExtrasPlugin::default(),
        PlayfieldPlugin,
        CameraPlugin,
        DespawnPlugin,
        InputPlugin,
        OrientationPlugin,
        PhysicsPlugin,
        SchedulePlugin,
        SplashPlugin,
        StatePlugin,
    ))
    .run();
}
