use crate::playfield::Boundary;
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
};
use std::ops::Range;

use crate::camera::RenderLayer;
use rand::{
    Rng,
    prelude::ThreadRng,
};

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_stars, setup_star_rendering).chain());
    }
}

#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct StarConfig {
    pub batch_size_replace:            usize,
    pub duration_replace_timer:        f32,
    pub star_color:                    Range<f32>,
    pub star_color_white_probability:  f32,
    pub star_color_white_start_ratio:  f32,
    pub star_count:                    usize,
    pub star_radius_max:               f32,
    pub star_radius_min:               f32,
    pub star_field_inner_diameter:     f32,
    pub star_field_outer_diameter:     f32,
    pub start_twinkling_delay:         f32,
    pub twinkle_duration:              Range<f32>,
    pub twinkle_intensity:             Range<f32>,
    pub twinkle_choose_multiple_count: usize,
}

impl Default for StarConfig {
    fn default() -> Self {
        Self {
            batch_size_replace:            10,
            duration_replace_timer:        1.,
            star_count:                    1000,
            star_color:                    -30.0..30.0,
            star_color_white_probability:  0.85,
            star_color_white_start_ratio:  0.7,
            star_radius_max:               2.5,
            star_radius_min:               0.3,
            star_field_inner_diameter:     200.,
            star_field_outer_diameter:     400.,
            start_twinkling_delay:         0.5,
            twinkle_duration:              0.5..2.,
            twinkle_intensity:             10.0..20.,
            twinkle_choose_multiple_count: 2, // stars to look at each update
        }
    }
}

#[derive(Reflect, Component, Default)]
pub struct Star {
    position:     Vec3,
    radius:       f32,
    pub emissive: Vec4,
}

// just set up the entities with their positions - we'll add an emissive
// StandardMaterial separately
fn spawn_stars(mut commands: Commands, config: Res<StarConfig>, boundary_config: Res<Boundary>) {
    let longest_diagonal = boundary_config.longest_diagonal();
    let inner_sphere_radius = longest_diagonal + config.star_field_inner_diameter;
    let outer_sphere_radius = inner_sphere_radius + config.star_field_outer_diameter;

    let mut rng = rand::rng();

    for _ in 0..config.star_count {
        let point = get_star_position(inner_sphere_radius, outer_sphere_radius, &mut rng);
        let radius = rng.random_range(config.star_radius_min..config.star_radius_max);
        let emissive = get_star_color(&config, &mut rng);

        commands.spawn((
            Star {
                position: point,
                radius,
                emissive,
            },
            RenderLayers::from_layers(RenderLayer::Stars.layers()),
        ));
    }
}

fn get_star_position(
    inner_sphere_radius: f32,
    outer_sphere_radius: f32,
    rng: &mut ThreadRng,
) -> Vec3 {
    let u: f32 = rng.random_range(0.0..1.0);
    let v: f32 = rng.random_range(0.0..1.0);
    let theta = u * std::f32::consts::PI * 2.0;
    let phi = (2.0 * v - 1.0).acos();
    let r = rng.random_range(inner_sphere_radius..outer_sphere_radius);

    let x = r * theta.cos() * phi.sin();
    let y = r * theta.sin() * phi.sin();
    let z = r * phi.cos();

    Vec3::new(x, y, z)
}

fn get_star_color(config: &StarConfig, rng: &mut impl Rng) -> Vec4 {
    let end = config.star_color.end;
    let color_start = config.star_color.start;
    let white_start = end * config.star_color_white_start_ratio;

    let start = if rng.random::<f32>() < config.star_color_white_probability {
        white_start
    } else {
        color_start
    };

    // Generate initial color components
    let mut r = rng.random_range(start..end);
    let mut g = rng.random_range(start..end);
    let mut b = rng.random_range(start..end);

    // Ensure minimum brightness
    let min_brightness = start + (end - start) * 0.2; // 20% above start
    let current_brightness = r.max(g).max(b);

    if current_brightness < min_brightness {
        let scale = min_brightness / current_brightness;
        r *= scale;
        g *= scale;
        b *= scale;
    }

    // Alpha can remain as is
    let a = rng.random_range(start..end);

    Vec4::new(r, g, b, a)
}

// add the emissive standard material generated randomly in spawn_stars
fn setup_star_rendering(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    stars: Query<(Entity, &Star)>,
) {
    let mesh = meshes.add(Sphere::new(1.));

    for (entity, star) in stars.iter() {
        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::new(
                star.emissive.x,
                star.emissive.y,
                star.emissive.z,
                star.emissive.w,
            ),
            ..default()
        });

        commands
            .entity(entity)
            .insert(Mesh3d(mesh.clone()))
            .insert(MeshMaterial3d(material))
            .insert(
                Transform::from_translation(star.position).with_scale(Vec3::splat(star.radius)),
            );
    }
}
