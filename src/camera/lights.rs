use crate::{
    global_input::{
        GlobalAction,
        toggle_active,
    },
    orientation::CameraOrientation,
};
use bevy::{
    color::palettes::tailwind,
    prelude::*,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};

pub struct DirectionalLightsPlugin;

impl Plugin for DirectionalLightsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AmbientLight>()
            .add_plugins(
                ResourceInspectorPlugin::<LightConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::LightsInspector)),
            )
            .init_resource::<LightConfig>()
            .register_type::<LightConfig>()
            .add_systems(Update, manage_lighting);
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct LightSettings {
    pub color:           Color,
    pub enabled:         bool,
    #[inspector(min = 0.0, max = 10_000.0, display = NumberDisplay::Slider)]
    pub illuminance:     f32,
    pub shadows_enabled: bool,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            color:           Color::from(tailwind::AMBER_400),
            enabled:         false,
            illuminance:     3000.0,
            shadows_enabled: false,
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone)]
#[reflect(Resource, InspectorOptions)]
pub struct LightConfig {
    #[inspector(min = 0.0, max = 1_000.0, display = NumberDisplay::Slider)]
    pub ambient_light_brightness: f32,
    pub ambient_light_color:      Color,
    pub front:                    LightSettings,
    pub back:                     LightSettings,
    pub top:                      LightSettings,
    pub bottom:                   LightSettings,
    pub left:                     LightSettings,
    pub right:                    LightSettings,
}

impl Default for LightConfig {
    fn default() -> Self {
        Self {
            ambient_light_brightness: 100.0,
            ambient_light_color:      Color::WHITE,
            front:                    LightSettings {
                enabled: true,
                ..Default::default()
            },
            back:                     LightSettings {
                enabled: true,
                ..Default::default()
            },
            top:                      LightSettings::default(),
            bottom:                   LightSettings::default(),
            left:                     LightSettings::default(),
            right:                    LightSettings::default(),
        }
    }
}

impl LightConfig {
    pub fn get_light_settings(&self, position: LightPosition) -> &LightSettings {
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
pub enum LightPosition {
    Front,
    Back,
    Top,
    Bottom,
    Left,
    Right,
}

impl LightPosition {
    pub fn get_rotation(&self, orientation: &CameraOrientation) -> RotationInfo {
        use std::f32::consts::{
            FRAC_PI_2,
            PI,
        };
        match self {
            LightPosition::Right => RotationInfo {
                axis:  orientation.config.axis_mundi,
                angle: FRAC_PI_2,
            },
            LightPosition::Left => RotationInfo {
                axis:  orientation.config.axis_mundi,
                angle: -FRAC_PI_2,
            },
            LightPosition::Front => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: 0.,
            },
            LightPosition::Back => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: PI,
            },
            LightPosition::Bottom => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: FRAC_PI_2,
            },
            LightPosition::Top => RotationInfo {
                axis:  orientation.config.axis_orbis,
                angle: -FRAC_PI_2,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RotationInfo {
    pub axis:  Vec3,
    pub angle: f32,
}

// looked this up on github - so it doesn't really matter where it's placed...
//
// Directional light sources are modelled to be at infinity and have parallel
// rays. As such they do not have a position in practical terms and only the
// rotation matters. The direction of the light is defined by the forward
// direction and by default the -z axis is forwards (right-handed, y-up, z
// points backwards and -z is forwards). Rotations are then applied to a Vec3 of
// (0,0,-1) to calculate the transform’s forward direction.

#[derive(Component)]
struct LightDirection(LightPosition);

fn manage_lighting(
    mut commands: Commands,
    mut ambient_light: ResMut<AmbientLight>,
    light_config: Res<LightConfig>,
    camera_orientation: Res<CameraOrientation>,
    mut query: Query<(Entity, &mut DirectionalLight, &LightDirection)>,
) {
    if !light_config.is_changed() {
        return;
    }

    ambient_light.brightness = light_config.ambient_light_brightness;
    ambient_light.color = light_config.ambient_light_color;

    // iterate through all possible positions to see if any of them exist...
    // if it's been enabled and it doesn't exist then spawn it
    // if it has changed then update it to what it's changed to
    for position in [
        LightPosition::Right,
        LightPosition::Left,
        LightPosition::Front,
        LightPosition::Back,
        LightPosition::Bottom,
        LightPosition::Top,
    ]
    .iter()
    {
        let settings = light_config.get_light_settings(*position);

        // we always spawn a light with its current LightDirection - see
        // if we have the current loop's position in an already spawned entity
        let existing_light = query.iter_mut().find(|(_, _, dir)| dir.0 == *position);

        let light_rotation = position.get_rotation(&camera_orientation);

        match (existing_light, settings.enabled) {
            (Some((_, mut light, _)), true) => {
                // Update existing light
                light.color = settings.color;
                light.illuminance = settings.illuminance;
                light.shadows_enabled = settings.shadows_enabled;
            },
            (Some((entity, _, _)), false) => {
                // Remove disabled light
                commands.entity(entity).despawn();
            },
            (None, true) => {
                // Spawn new light
                commands
                    .spawn(DirectionalLight {
                        color: settings.color,
                        illuminance: settings.illuminance,
                        shadows_enabled: settings.shadows_enabled,
                        ..default()
                    })
                    .insert(Transform::from_rotation(Quat::from_axis_angle(
                        light_rotation.axis,
                        light_rotation.angle,
                    )))
                    .insert(LightDirection(*position));
            },
            (None, false) => {}, // Do nothing for disabled lights that don't exist
        }
    }
}
