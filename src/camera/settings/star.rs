use std::ops::Range;

use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::camera::constants::STAR_BATCH_SIZE_REPLACE;
use crate::camera::constants::STAR_COLOR_RANGE_MAX;
use crate::camera::constants::STAR_COLOR_RANGE_MIN;
use crate::camera::constants::STAR_COLOR_WHITE_PROBABILITY;
use crate::camera::constants::STAR_COLOR_WHITE_START_RATIO;
use crate::camera::constants::STAR_COUNT;
use crate::camera::constants::STAR_DURATION_REPLACE_TIMER;
use crate::camera::constants::STAR_FIELD_DIAMETER;
use crate::camera::constants::STAR_RADIUS;
use crate::camera::constants::STAR_ROTATION_CYCLE_MAX;
use crate::camera::constants::STAR_ROTATION_CYCLE_MINIMUM_MINUTES;
use crate::camera::constants::STAR_ROTATION_CYCLE_MINUTES;
use crate::camera::constants::STAR_TWINKLE_CHOOSE_MULTIPLE_COUNT;
use crate::camera::constants::STAR_TWINKLE_DURATION_MAX;
use crate::camera::constants::STAR_TWINKLE_DURATION_MIN;
use crate::camera::constants::STAR_TWINKLE_INTENSITY_MAX;
use crate::camera::constants::STAR_TWINKLE_INTENSITY_MIN;
use crate::camera::constants::STAR_TWINKLING_DELAY;
use crate::input::InspectStarSwitch;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(InspectStarEvent);

pub(super) struct StarSettingsInspectorPlugin;

impl Plugin for StarSettingsInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ResourceInspectorPlugin::<StarSettings>::default()
                .run_if(switches::is_switch_on(Switch::InspectStar)),
        )
        .init_resource::<StarSettings>();
        bind_action_switch!(
            app,
            InspectStarSwitch,
            InspectStarEvent,
            Switch::InspectStar
        );
    }
}

#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct StarColorSettings {
    pub range:             Range<f32>,
    pub white_probability: f32,
    pub white_start_ratio: f32,
}

#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub struct StarTwinkleSettings {
    pub delay:                 f32,
    pub duration:              Range<f32>,
    pub intensity:             Range<f32>,
    pub choose_multiple_count: usize,
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct StarSettings {
    pub batch_size_replace:     usize,
    pub duration_replace_timer: f32,
    pub color:                  StarColorSettings,
    pub count:                  usize,
    pub radius:                 Range<f32>,
    pub field_diameter:         Range<f32>,
    pub twinkle:                StarTwinkleSettings,
    #[inspector(
        min = STAR_ROTATION_CYCLE_MINIMUM_MINUTES,
        max = STAR_ROTATION_CYCLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub rotation_cycle_minutes: f32,
    pub rotation_axis:          Vec3,
}

impl Default for StarSettings {
    fn default() -> Self {
        Self {
            batch_size_replace:     STAR_BATCH_SIZE_REPLACE,
            duration_replace_timer: STAR_DURATION_REPLACE_TIMER,
            count:                  STAR_COUNT,
            color:                  StarColorSettings {
                range:             STAR_COLOR_RANGE_MIN..STAR_COLOR_RANGE_MAX,
                white_probability: STAR_COLOR_WHITE_PROBABILITY,
                white_start_ratio: STAR_COLOR_WHITE_START_RATIO,
            },
            radius:                 STAR_RADIUS,
            field_diameter:         STAR_FIELD_DIAMETER,
            twinkle:                StarTwinkleSettings {
                delay:                 STAR_TWINKLING_DELAY,
                duration:              STAR_TWINKLE_DURATION_MIN..STAR_TWINKLE_DURATION_MAX,
                intensity:             STAR_TWINKLE_INTENSITY_MIN..STAR_TWINKLE_INTENSITY_MAX,
                choose_multiple_count: STAR_TWINKLE_CHOOSE_MULTIPLE_COUNT,
            },
            rotation_cycle_minutes: STAR_ROTATION_CYCLE_MINUTES,
            rotation_axis:          Vec3::Y,
        }
    }
}
