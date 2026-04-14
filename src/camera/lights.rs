use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use bevy::color::palettes::tailwind;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use super::RenderLayer;
use super::constants::AMBIENT_LIGHT_BRIGHTNESS;
use super::constants::CASCADE_SHADOW_FIRST_FAR_BOUND;
use super::constants::CASCADE_SHADOW_MAX_DISTANCE;
use super::constants::CASCADE_SHADOW_NUM_CASCADES;
use super::constants::CASCADE_SHADOW_OVERLAP_PROPORTION;
use super::constants::DIRECTIONAL_LIGHT_ILLUMINANCE;
use super::constants::ENVIRONMENT_MAP_INTENSITY;
use super::constants::SHADOW_DEPTH_BIAS;
use super::constants::SHADOW_NORMAL_BIAS;
use crate::input::InspectLightsSwitch;
use crate::orientation::CameraOrientation;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(LightsInspectorEvent);

pub(super) struct DirectionalLightsPlugin;

impl Plugin for DirectionalLightsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalAmbientLight>()
            .add_plugins(
                ResourceInspectorPlugin::<LightSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectLights)),
            )
            .init_resource::<LightSettings>()
            .add_systems(Update, manage_lighting);
        bind_action_switch!(
            app,
            InspectLightsSwitch,
            LightsInspectorEvent,
            Switch::InspectLights
        );
    }
}

/// Whether a directional light should be active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub(super) enum LightSwitch {
    On,
    Off,
}

/// Whether shadow casting is active for a directional light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub(super) enum ShadowSwitch {
    On,
    Off,
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct DirectionalLightSettings {
    pub color:       Color,
    pub enabled:     LightSwitch,
    #[inspector(min = 0.0, max = 10_000.0, display = NumberDisplay::Slider)]
    pub illuminance: f32,
    pub shadows:     ShadowSwitch,
}

