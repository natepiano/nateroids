use bevy::prelude::Color;
use bevy_kana::Position;

/// Initial camera distance for splash screen animation.
/// Camera spawns at this distance to appear stationary during the opening text.
pub(super) const CAMERA_SPLASH_START_RADIUS: f32 = 3000.0;
pub(super) const CAMERA_SPLASH_START_FOCUS: Position = Position::new(0.0, 0.0, 0.0);
pub(super) const CAMERA_SPLASH_START_PITCH: f32 = std::f32::consts::FRAC_PI_2;
pub(super) const CAMERA_SPLASH_START_YAW: f32 = -std::f32::consts::PI;

/// Minimum rotation cycle in minutes (1 second = 0.01667 minutes)
pub(super) const STAR_ROTATION_CYCLE_MINIMUM_MINUTES: f32 = 0.01667;

/// Default margin for zoom-to-fit operations (0.1 = 10% margin on each side)
pub const ZOOM_MARGIN: f32 = 0.1;

/// Duration in milliseconds for zoom-to-fit animation
pub(super) const ZOOM_TO_FIT_DURATION_MS: u64 = 500;

/// Duration in milliseconds for home camera animation
pub(super) const HOME_ANIMATION_DURATION_MS: u64 = 1200;

/// Default zoom sensitivity for orbit camera controls
pub(super) const CAMERA_ZOOM_SENSITIVITY: f32 = 0.2;

/// Minimum zoom distance (allows zoom-to-fit to get very close)
pub(super) const CAMERA_ZOOM_LOWER_LIMIT: f32 = 0.001;

/// Font size for debug edge markers
pub(super) const EDGE_MARKER_FONT_SIZE: f32 = 11.0;

/// Radius for edge marker spheres
pub(super) const EDGE_MARKER_SPHERE_RADIUS: f32 = 1.0;

/// Outline width in screen-space pixels for selected entities
pub(super) const SELECTION_OUTLINE_WIDTH: f32 = 5.0;

/// Outline intensity for selected entities (values > 1.0 create glow with bloom)
pub(super) const SELECTION_OUTLINE_INTENSITY: f32 = 4.0;

/// Outline color for selected entities
pub(super) const SELECTION_OUTLINE_COLOR: Color = Color::Srgba(bevy::color::Srgba {
    red:   0.0,
    green: 0.24,
    blue:  1.0,
    alpha: 1.0,
});
