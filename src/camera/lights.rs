use std::f32::consts::FRAC_PI_2;
use std::f32::consts::PI;

use bevy::light::CascadeShadowConfigBuilder;
use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use super::RenderLayer;
use super::constants::AMBIENT_LIGHT_BRIGHTNESS;
use super::constants::AMBIENT_LIGHT_BRIGHTNESS_MAX;
use super::constants::AMBIENT_LIGHT_BRIGHTNESS_MIN;
use super::constants::CASCADE_SHADOW_FIRST_FAR_BOUND;
use super::constants::CASCADE_SHADOW_MAX_DISTANCE;
use super::constants::CASCADE_SHADOW_NUM_CASCADES;
use super::constants::CASCADE_SHADOW_OVERLAP_PROPORTION;
use super::constants::DIRECTIONAL_LIGHT_ILLUMINANCE;
use super::constants::DIRECTIONAL_LIGHT_ILLUMINANCE_MAX;
use super::constants::DIRECTIONAL_LIGHT_ILLUMINANCE_MIN;
use super::constants::DIRECTIONAL_LIGHT_SETTINGS_COLOR;
use super::constants::ENVIRONMENT_MAP_INTENSITY;
use super::constants::ENVIRONMENT_MAP_INTENSITY_MAX;
use super::constants::ENVIRONMENT_MAP_INTENSITY_MIN;
use super::constants::SHADOW_DEPTH_BIAS;
use super::constants::SHADOW_NORMAL_BIAS;
use crate::input::InspectLightsSwitch;
use crate::orientation::CameraOrientation;
use crate::switches;
use crate::switches::Switch;

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
    pub(super) color:         Color,
    pub(super) light_switch:  LightSwitch,
    #[inspector(
        min = DIRECTIONAL_LIGHT_ILLUMINANCE_MIN,
        max = DIRECTIONAL_LIGHT_ILLUMINANCE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) illuminance:   f32,
    pub(super) shadow_switch: ShadowSwitch,
}

