use bevy::prelude::Color;
use bevy_kana::Position;

// Camera bloom
pub(super) const CAMERA_BLOOM_HIGH_PASS_FREQUENCY: f32 = 0.5;
pub(super) const CAMERA_BLOOM_INTENSITY: f32 = 0.5;
pub(super) const CAMERA_BLOOM_LOW_FREQUENCY_BOOST: f32 = 0.5;

// Camera smoothness
/// Default orbit smoothness (0.02 = 2% per frame)
pub(super) const CAMERA_ORBIT_SMOOTHNESS: f32 = 0.02;
/// Default pan smoothness (0.02 = 2% per frame)
pub(super) const CAMERA_PAN_SMOOTHNESS: f32 = 0.02;
/// Default zoom smoothness (0.10 = 10% per frame)
pub(super) const CAMERA_ZOOM_SMOOTHNESS: f32 = 0.10;

// Camera splash
/// Initial camera distance for splash screen animation.
/// Camera spawns at this distance to appear stationary during the opening text.
pub(super) const CAMERA_SPLASH_START_FOCUS: Position = Position::new(0.0, 0.0, 0.0);
pub(super) const CAMERA_SPLASH_START_PITCH: f32 = std::f32::consts::FRAC_PI_2;
pub(super) const CAMERA_SPLASH_START_RADIUS: f32 = 3000.0;
pub(super) const CAMERA_SPLASH_START_YAW: f32 = -std::f32::consts::PI;

// Camera trackpad
pub(super) const CAMERA_TRACKPAD_SENSITIVITY: f32 = 0.8;

// Camera zoom
/// Minimum zoom distance (allows zoom-to-fit to get very close)
pub(super) const CAMERA_ZOOM_LOWER_LIMIT: f32 = 0.001;
pub(super) const CAMERA_ZOOM_SENSITIVITY: f32 = 0.2;

// Edge markers
pub(super) const EDGE_MARKER_FONT_SIZE: f32 = 11.0;
pub(super) const EDGE_MARKER_SPHERE_RADIUS: f32 = 1.0;

// Focus gizmo
pub(super) const FOCUS_GIZMO_DEFAULT_CAMERA_RADIUS: f32 = 100.0;
pub(super) const FOCUS_GIZMO_COLOR: Color = Color::Srgba(bevy::color::Srgba {
    red:   1.0,
    green: 0.0,
    blue:  0.0,
    alpha: 1.0,
});
pub(super) const FOCUS_GIZMO_LINE_WIDTH: f32 = 2.0;

// Home animation
pub(super) const HOME_ANIMATION_DURATION_MS: u64 = 1200;

// Lighting
pub(super) const AMBIENT_LIGHT_BRIGHTNESS: f32 = 100.0;
pub(super) const SHADOW_DEPTH_BIAS: f32 = 0.02;
pub(super) const SHADOW_NORMAL_BIAS: f32 = 0.6;
pub(super) const CASCADE_SHADOW_FIRST_FAR_BOUND: f32 = 50.0;
pub(super) const CASCADE_SHADOW_MAX_DISTANCE: f32 = 1500.0;
pub(super) const CASCADE_SHADOW_NUM_CASCADES: usize = 4;
pub(super) const CASCADE_SHADOW_OVERLAP_PROPORTION: f32 = 0.3;
pub(super) const DIRECTIONAL_LIGHT_ILLUMINANCE: f32 = 1700.0;
pub(super) const ENVIRONMENT_MAP_INTENSITY: f32 = 25_000.0;

// Selection outline
pub(super) const SELECTION_OUTLINE_COLOR: Color = Color::Srgba(bevy::color::Srgba {
    red:   0.0,
    green: 0.24,
    blue:  1.0,
    alpha: 1.0,
});
/// Outline intensity for selected entities (values > 1.0 create glow with bloom)
pub(super) const SELECTION_OUTLINE_INTENSITY: f32 = 4.0;
pub(super) const SELECTION_OUTLINE_WIDTH: f32 = 5.0;

// Star brightness
/// Minimum star brightness as fraction of range (0.2 = 20%)
pub(super) const STAR_MINIMUM_BRIGHTNESS_FRACTION: f32 = 0.2;

// Star field
pub(super) const STAR_BATCH_SIZE_REPLACE: usize = 10;
pub(super) const STAR_COLOR_RANGE_MAX: f32 = 30.0;
pub(super) const STAR_COLOR_RANGE_MIN: f32 = -30.0;
pub(super) const STAR_COLOR_WHITE_PROBABILITY: f32 = 0.85;
pub(super) const STAR_COLOR_WHITE_START_RATIO: f32 = 0.7;
pub(super) const STAR_COUNT: usize = 1000;
pub(super) const STAR_DURATION_REPLACE_TIMER: f32 = 1.0;
pub(super) const STAR_FIELD_DIAMETER: std::ops::Range<f32> = 200.0..400.0;
pub(super) const STAR_RADIUS: std::ops::Range<f32> = 0.3..2.5;

// Star rotation
/// Minimum rotation cycle in minutes (1 second = 0.01667 minutes)
pub(super) const STAR_ROTATION_CYCLE_MINIMUM_MINUTES: f32 = 0.01667;
pub(super) const STAR_ROTATION_CYCLE_MINUTES: f32 = 15.0;

// Star twinkling
pub(super) const STAR_TWINKLE_CHOOSE_MULTIPLE_COUNT: usize = 2;
pub(super) const STAR_TWINKLE_DURATION_MAX: f32 = 2.0;
pub(super) const STAR_TWINKLE_DURATION_MIN: f32 = 0.5;
pub(super) const STAR_TWINKLE_INTENSITY_MAX: f32 = 20.0;
pub(super) const STAR_TWINKLE_INTENSITY_MIN: f32 = 10.0;
pub(super) const STAR_TWINKLING_DELAY: f32 = 0.5;

// Zoom
pub(super) const ZOOM_CONVERGENCE_RATE: f32 = 0.30;
/// Default margin for zoom-to-fit operations (0.1 = 10% margin on each side)
pub const ZOOM_MARGIN: f32 = 0.1;
pub(super) const ZOOM_MARGIN_TOLERANCE: f32 = 0.00001;
pub(super) const ZOOM_MAX_ITERATIONS: usize = 200;
pub(super) const ZOOM_SETTINGS_MARGIN: f32 = 0.1;
pub(super) const ZOOM_TO_FIT_DURATION_MS: u64 = 500;
