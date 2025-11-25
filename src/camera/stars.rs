use std::f32::consts::PI;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rand::prelude::ThreadRng;
use rand::Rng;

use super::config::StarConfig;
use super::RenderLayer;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::state::GameState;
use crate::traits::TransformExt;

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(StarRotationState { current_angle: 0.0 })
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
    debug!("despawning stars");
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
    debug!("spawning stars");
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
    debug!("setting up star rendering");
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