impl Default for DirectionalLightSettings {
    fn default() -> Self {
        Self {
            color:         DIRECTIONAL_LIGHT_SETTINGS_COLOR,
            light_switch:  LightSwitch::Off,
            illuminance:   DIRECTIONAL_LIGHT_ILLUMINANCE,
            // CRITICAL: Must start disabled. Enabling shadows at startup before the scene
            // is fully initialized breaks rendering (causes stars to disappear). Shadows
            // can be safely enabled at runtime via inspector or code.
            shadow_switch: ShadowSwitch::Off,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct LightSettings {
    #[inspector(
        min = AMBIENT_LIGHT_BRIGHTNESS_MIN,
        max = AMBIENT_LIGHT_BRIGHTNESS_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) ambient_brightness:        f32,
    pub(super) ambient_color:             Color,
    #[inspector(
        min = ENVIRONMENT_MAP_INTENSITY_MIN,
        max = ENVIRONMENT_MAP_INTENSITY_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) environment_map_intensity: f32,
    pub(super) front:                     DirectionalLightSettings,
    pub(super) back:                      DirectionalLightSettings,
    pub(super) top:                       DirectionalLightSettings,
    pub(super) bottom:                    DirectionalLightSettings,
    pub(super) left:                      DirectionalLightSettings,
    pub(super) right:                     DirectionalLightSettings,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            ambient_brightness:        AMBIENT_LIGHT_BRIGHTNESS,
            ambient_color:             Color::WHITE,
            environment_map_intensity: ENVIRONMENT_MAP_INTENSITY,
            front:                     DirectionalLightSettings {
                light_switch: LightSwitch::On,
                ..Default::default()
            },
            back:                      DirectionalLightSettings {
                light_switch: LightSwitch::On,
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
    const fn get_light_settings(&self, light_position: LightPosition) -> &DirectionalLightSettings {
        match light_position {
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
    const fn get_rotation(self, camera_orientation: &CameraOrientation) -> RotationInfo {
        match self {
            Self::Right => RotationInfo {
                axis:  camera_orientation.orientation_settings.axis_mundi,
                angle: FRAC_PI_2,
            },
            Self::Left => RotationInfo {
                axis:  camera_orientation.orientation_settings.axis_mundi,
                angle: -FRAC_PI_2,
            },
            Self::Front => RotationInfo {
                axis:  camera_orientation.orientation_settings.axis_orbis,
                angle: 0.,
            },
            Self::Back => RotationInfo {
                axis:  camera_orientation.orientation_settings.axis_orbis,
                angle: PI,
            },
            Self::Bottom => RotationInfo {
                axis:  camera_orientation.orientation_settings.axis_orbis,
                angle: FRAC_PI_2,
            },
            Self::Top => RotationInfo {
                axis:  camera_orientation.orientation_settings.axis_orbis,
                angle: -FRAC_PI_2,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RotationInfo {
    pub(super) axis:  Vec3,
    pub(super) angle: f32,
}

// `DirectionalLight` uses rotation only: Bevy models directional light sources
// at infinity with parallel rays, so light placement has no practical effect.
// The direction of the light is defined by the forward direction and by default
// the -z axis is forwards (right-handed, y-up, z points backwards and -z is
// forwards). Rotations are then applied to a `Vec3` of (0,0,-1) to calculate the
// transform’s forward direction.

#[derive(Component)]
struct LightDirection(LightPosition);

fn spawn_directional_light(
    commands: &mut Commands,
    directional_light_settings: &DirectionalLightSettings,
    light_position: LightPosition,
    light_rotation: &RotationInfo,
) {
    commands
        .spawn(DirectionalLight {
            color: directional_light_settings.color,
            illuminance: directional_light_settings.illuminance,
            shadows_enabled: matches!(directional_light_settings.shadow_switch, ShadowSwitch::On),
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
        .insert(LightDirection(light_position));
}

fn manage_lighting(
    mut commands: Commands,
    mut global_ambient_light: ResMut<GlobalAmbientLight>,
    light_settings: Res<LightSettings>,
    camera_orientation: Res<CameraOrientation>,
    mut directional_light_query: Query<(Entity, &mut DirectionalLight, &LightDirection)>,
) {
    if !light_settings.is_changed() && !light_settings.is_added() {
        return;
    }

    global_ambient_light.brightness = light_settings.ambient_brightness;
    global_ambient_light.color = light_settings.ambient_color;

    // Reconcile each `LightPosition` with its `DirectionalLightSettings` and
    // any spawned `DirectionalLight` tagged by `LightDirection`.
    for light_position in &[
        LightPosition::Right,
        LightPosition::Left,
        LightPosition::Front,
        LightPosition::Back,
        LightPosition::Bottom,
        LightPosition::Top,
    ] {
        let directional_light_settings = light_settings.get_light_settings(*light_position);

        // `directional_light_query` stores spawned lights by `LightDirection`;
        // match the current `LightPosition` before updating or spawning.
        let existing_light = directional_light_query
            .iter_mut()
            .find(|(_, _, direction)| direction.0 == *light_position);

        let light_rotation = light_position.get_rotation(&camera_orientation);

        match (existing_light, directional_light_settings.light_switch) {
            (Some((_, mut light, _)), LightSwitch::On) => {
                // Update the existing `DirectionalLight`.
                light.color = directional_light_settings.color;
                light.illuminance = directional_light_settings.illuminance;
                light.shadows_enabled =
                    matches!(directional_light_settings.shadow_switch, ShadowSwitch::On);
            },
            (Some((entity, _, _)), LightSwitch::Off) => {
                // Despawn the disabled `DirectionalLight` entity.
                commands.entity(entity).despawn();
            },
            (None, LightSwitch::On) => {
                // Spawn the missing `DirectionalLight`.
                spawn_directional_light(
                    &mut commands,
                    directional_light_settings,
                    *light_position,
                    &light_rotation,
                );
            },
            (None, LightSwitch::Off) => {}, /* No `DirectionalLight` exists for the disabled
                                             * setting. */
        }
    }
}
