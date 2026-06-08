use bevy::prelude::*;
use bevy_kana::Displacement;
use bevy_kana::Position;

use crate::constants::CAMERA_ORIENTATION_DEFAULT_SETTINGS;

pub(crate) struct OrientationPlugin;

impl Plugin for OrientationPlugin {
    fn build(&self, app: &mut App) { app.init_resource::<CameraOrientation>(); }
}

// `OrientationType` selects the `OrientationSettings` basis used by
// `CameraOrientation`.
// `axis_mundi` is the local up axis, `axis_orbis` is the local horizontal axis,
// and `axis_profundus` is the local depth axis.
// `nexus: Position` stores the world-space center, and `locus: Transform`
// stores the camera home transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub(crate) enum OrientationType {
    TopDown,
    BehindSpaceship,
    BehindSpaceship3D,
}

/// Whether the orientation permits full 3D movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub(crate) enum DimensionMode {
    TwoD,
    ThreeD,
}

#[derive(Debug, Clone, Reflect)]
pub(crate) struct OrientationSettings {
    pub(crate) dimension_mode:   DimensionMode,
    pub(crate) axis_mundi:       Vec3,
    pub(crate) axis_orbis:       Vec3,
    pub(crate) axis_profundus:   Vec3,
    pub(crate) locus:            Transform,
    pub(crate) nexus:            Position,
    pub(crate) spaceship_offset: Displacement,
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct CameraOrientation {
    pub(crate) orientation_type:     OrientationType,
    pub(crate) orientation_settings: OrientationSettings,
}

impl CameraOrientation {
    pub(crate) fn set_orientation(&mut self, new_orientation: OrientationType) {
        self.orientation_type = new_orientation;
        self.orientation_settings = match new_orientation {
            OrientationType::TopDown => OrientationSettings {
                axis_mundi: Vec3::Y,
                axis_orbis: Vec3::X,
                axis_profundus: Vec3::Z,
                ..CAMERA_ORIENTATION_DEFAULT_SETTINGS
            },
            OrientationType::BehindSpaceship => OrientationSettings {
                axis_mundi: Vec3::Z,
                axis_orbis: Vec3::X,
                axis_profundus: -Vec3::Y,
                ..CAMERA_ORIENTATION_DEFAULT_SETTINGS
            },
            OrientationType::BehindSpaceship3D => OrientationSettings {
                dimension_mode: DimensionMode::ThreeD,
                axis_mundi: Vec3::Z,
                axis_orbis: Vec3::X,
                axis_profundus: -Vec3::Y,
                ..CAMERA_ORIENTATION_DEFAULT_SETTINGS
            },
        };
    }
}

impl Default for CameraOrientation {
    fn default() -> Self {
        let mut camera_orientation = Self {
            orientation_type:     OrientationType::TopDown,
            orientation_settings: CAMERA_ORIENTATION_DEFAULT_SETTINGS,
        };
        camera_orientation.set_orientation(OrientationType::TopDown);
        camera_orientation
    }
}
