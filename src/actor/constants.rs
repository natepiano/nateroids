use bevy::prelude::*;

// Spaceship constants
pub const SPACESHIP_ANGULAR_DAMPING: f32 = 0.1;
pub const SPACESHIP_COLLIDER_MARGIN: f32 = 1.0;
pub const SPACESHIP_COLLISION_DAMAGE: f32 = 50.0;
pub const SPACESHIP_HEALTH: f32 = 5000.0;
pub const SPACESHIP_INITIAL_POSITION: Vec3 = Vec3::new(0.0, -20.0, 0.0);
pub const SPACESHIP_LINEAR_DAMPING: f32 = 0.05;
pub const SPACESHIP_MASS: f32 = 10.0;
pub const SPACESHIP_RESTITUTION: f32 = 0.1;
pub const SPACESHIP_SCALE: f32 = 2.0;

// Nateroid constants
pub const NATEROID_ANGULAR_DAMPING: f32 = 0.001;
pub const NATEROID_ANGULAR_VELOCITY: f32 = 4.5;
pub const NATEROID_COLLIDER_MARGIN: f32 = 1.0 / 3.0;
pub const NATEROID_COLLISION_DAMAGE: f32 = 10.0;
pub const NATEROID_DEATH_DURATION_SECS: f32 = 3.0;
pub const NATEROID_DEATH_SHRINK_PCT: f32 = 0.3;
pub const NATEROID_DENSITY_CULLING_THRESHOLD: f32 = 0.01;
pub const NATEROID_HEALTH: f32 = 200.0;
pub const NATEROID_INITIAL_ALPHA: f32 = 0.35;
pub const NATEROID_LINEAR_DAMPING: f32 = 0.001;
pub const NATEROID_LINEAR_VELOCITY: f32 = 35.0;
pub const NATEROID_MASS: f32 = 1.0;
pub const NATEROID_RESTITUTION: f32 = 0.3;
pub const NATEROID_SCALE_UP: f32 = 100.0; // we need bigger nateroids than just donut sized ones
pub const NATEROID_SPAWN_TIMER_SECONDS: f32 = 2.0;
pub const NATEROID_TARGET_ALPHA: f32 = 0.05;

// Missile constants
pub const MISSILE_BASE_VELOCITY: f32 = 85.0;
pub const MISSILE_COLLIDER_MARGIN: f32 = 1.0;
pub const MISSILE_COLLISION_DAMAGE: f32 = 50.0;
pub const MISSILE_FORWARD_DISTANCE_SCALAR: f32 = 7.0;
pub const MISSILE_HEALTH: f32 = 1.0;
pub const MISSILE_MASS: f32 = 0.1;
pub const MISSILE_RESTITUTION: f32 = 0.1;
pub const MISSILE_SCALE: f32 = 2.5;
pub const MISSILE_SPAWN_TIMER_SECONDS: f32 = 1.0 / 20.0;

// Actor physics velocity limits
pub const MAX_MISSILE_ANGULAR_VELOCITY: f32 = 20.0;
pub const MAX_MISSILE_LINEAR_VELOCITY: f32 = 300.0;
pub const MAX_NATEROID_ANGULAR_VELOCITY: f32 = 20.0;
pub const MAX_NATEROID_LINEAR_VELOCITY: f32 = 80.0;
pub const MAX_SPACESHIP_ANGULAR_VELOCITY: f32 = 20.0;
pub const MAX_SPACESHIP_LINEAR_VELOCITY: f32 = 80.0;
