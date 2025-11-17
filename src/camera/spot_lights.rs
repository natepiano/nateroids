use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;

pub struct SpotLightsPlugin;

impl Plugin for SpotLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<SpotLightConfig>::default()
                .run_if(toggle_active(false, GameAction::SpotLightsInspector)),
        )
        .init_resource::<SpotLightConfig>()
        .add_systems(
            Startup,
            setup_spot_lights.after(super::cameras::spawn_panorbit_camera),
        )
        .add_systems(Update, update_spot_lights);
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone)]
#[reflect(Resource, InspectorOptions)]
struct SpotLightConfig {
    #[inspector(min = 0.0, max = 5_000_000.0, display = NumberDisplay::Slider)]
    intensity:   f32,
    #[inspector(min = 0.0, max = 10_000.0, display = NumberDisplay::Slider)]
    range:       f32,
    #[inspector(min = 0.0, max = 3.1415, display = NumberDisplay::Slider)]
    rotation:    f32,
    translation: Vec3,
}

impl Default for SpotLightConfig {
    fn default() -> Self {
        Self {
            intensity:   1_000_000.0,
            range:       1_000.0,
            rotation:    std::f32::consts::PI,
            translation: Vec3::ZERO,
        }
    }
}

fn setup_spot_lights(mut commands: Commands, camera_entity: Single<Entity, With<PanOrbitCamera>>) {
    // let spot_light_entity =
    commands.spawn((
        // SpotLight::default(),
        // Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        SpotLight {
            intensity: 1_000_000.0,
            range: 1_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 500.0)).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::from_layers(RenderLayer::Game.layers()),
    ));
    // .id();

    // WORKAROUND for Bevy entity ID hash collision bug:
    // Spawning an entity with `Transform` changes entity ID/archetype allocation, affecting
    // hash map bucket placement for cameras in the render world. Without this, the Stars
    // camera's render phases get cleared after frame 1. Note: `spawn_empty()` does NOT work
    // - it must have a Transform component. See examples/dual_camera_spotlight_bug.rs
    let _workaround = commands.spawn(Transform::default()).id();

    // commands.entity(*camera_entity).add_child(spot_light_entity);
}

fn update_spot_lights(
    mut spotlight: Single<(&mut SpotLight, &mut Transform), With<SpotLight>>,
    spot_light_config: ResMut<SpotLightConfig>,
) {
    let (ref mut spotlight, ref mut transform) = *spotlight;
    spotlight.intensity = spot_light_config.intensity;
    spotlight.range = spot_light_config.range;
    transform.rotation = Quat::from_rotation_y(spot_light_config.rotation);
    transform.translation = spot_light_config.translation;

    info!(
        "Spotlight updated to {} intensity, {} range, {} rotation",
        spot_light_config.intensity, spot_light_config.range, transform.rotation
    );
}
