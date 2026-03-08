//! Constants for the playfield module
//! All magic numbers and configuration values used within playfield/

use bevy::color::Color;
use bevy::math::UVec3;

// =============================================================================
// Boundary Configuration
// =============================================================================

/// Default cell count for the boundary grid (X, Y, Z cells)
pub(super) const BOUNDARY_CELL_COUNT: UVec3 = UVec3::new(3, 2, 1);

/// Scalar multiplier for boundary dimensions
pub(super) const BOUNDARY_SCALAR: f32 = 110.0;

/// Line width for the boundary grid lines
pub(super) const BOUNDARY_GRID_LINE_WIDTH: f32 = 1.5;

/// Line width for the outer boundary box
pub(super) const BOUNDARY_OUTER_LINE_WIDTH: f32 = 4.0;

/// Target alpha value for grid color
pub(super) const BOUNDARY_GRID_ALPHA: f32 = 0.25;

/// Target alpha value for outer boundary color
pub(super) const BOUNDARY_OUTER_ALPHA: f32 = 1.0;

// =============================================================================
// Boundary Position Snapping
// =============================================================================

/// Epsilon for boundary position snapping to prevent false-positive overextension
pub(super) const BOUNDARY_SNAP_EPSILON: f32 = 0.01;

/// Epsilon for portal overextension detection (2x snap epsilon)
pub(super) const BOUNDARY_OVEREXTENSION_EPSILON: f32 = BOUNDARY_SNAP_EPSILON * 2.0;

/// Epsilon tolerance for normal comparison in boundary position detection
pub(super) const BOUNDARY_NORMAL_EPSILON: f32 = 0.001;

// =============================================================================
// Portal Colors
// =============================================================================

/// Color for Deaderoid approaching portals (Red)
pub(super) const DEADEROID_APPROACHING_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);

/// Color for Left/Right face corners on YZ plane (Red)
pub(super) const CORNER_COLOR_LEFT_RIGHT_YZ: Color = Color::srgb(1.0, 0.0, 0.0);

/// Color for Top/Bottom face corners on XZ plane (Green)
pub(super) const CORNER_COLOR_TOP_BOTTOM_XZ: Color = Color::srgb(0.0, 1.0, 0.0);

/// Color for Front/Back face corners on XY plane (Yellow)
pub(super) const CORNER_COLOR_FRONT_BACK_XY: Color = Color::srgb(1.0, 1.0, 0.0);

// =============================================================================
// Portal Configuration
// =============================================================================

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

/// Minimum portal radius before removal
pub(super) const PORTAL_MINIMUM_RADIUS: f32 = 0.1;

/// Smoothing factor for portal position interpolation (0.0 to 1.0)
pub(super) const PORTAL_MOVEMENT_SMOOTHING_FACTOR: f32 = 0.08;

/// Scalar multiplier for portal size relative to actor AABB
pub(super) const PORTAL_SCALAR: f32 = 2.0;

/// Smallest base portal size
pub(super) const PORTAL_SMALLEST: f32 = 5.0;

/// Resolution (segments) for rendering portal circles
pub(super) const PORTAL_RESOLUTION: u32 = 128;

/// Minimum radius fraction for approaching portals (0.5 = half of `max_radius`)
pub(super) const PORTAL_MIN_RADIUS_FRACTION: f32 = 0.5;

/// Multiplier for boundary diagonal to detect physics burst events
pub(super) const PORTAL_PHYSICS_BURST_MULTIPLIER: f32 = 2.0;

// =============================================================================
// Plane Configuration
// =============================================================================

/// Default index of refraction for boundary planes
pub(super) const PLANE_IOR: f32 = 1.5;

/// Default perceptual roughness for plane material
pub(super) const PLANE_PERCEPTUAL_ROUGHNESS: f32 = 0.5;

/// Default reflectance for plane material
pub(super) const PLANE_REFLECTANCE: f32 = 0.5;

/// Default thickness for boundary planes
pub(super) const PLANE_THICKNESS: f32 = 0.001;

/// Rotation angle for plane orientation (no rotation)
pub(super) const PLANE_ROTATION_ANGLE: f32 = 0.0;
