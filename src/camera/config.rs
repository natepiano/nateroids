use crate::global_input::{
    GlobalAction,
    toggle_active,
};
use bevy::{
    color::palettes::tailwind,
    prelude::*,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};

pub struct CameraConfigPlugin;

impl Plugin for CameraConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CameraConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<CameraConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::CameraConfigInspector)),
            )
            .init_resource::<CameraConfig>();
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, PartialEq, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct CameraConfig {
    pub clear_color:               Color,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub darkening_factor:          f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_intensity:           f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_low_frequency_boost: f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub bloom_high_pass_frequency: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            clear_color:               Color::from(tailwind::SLATE_900),
            darkening_factor:          0.002,
            bloom_intensity:           0.9,
            bloom_low_frequency_boost: 0.5,
            bloom_high_pass_frequency: 0.5,
        }
    }
}
