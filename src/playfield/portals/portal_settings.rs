use bevy::color::Color;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;

use crate::camera::RenderLayer;
use crate::playfield::constants::PORTAL_DIRECTION_CHANGE_FACTOR;
use crate::playfield::constants::PORTAL_DISTANCE_APPROACH;
use crate::playfield::constants::PORTAL_DISTANCE_SHRINK;
use crate::playfield::constants::PORTAL_FADEOUT_DURATION;
use crate::playfield::constants::PORTAL_LINE_JOINTS;
use crate::playfield::constants::PORTAL_LINE_WIDTH;
use crate::playfield::constants::PORTAL_MINIMUM_RADIUS;
use crate::playfield::constants::PORTAL_MOVEMENT_SMOOTHING_FACTOR;
use crate::playfield::constants::PORTAL_RESOLUTION;
use crate::playfield::constants::PORTAL_SCALAR;
use crate::playfield::constants::PORTAL_SMALLEST;

#[derive(Debug, Default, Reflect, GizmoConfigGroup)]
pub struct PortalGizmo {}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct PortalSettings {
    pub(super) color_approaching:         Color,
    pub(super) color_emerging:            Color,
    #[inspector(min = 0.0, max = std::f32::consts::PI, display = NumberDisplay::Slider)]
    pub(super) direction_change_factor:   f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub(super) distance_approach:         f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub(super) distance_shrink:           f32,
    #[inspector(min = 1.0, max = 30.0, display = NumberDisplay::Slider)]
    pub(super) fadeout_duration:          f32,
    #[inspector(min = 0, max = 40, display = NumberDisplay::Slider)]
    pub(super) line_joints:               u32,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub(super) line_width:                f32,
    #[inspector(min = 0.001, max = 1.0, display = NumberDisplay::Slider)]
    pub(super) minimum_radius:            f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub(super) movement_smoothing_factor: f32,
    #[inspector(min = 1., max = 10., display = NumberDisplay::Slider)]
    pub(super) scalar:                    f32,
    #[inspector(min = 1., max = 10., display = NumberDisplay::Slider)]
    pub(super) smallest:                  f32,
    #[inspector(min = 3, max = 256, display = NumberDisplay::Slider)]
    pub(super) resolution:                u32,
}

impl Default for PortalSettings {
    fn default() -> Self {
        Self {
            color_approaching:         Color::from(tailwind::BLUE_600),
            color_emerging:            Color::from(tailwind::YELLOW_800),
            direction_change_factor:   PORTAL_DIRECTION_CHANGE_FACTOR,
            distance_approach:         PORTAL_DISTANCE_APPROACH,
            distance_shrink:           PORTAL_DISTANCE_SHRINK,
            fadeout_duration:          PORTAL_FADEOUT_DURATION,
            line_joints:               PORTAL_LINE_JOINTS,
            line_width:                PORTAL_LINE_WIDTH,
            minimum_radius:            PORTAL_MINIMUM_RADIUS,
            movement_smoothing_factor: PORTAL_MOVEMENT_SMOOTHING_FACTOR,
            scalar:                    PORTAL_SCALAR,
            smallest:                  PORTAL_SMALLEST,
            resolution:                PORTAL_RESOLUTION,
        }
    }
}

pub(super) fn apply_portal_settings(
    mut config_store: ResMut<GizmoConfigStore>,
    portal_settings: Res<PortalSettings>,
) {
    let (config, _) = config_store.config_mut::<PortalGizmo>();
    config.line.width = portal_settings.line_width;
    config.line.joints = GizmoLineJoint::Round(portal_settings.line_joints);
    config.render_layers = RenderLayer::Game.layers();
}
