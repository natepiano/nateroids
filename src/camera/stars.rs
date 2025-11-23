use std::f32::consts::PI;
use std::ops::Range;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rand::prelude::ThreadRng;
use rand::Rng;

use crate::camera::RenderLayer;
use crate::game_input::toggle_active;
use crate::game_input::GameAction;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::state::GameState;
use crate::traits::TransformExt;

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StarRotationState { current_angle: 0.0 })
            .add_plugins(
                ResourceInspectorPlugin::<StarConfig>::default()
                    .run_if(toggle_active(false, GameAction::StarConfigInspector)),
            )
            .add_systems(
                OnEnter(GameState::Splash),
                (despawn_stars, spawn_stars, setup_star_rendering).chain(),
            )
            .add_systems(
                OnEnter(GameState::GameOver),
                (despawn_stars, spawn_stars, setup_star_rendering).chain(),
            )
            .add_systems(Update, rotate_stars.in_set(InGameSet::EntityUpdates));
    }
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
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
    #[inspector(min = 0.01667, max = 30.0, display = NumberDisplay::Slider)]
    pub rotation_cycle_minutes:        f32,
    pub rotation_axis:                 Vec3,
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
            rotation_cycle_minutes:        15., // i mean why not
            rotation_axis:                 Vec3::Y,
        }
    }
}

#[derive(Reflect, Component, Default)]
pub struct Star {
    position:     Vec3,
    radius:       f32,
    pub emissive: Vec4,
}

#[derive(Resource)]
struct StarRotationState {
    current_angle: f32,
}

fn despawn_stars(
    mut commands: Commands,
    stars: Query<Entity, With<Star>>,
    mut rotation_state: ResMut<StarRotationState>,
) {
    for entity in stars.iter() {
        commands.entity(entity).despawn();
    }
    // Reset rotation angle so new stars start from 0 (prevents jump on reset)
    // This was a nasty bug - we couldn't tell why the Splash animation would land smoothly
    // but when we manally re-invoked this, it looked like the spaceship jumped with
    // respect to the star background at the end - thinking this was a camera movement but
    // but it was actually that we needed to reset the rotation angle so we wouldn't be using the
    // previous rotation state when spawning a new set of stars. dang!
    rotation_state.current_angle = 0.0;
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
    // Generate uniform random points on spherical shell using spherical coordinates
    let azimuth_norm: f32 = rng.random_range(0.0..1.0); // normalized azimuthal angle
    let polar_norm: f32 = rng.random_range(0.0..1.0); // normalized polar angle

    let theta = azimuth_norm * std::f32::consts::PI * 2.0; // azimuthal: 0 to 2π
                                                           // FMA optimization (faster + more precise): 2.0 * polar_norm - 1.0
    let phi = 2.0f32.mul_add(polar_norm, -1.0).acos(); // polar angle
    let radius = rng.random_range(inner_sphere_radius..outer_sphere_radius);

    // Convert spherical to Cartesian coordinates
    let x = radius * theta.cos() * phi.sin();
    let y = radius * theta.sin() * phi.sin();
    let z = radius * phi.cos();

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
    // FMA optimization (faster + more precise): start + (end - start) * 0.2
    let min_brightness = (end - start).mul_add(0.2, start); // 20% above start
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
            .insert(Transform::from_trs(
                star.position,
                Quat::IDENTITY,
                Vec3::splat(star.radius),
            ));
    }
}

fn rotate_stars(
    time: Res<Time>,
    config: Res<StarConfig>,
    mut rotation_state: ResMut<StarRotationState>,
    mut stars: Query<(&Star, &mut Transform)>,
) {
    // Guard against invalid rotation cycle values (min: 1 second = 0.01667 minutes)
    if config.rotation_cycle_minutes < 0.01667 {
        return;
    }

    // Calculate rotation speed (radians per second)
    let rotation_speed = (2.0 * PI) / (config.rotation_cycle_minutes * 60.0);

    // Update current angle (negative for clockwise rotation when viewed from above)
    rotation_state.current_angle -= rotation_speed * time.delta_secs();

    // Apply rotation to each star around the configured axis
    let rotation = Quat::from_axis_angle(config.rotation_axis, rotation_state.current_angle);

    for (star, mut transform) in &mut stars {
        transform.translation = rotation * star.position;
    }
}
