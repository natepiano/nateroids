use std::f32::consts::PI;

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rand::Rng;
use rand::prelude::ThreadRng;

use super::RenderLayer;
use super::config::StarConfig;
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
                (despawn_stars, spawn_stars).chain(),
            )
            .add_systems(
                OnEnter(GameState::GameOver),
                (despawn_stars, spawn_stars).chain(),
            )
            .add_systems(Update, rotate_stars.in_set(InGameSet::EntityUpdates))
            .add_systems(Update, debug_stars);
    }
}

fn debug_stars(
    stars: Query<(Entity, Option<&ViewVisibility>), With<Star>>,
    stars_camera: Query<
        (Entity, &Camera, Option<&RenderLayers>),
        With<super::cameras::StarsCamera>,
    >,
) {
    let count = stars.iter().count();
    if count > 0 {
        let visible_count = stars
            .iter()
            .filter(|(_, v)| v.is_some_and(|vv| vv.get()))
            .count();

        if let Ok((cam_entity, camera, render_layers)) = stars_camera.single() {
            debug!(
                "Stars: {count} total, {visible_count} visible | Camera {cam_entity}: active={}, layers={:?}",
                camera.is_active, render_layers
            );
        } else {
            debug!("Stars: {count} total, {visible_count} visible | NO STARS CAMERA!");
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

/// Spawn stars with all components at once to avoid archetype changes after spawn
fn spawn_stars(
    mut commands: Commands,
    config: Res<StarConfig>,
    boundary_config: Res<Boundary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    debug!("spawning stars");
    let longest_diagonal = boundary_config.longest_diagonal();
    let inner_sphere_radius = longest_diagonal + config.star_field_inner_diameter;
    let outer_sphere_radius = inner_sphere_radius + config.star_field_outer_diameter;

    let mesh = meshes.add(Sphere::new(1.));
    let mut rng = rand::rng();

    for _ in 0..config.star_count {
        let position = get_star_position(inner_sphere_radius, outer_sphere_radius, &mut rng);
        let radius = rng.random_range(config.star_radius_min..config.star_radius_max);
        let emissive = get_star_color(&config, &mut rng);

        let material = materials.add(StandardMaterial {
            emissive: LinearRgba::new(emissive.x, emissive.y, emissive.z, emissive.w),
            ..default()
        });

        commands.spawn((
            Star {
                position,
                radius,
                emissive,
            },
            RenderLayers::from_layers(RenderLayer::Stars.layers()),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_trs(position, Quat::IDENTITY, Vec3::splat(radius)),
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
