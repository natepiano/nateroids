use bevy::math::Vec3;

/// Initial camera distance for splash screen animation.
/// Camera spawns at this distance to appear stationary during the opening text.
pub const CAMERA_SPLASH_START_RADIUS: f32 = 3000.0;
pub const CAMERA_SPLASH_START_FOCUS: Vec3 = Vec3::ZERO;
pub const CAMERA_SPLASH_START_PITCH: f32 = std::f32::consts::FRAC_PI_2;
pub const CAMERA_SPLASH_START_YAW: f32 = -std::f32::consts::PI;

/// Default zoom sensitivity for pan-orbit camera controls
pub const CAMERA_ZOOM_SENSITIVITY: f32 = 0.2;

/// Minimum zoom distance (allows zoom-to-fit to get very close)
pub const CAMERA_ZOOM_LOWER_LIMIT: f32 = 0.001;

/// Font size for debug edge markers
pub const EDGE_MARKER_FONT_SIZE: f32 = 11.0;

/// Radius for edge marker spheres
pub const EDGE_MARKER_SPHERE_RADIUS: f32 = 1.0;
