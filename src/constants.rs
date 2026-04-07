//! Top-level constants for standalone modules (physics, splash, despawn)

// Despawn constants
pub(crate) const DEATH_VELOCITY_EPSILON: f32 = 0.001;

// Physics constants
/// Minimum interval between physics-stress log messages
pub(crate) const PHYSICS_WARN_THROTTLE_INTERVAL_SECS: f64 = 1.0;

pub(crate) const MIN_NATEROIDS_FOR_MONITORING: usize = 50;
pub(crate) const PHYSICS_SUBSTEP_COUNT: u32 = 15;
pub(crate) const STRESS_ENTER_FPS_THRESHOLD: f64 = 35.0;
pub(crate) const STRESS_EXIT_FPS_THRESHOLD: f64 = 45.0;
pub(crate) const STRESS_VELOCITY_THRESHOLD: f32 = 200.0;

// Splash constants
pub(crate) const SPLASH_FAST_SPIN_COUNT: usize = 5;
pub(crate) const SPLASH_FAST_SPIN_DURATION_MS: u64 = 25;
pub(crate) const SPLASH_HOLD_DURATION_MS: u64 = 2500;
pub(crate) const SPLASH_LAND_HOME_DURATION_MS: u64 = 1200;
pub(crate) const SPLASH_SKIP_HINT_ALPHA: f32 = 0.8;
pub(crate) const SPLASH_SKIP_HINT_BOTTOM_OFFSET: f32 = 24.0;
pub(crate) const SPLASH_SKIP_HINT_FONT_SIZE: f32 = 20.0;
pub(crate) const SPLASH_SLOWDOWN_DURATIONS_MS: &[u64] = &[50, 100, 150, 200];
pub(crate) const SPLASH_SPIN_DURATIONS_MS: &[u64] = &[500, 400, 300, 200, 100, 50, 25];
pub(crate) const SPLASH_TEXT_GROWTH_RATE: f32 = 1.2;
pub(crate) const SPLASH_TEXT_TIME: f32 = 2.;
pub(crate) const SPLASH_ZOOM_DURATION_MS: u64 = 1000;
