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

use crate::{
    actor::ActorPlugin,
    asset_loader::AssetLoaderPlugin,
    camera::CameraPlugin,
    despawn::DespawnPlugin,
    global_input::InputPlugin,
    orientation::OrientationPlugin,
    physics::PhysicsPlugin,
    playfield::PlayfieldPlugin,
    schedule::SchedulePlugin,
    splash::SplashPlugin,
    state::StatePlugin,
};
use bevy::prelude::*;

#[cfg(target_arch = "wasm32")]
use bevy::window::{
    PresentMode,
    WindowMode,
};

use bevy::remote::{
    RemotePlugin,
    http::RemoteHttpPlugin,
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;

fn main() {
    let mut app = App::new();

    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(
        DefaultPlugins
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
    .add_plugins(RemotePlugin::default())
    .add_plugins(RemoteHttpPlugin::default())
    .run();
}
