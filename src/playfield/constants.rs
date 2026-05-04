//! Constants for the playfield module
//! All magic numbers and configuration values used within playfield/

use bevy::color::Color;
use bevy::math::UVec3;

// boundary configuration

/// Default cell count for the boundary grid (X, Y, Z cells)
pub(super) const BOUNDARY_CELL_COUNT: UVec3 = UVec3::new(3, 2, 1);

/// Target alpha value for grid color
pub(super) const BOUNDARY_GRID_ALPHA: f32 = 0.0;

/// Line width for the boundary grid lines
pub(super) const BOUNDARY_GRID_LINE_WIDTH: f32 = 1.5;

/// Target alpha value for outer boundary color
pub(super) const BOUNDARY_OUTER_ALPHA: f32 = 1.0;

/// Line width for the outer boundary box
pub(super) const BOUNDARY_OUTER_LINE_WIDTH: f32 = 4.0;

/// Scalar multiplier for boundary dimensions
pub(super) const BOUNDARY_SCALAR: f32 = 110.0;

/// Duration in seconds for the grid flash animation when cell count changes
pub(super) const GRID_FLASH_DURATION: f32 = 2.0;

// boundary fade-in logging
/// Approximate frame duration for fade-in logging throttle (seconds)
pub(super) const FADE_LOG_FRAME_EPSILON: f32 = 0.016;
/// Interval between boundary fade-in log messages (seconds)
pub(super) const FADE_LOG_INTERVAL_SECS: f32 = 0.5;

// boundary position snapping

/// Epsilon for portal overextension detection (2x snap epsilon)
pub(super) const BOUNDARY_OVEREXTENSION_EPSILON: f32 = BOUNDARY_SNAP_EPSILON * 2.0;

/// Epsilon for boundary position snapping to prevent false-positive overextension
pub(super) const BOUNDARY_SNAP_EPSILON: f32 = 0.01;

// circle–line-segment intersection
/// Epsilon for deduplicating circle–line-segment intersection points
pub(super) const INTERSECTION_DEDUP_EPSILON: f32 = 1e-6;

// portal colors

/// Color for Front/Back face corners on XY plane (Yellow)
pub(super) const CORNER_COLOR_FRONT_BACK_XY: Color = Color::srgb(1.0, 1.0, 0.0);

/// Color for Left/Right face corners on YZ plane (Red)
pub(super) const CORNER_COLOR_LEFT_RIGHT_YZ: Color = Color::srgb(1.0, 0.0, 0.0);

/// Color for Top/Bottom face corners on XZ plane (Green)
pub(super) const CORNER_COLOR_TOP_BOTTOM_XZ: Color = Color::srgb(0.0, 1.0, 0.0);

/// Color for `Deaderoid` approaching portals (Red)
pub(super) const DEADEROID_APPROACHING_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);

// portal configuration

/// Threshold for considering normals "similar" during portal movement smoothing
pub(super) const PORTAL_DIRECTION_CHANGE_FACTOR: f32 = 0.75;

/// Distance from boundary to start showing approaching portal (as fraction of boundary size)
pub(super) const PORTAL_DISTANCE_APPROACH: f32 = 0.5;

/// Distance from boundary to start shrinking portal (as fraction of boundary size)
pub(super) const PORTAL_DISTANCE_SHRINK: f32 = 0.25;

/// Duration in seconds for portal fade-out animation
pub(super) const PORTAL_FADEOUT_DURATION: f32 = 14.0;

/// Number of joints for portal line rendering
pub(super) const PORTAL_LINE_JOINTS: u32 = 4;

/// Line width for portal rendering
pub(super) const PORTAL_LINE_WIDTH: f32 = 2.0;

/// Minimum radius fraction for approaching portals (0.5 = half of `max_radius`)
pub(super) const PORTAL_MIN_RADIUS_FRACTION: f32 = 0.5;

/// Minimum portal radius before removal
pub(super) const PORTAL_MINIMUM_RADIUS: f32 = 0.1;

/// Smoothing factor for portal position interpolation (0.0 to 1.0)
pub(super) const PORTAL_MOVEMENT_SMOOTHING_FACTOR: f32 = 0.08;

/// Multiplier for boundary diagonal to detect physics burst events
pub(super) const PORTAL_PHYSICS_BURST_MULTIPLIER: f32 = 2.0;

/// Resolution (segments) for rendering portal circles
pub(super) const PORTAL_RESOLUTION: u32 = 128;

/// Scalar multiplier for portal size relative to actor AABB
pub(super) const PORTAL_SCALAR: f32 = 2.0;

/// Smallest base portal size
pub(super) const PORTAL_SMALLEST: f32 = 5.0;
