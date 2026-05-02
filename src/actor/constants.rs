use avian3d::prelude::LockedAxes;
use bevy::prelude::Vec3;
use bevy_kana::Position;

// Aabb gizmo inspector bounds
pub(super) const AABB_LINE_WIDTH_MAX: f32 = 40.0;
pub(super) const AABB_LINE_WIDTH_MIN: f32 = 0.1;

// Actor health
/// Health value that triggers instant death
pub(super) const INSTANT_DEATH_HEALTH: f32 = -1.0;

// Actor inspector bounds
pub(super) const ACTOR_COLLIDER_MARGIN_MAX: f32 = 3.0;
pub(super) const ACTOR_COLLIDER_MARGIN_MIN: f32 = 0.1;
pub(super) const ACTOR_DAMPING_MAX: f32 = 1.0;
pub(super) const ACTOR_DAMPING_MIN: f32 = 0.0;
pub(super) const ACTOR_MASS_MAX: f32 = 20.0;
pub(super) const ACTOR_MASS_MIN: f32 = 0.0;
pub(super) const ACTOR_MAX_VELOCITY_MAX: f32 = 500.0;
pub(super) const ACTOR_MAX_VELOCITY_MIN: f32 = 0.0;
pub(super) const ACTOR_RESTITUTION_MAX: f32 = 1.0;
pub(super) const ACTOR_RESTITUTION_MIN: f32 = 0.1;

// Actor physics velocity limits
pub(super) const MAX_MISSILE_ANGULAR_VELOCITY: f32 = 20.0;
pub(super) const MAX_MISSILE_LINEAR_VELOCITY: f32 = 300.0;
pub(super) const MAX_NATEROID_ANGULAR_VELOCITY: f32 = 20.0;
pub(super) const MAX_NATEROID_LINEAR_VELOCITY: f32 = 80.0;
pub(super) const MAX_SPACESHIP_ANGULAR_VELOCITY: f32 = 20.0;
pub(super) const MAX_SPACESHIP_LINEAR_VELOCITY: f32 = 80.0;

// Actor test constants
#[cfg(test)]
pub(super) const TELEPORT_FLOAT_EPSILON: f32 = 0.000_001;

// Death effect constants
pub(super) const DEATH_EFFECT_DURATION_SECS: f32 = 3.0;
pub(super) const DEATH_EFFECT_EXPANDING_RING_START_SCALE: f32 = 0.2;
pub(super) const DEATH_EFFECT_RADIUS_MARGIN: f32 = 20.0;

// Death effect line constants (similar to thruster but around a ring)
pub(super) const DEATH_EFFECT_LINE_COUNT: usize = 365;
pub(super) const DEATH_EFFECT_LINE_LENGTH_BASE: f32 = 5.0;
pub(super) const DEATH_EFFECT_LINE_LENGTH_VARIANCE: f32 = 2.0;

// Flame gizmo shared constants
pub(super) const FLAME_COLOR_FLICKER_SPEED: f32 = 12.0;
pub(super) const FLAME_GIZMO_LINE_WIDTH: f32 = 2.5;
pub(super) const FLAME_LENGTH_FLICKER_PHASE_MULTIPLIER: f32 = 2.3;
pub(super) const FLAME_LENGTH_FLICKER_SPEED: f32 = 15.0;
pub(super) const FLAME_PHASE_SPREAD: f32 = 1.7;
pub(super) const FLAME_VIBRATION_AMPLITUDE: f32 = 0.4;
pub(super) const FLAME_VIBRATION_SPEED: f32 = 25.0;

// Missile constants
pub(super) const MISSILE_BASE_VELOCITY: f32 = 85.0;
pub(super) const MISSILE_COLLIDER_MARGIN: f32 = 1.0;
pub(super) const MISSILE_COLLISION_DAMAGE: f32 = 50.0;
pub(super) const MISSILE_FORWARD_DISTANCE_SCALAR: f32 = 7.0;
pub(super) const MISSILE_HEALTH: f32 = 1.0;
pub(super) const MISSILE_MASS: f32 = 0.1;
pub(super) const MISSILE_RESTITUTION: f32 = 0.1;
pub(super) const MISSILE_SCALE: f32 = 2.5;
pub(super) const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;

