//! Constants for the playfield module
//! All magic numbers and configuration values used within playfield/

use bevy::color::Color;
use bevy::math::UVec3;
use bevy::math::Vec2;

// =============================================================================
// Boundary Configuration
// =============================================================================

/// Default cell count for the boundary grid (X, Y, Z cells)
pub const BOUNDARY_CELL_COUNT: UVec3 = UVec3::new(3, 2, 1);

/// Scalar multiplier for boundary dimensions
pub const BOUNDARY_SCALAR: f32 = 110.0;

/// Line width for the boundary grid lines
pub const BOUNDARY_GRID_LINE_WIDTH: f32 = 1.5;

/// Line width for the outer boundary box
pub const BOUNDARY_OUTER_LINE_WIDTH: f32 = 4.0;

/// Target alpha value for grid color
pub const BOUNDARY_GRID_ALPHA: f32 = 0.25;

/// Target alpha value for outer boundary color
pub const BOUNDARY_OUTER_ALPHA: f32 = 1.0;

/// Default viewport size used when camera viewport size is unavailable
pub const BOUNDARY_DEFAULT_VIEWPORT_SIZE: Vec2 = Vec2::new(1920.0, 1080.0);

/// Multiplier for total line width when calculating outer scale
pub const BOUNDARY_LINE_WIDTH_MULTIPLIER: f32 = 0.1;

// =============================================================================
// Boundary Position Snapping
// =============================================================================

/// Epsilon for boundary position snapping to prevent false-positive overextension
pub const BOUNDARY_SNAP_EPSILON: f32 = 0.01;

/// Epsilon for portal overextension detection (2x snap epsilon)
pub const BOUNDARY_OVEREXTENSION_EPSILON: f32 = BOUNDARY_SNAP_EPSILON * 2.0;

/// Epsilon tolerance for normal comparison in boundary position detection
pub const BOUNDARY_NORMAL_EPSILON: f32 = 0.001;

// =============================================================================
// Portal Colors
// =============================================================================

/// Color for Deaderoid approaching portals (Red)
pub const DEADEROID_APPROACHING_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);

/// Color for Left/Right face corners on YZ plane (Red)
pub const CORNER_COLOR_LEFT_RIGHT_YZ: Color = Color::srgb(1.0, 0.0, 0.0);

/// Color for Top/Bottom face corners on XZ plane (Green)
pub const CORNER_COLOR_TOP_BOTTOM_XZ: Color = Color::srgb(0.0, 1.0, 0.0);

/// Color for Front/Back face corners on XY plane (Yellow)
pub const CORNER_COLOR_FRONT_BACK_XY: Color = Color::srgb(1.0, 1.0, 0.0);

// =============================================================================
// Portal Configuration
// =============================================================================

/// Threshold for considering normals "similar" during portal movement smoothing
pub const PORTAL_DIRECTION_CHANGE_FACTOR: f32 = 0.75;

/// Distance from boundary to start showing approaching portal (as fraction of boundary size)
pub const PORTAL_DISTANCE_APPROACH: f32 = 0.5;

/// Distance from boundary to start shrinking portal (as fraction of boundary size)
pub const PORTAL_DISTANCE_SHRINK: f32 = 0.25;

/// Duration in seconds for portal fade-out animation
pub const PORTAL_FADEOUT_DURATION: f32 = 14.0;

/// Number of joints for portal line rendering
pub const PORTAL_LINE_JOINTS: u32 = 4;

/// Line width for portal rendering
pub const PORTAL_LINE_WIDTH: f32 = 2.0;

/// Minimum portal radius before removal
pub const PORTAL_MINIMUM_RADIUS: f32 = 0.1;

/// Smoothing factor for portal position interpolation (0.0 to 1.0)
pub const PORTAL_MOVEMENT_SMOOTHING_FACTOR: f32 = 0.08;

/// Scalar multiplier for portal size relative to actor AABB
pub const PORTAL_SCALAR: f32 = 2.0;

/// Smallest base portal size
pub const PORTAL_SMALLEST: f32 = 5.0;

/// Resolution (segments) for rendering portal circles
pub const PORTAL_RESOLUTION: u32 = 128;

/// Minimum radius fraction for approaching portals (0.5 = half of max_radius)
pub const PORTAL_MIN_RADIUS_FRACTION: f32 = 0.5;

/// Multiplier for boundary diagonal to detect physics burst events
pub const PORTAL_PHYSICS_BURST_MULTIPLIER: f32 = 2.0;

// =============================================================================
// Plane Configuration
// =============================================================================

/// Default index of refraction for boundary planes
pub const PLANE_IOR: f32 = 1.5;

/// Default perceptual roughness for plane material
pub const PLANE_PERCEPTUAL_ROUGHNESS: f32 = 0.5;

/// Default reflectance for plane material
pub const PLANE_REFLECTANCE: f32 = 0.5;

/// Default thickness for boundary planes
pub const PLANE_THICKNESS: f32 = 0.001;

/// Rotation angle for plane orientation (no rotation)
pub const PLANE_ROTATION_ANGLE: f32 = 0.0;

// =============================================================================
// Screen Boundary Configuration
// =============================================================================

/// Default line width for screen boundary gizmo
pub const SCREEN_BOUNDARY_LINE_WIDTH: f32 = 1.0;

/// Text offset from screen edge for margin labels
pub const SCREEN_BOUNDARY_TEXT_OFFSET: f32 = 0.01;

/// Font size for margin labels
pub const SCREEN_BOUNDARY_FONT_SIZE: f32 = 11.0;

/// Offset distance for world position label placement
pub const SCREEN_BOUNDARY_WORLD_POS_OFFSET: f32 = 1.0;