impl Default for DirectionalLightSettings {
    fn default() -> Self {
        Self {
            color:       Color::from(tailwind::GRAY_50),
            enabled:     LightSwitch::Off,
            illuminance: DIRECTIONAL_LIGHT_ILLUMINANCE,
            // CRITICAL: Must start disabled. Enabling shadows at startup before the scene
            // is fully initialized breaks rendering (causes stars to disappear). Shadows
            // can be safely enabled at runtime via inspector or code.
            shadows:     ShadowSwitch::Off,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct LightSettings {
    #[inspector(min = 0.0, max = 10_000.0, display = NumberDisplay::Slider)]
    pub ambient_light_brightness:  f32,
    pub ambient_light_color:       Color,
    #[inspector(min = 0.0, max = 100_000.0, display = NumberDisplay::Slider)]
    pub environment_map_intensity: f32,
    pub front:                     DirectionalLightSettings,
    pub back:                      DirectionalLightSettings,
    pub top:                       DirectionalLightSettings,
    pub bottom:                    DirectionalLightSettings,
    pub left:                      DirectionalLightSettings,
    pub right:                     DirectionalLightSettings,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            ambient_light_brightness:  AMBIENT_LIGHT_BRIGHTNESS,
            ambient_light_color:       Color::WHITE,
            environment_map_intensity: ENVIRONMENT_MAP_INTENSITY,
            front:                     DirectionalLightSettings {
                enabled: LightSwitch::On,
                ..Default::default()
            },
            back:                      DirectionalLightSettings {
                enabled: LightSwitch::On,
                ..Default::default()
            },
            top:                       DirectionalLightSettings::default(),
            bottom:                    DirectionalLightSettings::default(),
            left:                      DirectionalLightSettings::default(),
            right:                     DirectionalLightSettings::default(),
        }
    }
}

impl LightSettings {
    pub const fn get_light_settings(&self, position: LightPosition) -> &DirectionalLightSettings {
        match position {
            LightPosition::Front => &self.front,
            LightPosition::Back => &self.back,
            LightPosition::Top => &self.top,
            LightPosition::Bottom => &self.bottom,
            LightPosition::Left => &self.left,
            LightPosition::Right => &self.right,
        }
    }
}

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum LightPosition {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

impl LightPosition {
    pub fn get_rotation(self, orientation: &CameraOrientation) -> RotationInfo {
        match self {
            Self::Right => RotationInfo {
                axis:  orientation.settings.axis_mundi,
                angle: FRAC_PI_2,
            },
            Self::Left => RotationInfo {
                axis:  orientation.settings.axis_mundi,
                angle: -FRAC_PI_2,
            },
            Self::Front => RotationInfo {
                axis:  orientation.settings.axis_orbis,
                angle: 0.,
            },
            Self::Back => RotationInfo {
                axis:  orientation.settings.axis_orbis,
                angle: PI,
            },
            Self::Bottom => RotationInfo {
                axis:  orientation.settings.axis_orbis,
                angle: FRAC_PI_2,
            },
            Self::Top => RotationInfo {
                axis:  orientation.settings.axis_orbis,
                angle: -FRAC_PI_2,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RotationInfo {
    pub axis:  Vec3,
    pub angle: f32,
}

// looked this up on github - so it doesn't really matter where it's placed...
//
// Directional light sources are modelled to be at infinity and have parallel
// rays. As such they do not have a position in practical terms and only the
// rotation matters. The direction of the light is defined by the forward
// direction and by default the -z axis is forwards (right-handed, y-up, z
// points backwards and -z is forwards). Rotations are then applied to a `Vec3` of
// (0,0,-1) to calculate the transform’s forward direction.

#[derive(Component)]
struct LightDirection(LightPosition);

fn spawn_directional_light(
    commands: &mut Commands,
    settings: &DirectionalLightSettings,
    position: LightPosition,
    light_rotation: &RotationInfo,
) {
    commands
        .spawn(DirectionalLight {
            color: settings.color,
            illuminance: settings.illuminance,
            shadows_enabled: matches!(settings.shadows, ShadowSwitch::On),
            shadow_depth_bias: SHADOW_DEPTH_BIAS,
            shadow_normal_bias: SHADOW_NORMAL_BIAS,
            ..default()
        })
        .insert(
            CascadeShadowConfigBuilder {
                num_cascades: CASCADE_SHADOW_NUM_CASCADES,
                maximum_distance: CASCADE_SHADOW_MAX_DISTANCE,
                first_cascade_far_bound: CASCADE_SHADOW_FIRST_FAR_BOUND,
                overlap_proportion: CASCADE_SHADOW_OVERLAP_PROPORTION,
                ..default()
            }
            .build(),
        )
        .insert(Transform::from_rotation(Quat::from_axis_angle(
            light_rotation.axis,
            light_rotation.angle,
        )))
        .insert(RenderLayer::Game.layers())
        .insert(LightDirection(position));
}

fn manage_lighting(
    mut commands: Commands,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    light_settings: Res<LightSettings>,
    camera_orientation: Res<CameraOrientation>,
    mut query: Query<(Entity, &mut DirectionalLight, &LightDirection)>,
) {
    if !light_settings.is_changed() && !light_settings.is_added() {
        return;
    }

    ambient_light.brightness = light_settings.ambient_light_brightness;
    ambient_light.color = light_settings.ambient_light_color;

    // iterate through all possible positions to see if any of them exist...
    // if it's been enabled and it doesn't exist then spawn it
    // if it has changed then update it to what it's changed to
    for position in &[
        LightPosition::Right,
        LightPosition::Left,
        LightPosition::Front,
        LightPosition::Back,
        LightPosition::Bottom,
        LightPosition::Top,
    ] {
        let settings = light_settings.get_light_settings(*position);

        // we always spawn a light with its current `LightDirection` - see
        // if we have the current loop's position in an already spawned entity
        let existing_light = query.iter_mut().find(|(_, _, dir)| dir.0 == *position);

        let light_rotation = position.get_rotation(&camera_orientation);

        match (existing_light, settings.enabled) {
            (Some((_, mut light, _)), LightSwitch::On) => {
                // Update existing light
                light.color = settings.color;
                light.illuminance = settings.illuminance;
                light.shadows_enabled = matches!(settings.shadows, ShadowSwitch::On);
            },
            (Some((entity, _, _)), LightSwitch::Off) => {
                // Remove disabled light
                commands.entity(entity).despawn();
            },
            (None, LightSwitch::On) => {
                // Spawn new light
                spawn_directional_light(&mut commands, settings, *position, &light_rotation);
            },
            (None, LightSwitch::Off) => {}, // Do nothing for disabled lights that don't exist
        }
    }
}
