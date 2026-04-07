use bevy::prelude::*;
use bevy_kana::Displacement;
use bevy_kana::Position;

pub(crate) struct OrientationPlugin;

impl Plugin for OrientationPlugin {
    fn build(&self, app: &mut App) { app.init_resource::<CameraOrientation>(); }
}

// centralize orientation defaults for a quick change-up
// Y-axis (vertical): Axis Mundi
// This represents the central axis of the world, connecting the heavens, earth,
// and underworld.
//
// X-axis (horizontal):
// Axis Orbis: Latin for "axis of the circle" or "axis of the world"
// This could represent the east-west movement of the sun or the horizon line.
//
// Z-axis (depth):
// Axis Profundus: Latin for "deep axis" or "profound axis"
// This could represent the concept of depth or the path between the observer
// and the horizon.
//
// nexus is the center of the game - It suggests a central point where all game
// elements connect or interact, which fits well with the concept of a game's
// core or hub.
//
// locus is the home position of the camera - It implies a specific, fixed point
// of reference, which aligns well with the idea of a camera's home or default
// position.
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
    pub dimension_mode:   DimensionMode,
    pub axis_mundi:       Vec3,
    pub axis_orbis:       Vec3,
    pub axis_profundus:   Vec3,
    pub locus:            Transform,
    pub nexus:            Position,
    pub spaceship_offset: Displacement,
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct CameraOrientation {
    pub orientation: OrientationType,
    pub settings:    OrientationSettings,
}

impl CameraOrientation {
    const DEFAULT_CONFIG: OrientationSettings = OrientationSettings {
        dimension_mode:   DimensionMode::TwoD,
        axis_mundi:       Vec3::ZERO,
        axis_orbis:       Vec3::ZERO,
        axis_profundus:   Vec3::ZERO,
        locus:            Transform::IDENTITY,
        nexus:            Position::new(0.0, 0.0, 0.0),
        spaceship_offset: Displacement::new(0.0, 5.0, -10.0),
    };

    pub(crate) fn set_orientation(&mut self, new_orientation: OrientationType) {
        self.orientation = new_orientation;
        self.settings = match new_orientation {
            OrientationType::TopDown => OrientationSettings {
                axis_mundi: Vec3::Y,
                axis_orbis: Vec3::X,
                axis_profundus: Vec3::Z,
                ..Self::DEFAULT_CONFIG
            },
            OrientationType::BehindSpaceship => OrientationSettings {
                axis_mundi: Vec3::Z,
                axis_orbis: Vec3::X,
                axis_profundus: -Vec3::Y,
                ..Self::DEFAULT_CONFIG
            },
            OrientationType::BehindSpaceship3D => OrientationSettings {
                dimension_mode: DimensionMode::ThreeD,
                axis_mundi: Vec3::Z,
                axis_orbis: Vec3::X,
                axis_profundus: -Vec3::Y,
                ..Self::DEFAULT_CONFIG
            },
        };
    }
}

impl Default for CameraOrientation {
    fn default() -> Self {
        let mut mode = Self {
            orientation: OrientationType::TopDown,
            settings:    Self::DEFAULT_CONFIG,
        };
        mode.set_orientation(OrientationType::TopDown);
        mode
    }
}
