use bevy::prelude::*;

pub struct OrientationPlugin;

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
pub enum OrientationType {
    TopDown,
    BehindSpaceship,
    BehindSpaceship3D,
}

#[derive(Debug, Clone, Reflect)]
pub struct OrientationConfig {
    pub allow_3d:         bool,
    pub axis_mundi:       Vec3,
    pub axis_orbis:       Vec3,
    pub axis_profundus:   Vec3,
    pub locus:            Transform,
    pub nexus:            Vec3,
    pub spaceship_offset: Vec3,
}

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct CameraOrientation {
    pub orientation: OrientationType,
    pub config:      OrientationConfig,
}

impl CameraOrientation {
    const DEFAULT_CONFIG: OrientationConfig = OrientationConfig {
        allow_3d:         false,
        axis_mundi:       Vec3::ZERO,
        axis_orbis:       Vec3::ZERO,
        axis_profundus:   Vec3::ZERO,
        locus:            Transform::IDENTITY,
        nexus:            Vec3::ZERO,
        spaceship_offset: Vec3::new(0.0, 5.0, -10.0),
    };

    pub fn set_orientation(&mut self, new_orientation: OrientationType) {
        self.orientation = new_orientation;
        self.config = match new_orientation {
            OrientationType::TopDown => OrientationConfig {
                axis_mundi: Vec3::Y,
                axis_orbis: Vec3::X,
                axis_profundus: Vec3::Z,
                ..Self::DEFAULT_CONFIG
            },
            OrientationType::BehindSpaceship => OrientationConfig {
                axis_mundi: Vec3::Z,
                axis_orbis: Vec3::X,
                axis_profundus: -Vec3::Y,
                ..Self::DEFAULT_CONFIG
            },
            OrientationType::BehindSpaceship3D => OrientationConfig {
                allow_3d: true,
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
            config:      Self::DEFAULT_CONFIG,
        };
        mode.set_orientation(OrientationType::TopDown);
        mode
    }
}
