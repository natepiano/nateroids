use std::f32::consts::PI;

use bevy::color::Color;
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::color::palettes::tailwind::YELLOW_800;

// portal colors
pub(super) const PORTAL_COLOR_APPROACHING: Color = Color::Srgba(BLUE_600);
pub(super) const PORTAL_COLOR_EMERGING: Color = Color::Srgba(YELLOW_800);

// portal defaults
/// Default face count for a portal that does not straddle a boundary edge.
pub(super) const DEFAULT_PORTAL_FACE_COUNT: usize = 1;

// portal inspector bounds
pub(super) const PORTAL_DIRECTION_CHANGE_FACTOR_MAX: f32 = PI;
pub(super) const PORTAL_DIRECTION_CHANGE_FACTOR_MIN: f32 = 0.0;
pub(super) const PORTAL_DISTANCE_MAX: f32 = 1.0;
pub(super) const PORTAL_DISTANCE_MIN: f32 = 0.0;
pub(super) const PORTAL_FADEOUT_DURATION_MAX: f32 = 30.0;
pub(super) const PORTAL_FADEOUT_DURATION_MIN: f32 = 1.0;
pub(super) const PORTAL_LINE_JOINTS_MAX: u32 = 40;
pub(super) const PORTAL_LINE_JOINTS_MIN: u32 = 0;
pub(super) const PORTAL_LINE_WIDTH_MAX: f32 = 40.0;
pub(super) const PORTAL_LINE_WIDTH_MIN: f32 = 0.1;
pub(super) const PORTAL_MINIMUM_RADIUS_MAX: f32 = 1.0;
pub(super) const PORTAL_MINIMUM_RADIUS_MIN: f32 = 0.001;
pub(super) const PORTAL_MOVEMENT_SMOOTHING_FACTOR_MAX: f32 = 1.0;
pub(super) const PORTAL_MOVEMENT_SMOOTHING_FACTOR_MIN: f32 = 0.0;
pub(super) const PORTAL_RESOLUTION_MAX: u32 = 256;
pub(super) const PORTAL_RESOLUTION_MIN: u32 = 3;
pub(super) const PORTAL_SCALE_MAX: f32 = 10.0;
pub(super) const PORTAL_SCALE_MIN: f32 = 1.0;
pub(super) const PORTAL_SMALLEST_MAX: f32 = 10.0;
pub(super) const PORTAL_SMALLEST_MIN: f32 = 1.0;
