use bevy::{
    color::{
        palettes::{
            css,
            tailwind,
        },
        Color::Srgba,
    },
    prelude::*,
    render::view::{
        Layer,
        RenderLayers,
    },
};

use crate::inspector::AmbientLightBrightness;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AppearanceConfig>()
            .init_gizmo_group::<BoundaryGizmos>()
            .add_systems(Startup, init_gizmo_configs)
            .add_systems(Update, update_appearance_config);
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
pub struct BoundaryGizmos {}

fn init_gizmo_configs(
    mut config_store: ResMut<GizmoConfigStore>,
    appearance_config: Res<AppearanceConfig>,
) {
    for (_, any_config, _) in config_store.iter_mut() {
        any_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
        any_config.line_width = 2.;
    }

    // so we can avoid an error of borrowing the mutable config_store twice
    // in the same context
    {
        let (config, _) = config_store.config_mut::<BoundaryGizmos>();
        config.line_width = appearance_config.boundary_line_width;
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct AppearanceConfig {
    pub ambient_light_brightness:       f32,
    pub bloom_intensity:                f32,
    pub bloom_low_frequency_boost:      f32,
    pub bloom_high_pass_frequency:      f32,
    pub boundary_color:                 Color,
    pub boundary_distance_approach:     f32,
    pub boundary_distance_shrink:       f32,
    pub boundary_line_width:            f32,
    pub boundary_cell_count:            UVec3,
    pub boundary_scalar:                f32,
    pub clear_color:                    Color,
    pub clear_color_darkening_factor:   f32,
    pub missile_forward_spawn_distance: f32,
    pub smallest_teleport_circle:       f32,
    pub splash_timer:                   f32,
    pub zoom_sensitivity_pinch:         f32,
    pub zoom_sensitivity_mouse:         f32,
}

// centralize appearance defaults
// plus this allows us to use the inspector to dynamically change them
// to try out different ratios while the game is running
impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            ambient_light_brightness:       1000.,
            bloom_intensity:                0.9,
            bloom_low_frequency_boost:      0.5,
            bloom_high_pass_frequency:      0.5,
            boundary_color:                 Color::from(tailwind::BLUE_300),
            boundary_distance_approach:     0.5,
            boundary_distance_shrink:       0.25,
            boundary_line_width:            4.,
            boundary_cell_count:            UVec3::new(2, 1, 1),
            boundary_scalar:                110.,
            clear_color:                    Srgba(css::MIDNIGHT_BLUE),
            clear_color_darkening_factor:   0.019,
            missile_forward_spawn_distance: 5.6,
            smallest_teleport_circle:       5.,
            splash_timer:                   2.,
            zoom_sensitivity_pinch:         100.,
            zoom_sensitivity_mouse:         5.,
        }
    }
}

fn update_appearance_config(
    mut commands: Commands,
    ambient_light: Res<AmbientLightBrightness>,
    mut appearance_config: ResMut<AppearanceConfig>,
) {
    if ambient_light.is_changed() {
        appearance_config.ambient_light_brightness = ambient_light.0;
        commands.insert_resource(AmbientLight {
            color:      default(),
            brightness: appearance_config.ambient_light_brightness,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraOrder {
    Game,
    Stars,
}

impl CameraOrder {
    pub const fn order(self) -> isize {
        match self {
            CameraOrder::Game => 1,
            CameraOrder::Stars => 0,
        }
    }
}

// todo: how can i get PBRs to actually render on RenderLayer 1 so i could
// choose to have some       affected by bloom and some not...
// weird - if i put game on render layer 1 and stars on render layer 0,
// to line up with the camera order, the PBRs on render layer 1 are still
// showing on render layer 0 even though i don't think i asked for that

// used for both camera order and render layer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderLayer {
    Both,
    Game,
    Stars,
}

// returning the array rather than just one in case we have more complex
// situations in the future that require overlapping layers
impl RenderLayer {
    pub const fn layers(self) -> &'static [Layer] {
        match self {
            RenderLayer::Both => &[0, 1],
            RenderLayer::Game => &[0],
            RenderLayer::Stars => &[1],
        }
    }
}
