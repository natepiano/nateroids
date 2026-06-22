//! Nateroids - An asteroids-style game built with Bevy 0.18
//!
//! A 3D space shooter featuring:
//! - Portal-based boundary wrapping mechanics
//! - Physics-based asteroid destruction
//! - Dynamic camera system

#[macro_use]
extern crate bevy_kana;
mod input;
#[macro_use]
mod switches;
mod actor;
mod asset_loader;
mod camera;
mod constants;
mod despawn;
mod orientation;
mod physics;
mod playfield;
mod schedule;
mod splash;
mod state;

use bevy::gltf::GltfPlugin;
use bevy::gltf::convert_coordinates::GltfConvertCoordinates;
use bevy::prelude::*;
use bevy_brp_extras::BrpExtrasPlugin;
use bevy_brp_extras::DEFAULT_REMOTE_PORT;
use bevy_clerestory::WindowManagerPlugin;
use bevy_inspector_egui::bevy_egui::EguiPlugin;

use crate::actor::ActorPlugin;
use crate::asset_loader::AssetLoaderPlugin;
use crate::camera::CameraPlugin;
use crate::constants::APPLICATION_TITLE;
use crate::despawn::DespawnPlugin;
use crate::input::EnhancedInputAppPlugin;
use crate::orientation::OrientationPlugin;
use crate::physics::PhysicsPlugin;
use crate::playfield::PlayfieldPlugin;
use crate::schedule::SchedulePlugin;
use crate::splash::SplashPlugin;
use crate::state::StatePlugin;
use crate::switches::SwitchesPlugin;

fn main() {
    let mut app = App::new();

    // Get effective port from `BrpExtrasPlugin` to include in window title if non-default
    let brp_extras_plugin = BrpExtrasPlugin::default();
    let (effective_port, _) = brp_extras_plugin.get_effective_port();
    let window_title = if effective_port == DEFAULT_REMOTE_PORT {
        APPLICATION_TITLE.to_string()
    } else {
        format!("{APPLICATION_TITLE} - {effective_port}")
    };

    app.add_plugins(
        DefaultPlugins
            .set(GltfPlugin {
                convert_coordinates: GltfConvertCoordinates {
                    rotate_scene_entity: true,
                    rotate_meshes:       true,
                },
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: window_title,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins(SwitchesPlugin)
        .add_plugins((
            EguiPlugin::default(),
            EnhancedInputAppPlugin,
            ActorPlugin,
            AssetLoaderPlugin,
            brp_extras_plugin,
            PlayfieldPlugin,
            CameraPlugin,
            DespawnPlugin,
            OrientationPlugin,
            PhysicsPlugin,
            SchedulePlugin,
            SplashPlugin,
            StatePlugin,
            WindowManagerPlugin,
        ))
        .run();
}
