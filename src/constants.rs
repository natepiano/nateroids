//! Top-level constants for standalone modules.

use bevy::prelude::Transform;
use bevy::prelude::Vec3;
use bevy_kana::Displacement;
use bevy_kana::Position;

use crate::orientation::DimensionMode;
use crate::orientation::OrientationSettings;
use crate::switches::Switch;

// despawn constants
pub(crate) const DEATH_VELOCITY_EPSILON: f32 = 0.001;

// orientation constants
pub(crate) const CAMERA_ORIENTATION_DEFAULT_SETTINGS: OrientationSettings = OrientationSettings {
    dimension_mode:   DimensionMode::TwoD,
    axis_mundi:       Vec3::ZERO,
    axis_orbis:       Vec3::ZERO,
    axis_profundus:   Vec3::ZERO,
    locus:            Transform::IDENTITY,
    nexus:            Position::new(0.0, 0.0, 0.0),
    spaceship_offset: Displacement::new(0.0, 5.0, -10.0),
};

// physics constants
pub(crate) const MIN_NATEROIDS_FOR_MONITORING: usize = 50;
pub(crate) const PHYSICS_SUBSTEP_COUNT: u32 = 15;
/// Minimum interval between physics-stress log messages
pub(crate) const PHYSICS_WARN_THROTTLE_INTERVAL_SECS: f64 = 1.0;
pub(crate) const STRESS_ENTER_FPS_THRESHOLD: f64 = 35.0;
pub(crate) const STRESS_EXIT_FPS_THRESHOLD: f64 = 45.0;
pub(crate) const STRESS_VELOCITY_THRESHOLD: f32 = 200.0;

// splash constants
pub(crate) const SPLASH_FAST_SPIN_COUNT: usize = 5;
pub(crate) const SPLASH_FAST_SPIN_DURATION_MS: u64 = 25;
pub(crate) const SPLASH_HOLD_DURATION_MS: u64 = 2500;
pub(crate) const SPLASH_INITIAL_FONT_SIZE: f32 = 1.0;
pub(crate) const SPLASH_LAND_HOME_DURATION_MS: u64 = 1200;
pub(crate) const SPLASH_SKIP_HINT_ALPHA: f32 = 0.8;
pub(crate) const SPLASH_SKIP_HINT_BOTTOM_OFFSET: f32 = 24.0;
pub(crate) const SPLASH_SKIP_HINT_FONT_SIZE: f32 = 20.0;
pub(crate) const SPLASH_SLOWDOWN_DURATIONS_MS: &[u64] = &[50, 100, 150, 200];
pub(crate) const SPLASH_SPIN_DURATIONS_MS: &[u64] = &[500, 400, 300, 200, 100, 50, 25];
pub(crate) const SPLASH_TEXT_GROWTH_RATE: f32 = 1.2;
pub(crate) const SPLASH_TEXT_TIME: f32 = 2.;
pub(crate) const SPLASH_ZOOM_DURATION_MS: u64 = 1000;

// switches constants
pub(crate) const INSPECTOR_SWITCHES: [Switch; 13] = [
    Switch::InspectAabb,
    Switch::InspectBoundary,
    Switch::InspectCamera,
    Switch::InspectFocus,
    Switch::InspectLights,
    Switch::InspectMissile,
    Switch::InspectNateroid,
    Switch::InspectOutline,
    Switch::InspectPortals,
    Switch::InspectSpaceship,
    Switch::InspectSpaceshipControl,
    Switch::InspectStar,
    Switch::InspectZoom,
];