// Nateroid constants
pub(super) const NATEROID_ANGULAR_DAMPING: f32 = 0.001;
pub(super) const NATEROID_ANGULAR_VELOCITY: f32 = 4.5;
pub(super) const NATEROID_COLLIDER_MARGIN: f32 = 1.0 / 3.0;
pub(super) const NATEROID_COLLISION_DAMAGE: f32 = 10.0;
/// Alpha decrement per death animation material level
pub const NATEROID_DEATH_ALPHA_STEP: f32 = 0.01;
pub(super) const NATEROID_DEATH_DURATION_SECS: f32 = 3.0;
pub(super) const NATEROID_DEATH_SHRINK_PERCENTAGE: f32 = 0.3;
pub(super) const NATEROID_DENSITY_CULLING_THRESHOLD: f32 = 0.01;
pub(super) const NATEROID_HEALTH: f32 = 200.0;
pub(super) const NATEROID_INITIAL_ALPHA: f32 = 0.35;
pub(super) const NATEROID_LINEAR_DAMPING: f32 = 0.001;
pub(super) const NATEROID_LINEAR_VELOCITY: f32 = 35.0;
pub(super) const NATEROID_MASS: f32 = 1.0;
pub(super) const NATEROID_RESTITUTION: f32 = 0.3;
pub(super) const NATEROID_SCALE_UP: f32 = 100.0; // we need bigger nateroids than just donut sized ones
/// Number of recent spawn attempts to keep for rate tracking.
pub(super) const NATEROID_SPAWN_HISTORY_LEN: usize = 50;
/// Maximum spawn placement attempts before giving up
pub(super) const NATEROID_SPAWN_MAX_ATTEMPTS: u32 = 20;
pub(super) const NATEROID_SPAWN_TIMER_SECONDS: f32 = 2.0;
pub(super) const NATEROID_TARGET_ALPHA: f32 = 0.05;
/// Minimum interval between spawn-rate warning log messages
pub(super) const NATEROID_WARN_THROTTLE_INTERVAL_SECS: f32 = 1.0;

// Shared actor configuration
/// `Spaceship` model orientation correction: rotates the model so nose points +Y
pub(super) const GLTF_ROTATION_X: f32 = std::f32::consts::FRAC_PI_2; // +90°
pub(super) const LOCKED_AXES_2D: LockedAxes = LockedAxes::new().lock_translation_z();
pub(super) const LOCKED_AXES_SPACESHIP: LockedAxes = LockedAxes::new()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_translation_z();
/// Half the size of the boundary and only in the x,y plane
pub(super) const SPAWN_WINDOW: Vec3 = Vec3::new(0.5, 0.5, 0.0);

// Spaceship constants
pub(super) const SPACESHIP_ANGULAR_DAMPING: f32 = 0.1;
pub(super) const SPACESHIP_COLLIDER_MARGIN: f32 = 1.0;
pub(super) const SPACESHIP_COLLISION_DAMAGE: f32 = 50.0;
pub(super) const SPACESHIP_HEALTH: f32 = 5000.0;
pub(super) const SPACESHIP_INITIAL_POSITION: Position = Position::new(0.0, -20.0, 0.0);
pub(super) const SPACESHIP_LINEAR_DAMPING: f32 = 0.05;
pub(super) const SPACESHIP_MASS: f32 = 10.0;
pub(super) const SPACESHIP_RESTITUTION: f32 = 0.1;
pub(super) const SPACESHIP_SCALE: f32 = 2.0;

// Spaceship control constants
pub(super) const SPACESHIP_ACCELERATION: f32 = 60.0;
pub(super) const SPACESHIP_MAX_SPEED: f32 = 80.0;
pub(super) const SPACESHIP_ROTATION_SPEED: f32 = 5.0;

// Spaceship control inspector bounds
pub(super) const SPACESHIP_ACCELERATION_MAX: f32 = 300.0;
pub(super) const SPACESHIP_ACCELERATION_MIN: f32 = 30.0;
pub(super) const SPACESHIP_MAX_SPEED_MAX: f32 = 300.0;
pub(super) const SPACESHIP_MAX_SPEED_MIN: f32 = 50.0;
pub(super) const SPACESHIP_ROTATION_SPEED_MAX: f32 = 10.0;
pub(super) const SPACESHIP_ROTATION_SPEED_MIN: f32 = 1.0;

// Spaceship rotation enforcement
/// Forward vector epsilon for safe normalization
pub(super) const SPACESHIP_FORWARD_EPSILON: f32 = 0.0001;
/// Tilt correction threshold (~5 degrees) — only correct if tilted beyond this
pub(super) const SPACESHIP_TILT_THRESHOLD: f32 = 0.087;

// Thrust gizmo constants
pub(super) const THRUSTER_COLOR_FLICKER_INTENSITY: f32 = 0.4;
pub(super) const THRUSTER_COLOR_ZONE_SIZE: f32 = 1.0 / 3.0;
pub(super) const THRUSTER_CONE_HALF_ANGLE: f32 = 0.25;
pub(super) const THRUSTER_LINE_COUNT: usize = 6;
pub(super) const THRUSTER_LINE_LENGTH_BASE: f32 = 12.0;
pub(super) const THRUSTER_LINE_LENGTH_VARIANCE: f32 = 4.0;
pub(super) const THRUSTER_LINE_OFFSET: f32 = 3.0;
/// Vertical phase offset is 70% of lateral, making adjacent lines more in-sync vertically.
pub(super) const THRUSTER_VIBRATION_VERTICAL_PHASE_MULTIPLIER: f32 = 0.7;
/// Vertical vibration runs 30% faster than lateral, creating non-repeating patterns.
pub(super) const THRUSTER_VIBRATION_VERTICAL_SPEED_MULTIPLIER: f32 = 1.3;
