use std::any::TypeId;
use std::f32::consts::PI;
use std::ops::Range;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::visibility::VisibleEntities;
use bevy::diagnostic::FrameCount;
use bevy::mesh::Mesh3d;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_kana::Position;
use rand::Rng;
use rand::RngExt;
use rand::prelude::ThreadRng;
use rand::rng;

use super::RenderLayer;
use super::constants::SECONDS_PER_MINUTE;
use super::constants::STAR_BATCH_SIZE_REPLACE;
use super::constants::STAR_COLOR_RANGE_MAX;
use super::constants::STAR_COLOR_RANGE_MIN;
use super::constants::STAR_COLOR_WHITE_PROBABILITY;
use super::constants::STAR_COLOR_WHITE_START_RATIO;
use super::constants::STAR_COUNT;
use super::constants::STAR_DURATION_REPLACE_TIMER;
use super::constants::STAR_FIELD_DIAMETER;
use super::constants::STAR_MINIMUM_BRIGHTNESS_FRACTION;
use super::constants::STAR_RADIUS;
use super::constants::STAR_ROTATION_CYCLE_MAX;
use super::constants::STAR_ROTATION_CYCLE_MINIMUM_MINUTES;
use super::constants::STAR_ROTATION_CYCLE_MINUTES;
use super::constants::STAR_TWINKLE_CHOOSE_MULTIPLE_COUNT;
use super::constants::STAR_TWINKLE_DURATION_MAX;
use super::constants::STAR_TWINKLE_DURATION_MIN;
use super::constants::STAR_TWINKLE_INTENSITY_MAX;
use super::constants::STAR_TWINKLE_INTENSITY_MIN;
use super::constants::STAR_TWINKLING_DELAY;
use super::star::StarCamera;
use crate::input::InspectStarSwitch;
use crate::playfield::Boundary;
use crate::state::GameState;
use crate::switches;
use crate::switches::Switch;

event!(InspectStarEvent);

pub(super) struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StarSettings>()
            .insert_resource(StarRotationState { current_angle: 0.0 })
            .add_plugins(
                ResourceInspectorPlugin::<StarSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectStar)),
            )
            .add_systems(
                OnEnter(GameState::Splash),
                (despawn_stars, spawn_stars).chain(),
            )
            .add_systems(
                OnEnter(GameState::GameOver),
                (despawn_stars, spawn_stars).chain(),
            )
            .add_systems(Update, rotate_stars)
            .add_systems(Update, debug_stars);
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
pub(super) struct StarColorSettings {
    pub(super) range:             Range<f32>,
    pub(super) white_probability: f32,
    pub(super) white_start_ratio: f32,
}

#[derive(Debug, Clone, Reflect, InspectorOptions)]
#[reflect(InspectorOptions)]
pub(super) struct StarTwinkleSettings {
    pub(super) delay:                 f32,
    pub(super) duration:              Range<f32>,
    pub(super) intensity:             Range<f32>,
    pub(super) choose_multiple_count: usize,
}

#[derive(Debug, Clone, Reflect, Resource, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub(super) struct StarSettings {
    pub(super) batch_size_replace:     usize,
    pub(super) duration_replace_timer: f32,
    pub(super) color:                  StarColorSettings,
    pub(super) count:                  usize,
    pub(super) radius:                 Range<f32>,
    pub(super) field_diameter:         Range<f32>,
    pub(super) twinkle:                StarTwinkleSettings,
    #[inspector(
        min = STAR_ROTATION_CYCLE_MINIMUM_MINUTES,
        max = STAR_ROTATION_CYCLE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) rotation_cycle_minutes: f32,
    pub(super) rotation_axis:          Vec3,
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

fn debug_stars(
    frame_count: Res<FrameCount>,
    stars: Query<(Entity, Option<&ViewVisibility>), With<Star>>,
    stars_camera: Query<
        (
            Entity,
            &Camera,
            Option<&RenderLayers>,
            Option<&VisibleEntities>,
        ),
        With<StarCamera>,
    >,
) {
    let frame = frame_count.0;
    let count = stars.iter().count();
    if count > 0 {
        let visible_count = stars
            .iter()
            .filter(|(_, v)| v.copied().is_some_and(ViewVisibility::get))
            .count();

        if let Ok((camera_entity, camera, render_layers, visible_entities)) = stars_camera.single()
        {
            let mesh3d_visible = visible_entities.map_or(0, |ve| {
                ve.entities.get(&TypeId::of::<Mesh3d>()).map_or(0, Vec::len)
            });
            debug!(
                "Frame {frame}: Stars: {count} total, {visible_count} ViewVisible | Camera {camera_entity}: active={}, layers={render_layers:?}, VisibleEntities(Mesh3d)={mesh3d_visible}",
                camera.is_active
            );
        } else {
            debug!(
                "Frame {frame}: Stars: {count} total, {visible_count} ViewVisible | NO STARS CAMERA!"
            );
        }
    }
}

#[derive(Reflect, Component, Default)]
pub(super) struct Star {
    position:            Position,
    radius:              f32,
    pub(super) emissive: Vec4,
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
    // Reset `StarRotationState::current_angle` before regenerating `Star`
    // entities. Without the reset, re-running the `Splash` animation keeps the
    // previous rotation state and the spaceship appears to jump relative to the
    // star background.
    rotation_state.current_angle = 0.0;
}

/// Spawn stars with all components at once to avoid archetype changes after spawn
fn spawn_stars(
    mut commands: Commands,
    star_settings: Res<StarSettings>,
    boundary: Res<Boundary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    debug!("spawning stars");
    let longest_diagonal = boundary.longest_diagonal();
    let inner_sphere_radius = longest_diagonal + star_settings.field_diameter.start;
    let outer_sphere_radius = inner_sphere_radius + star_settings.field_diameter.end;

    let mesh = meshes.add(Sphere::new(1.));
    let mut rng = rng();

    for _ in 0..star_settings.count {
        let position = get_star_position(inner_sphere_radius, outer_sphere_radius, &mut rng);
        let radius = rng.random_range(star_settings.radius.clone());
        let emissive = get_star_color(&star_settings, &mut rng);

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
            RenderLayer::Stars.layers(),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform {
                translation: *position,
                rotation:    Quat::IDENTITY,
                scale:       Vec3::splat(radius),
            },
        ));
    }
}

fn get_star_position(
    inner_sphere_radius: f32,
    outer_sphere_radius: f32,
    rng: &mut ThreadRng,
) -> Position {
    // Generate uniform random points on spherical shell using spherical coordinates
    let azimuth_norm: f32 = rng.random_range(0.0..1.0); // normalized azimuthal angle
    let polar_norm: f32 = rng.random_range(0.0..1.0); // normalized polar angle

    let theta = azimuth_norm * PI * 2.0; // azimuthal: 0 to 2π
    // FMA optimization (faster + more precise): 2.0 * polar_norm - 1.0
    let phi = 2.0f32.mul_add(polar_norm, -1.0).acos(); // polar angle
    let radius = rng.random_range(inner_sphere_radius..outer_sphere_radius);

    // Convert spherical to Cartesian coordinates
    let x = radius * theta.cos() * phi.sin();
    let y = radius * theta.sin() * phi.sin();
    let z = radius * phi.cos();

    Position::new(x, y, z)
}

fn get_star_color(star_settings: &StarSettings, rng: &mut impl Rng) -> Vec4 {
    let end = star_settings.color.range.end;
    let color_start = star_settings.color.range.start;
    let white_start = end * star_settings.color.white_start_ratio;

    let start = if rng.random::<f32>() < star_settings.color.white_probability {
        white_start
    } else {
        color_start
    };

    // Generate initial color components
    let mut r = rng.random_range(start..end);
    let mut g = rng.random_range(start..end);
    let mut b = rng.random_range(start..end);

    // Ensure minimum brightness
    // FMA optimization (faster + more precise): start + (end - start) * fraction
    let min_brightness = (end - start).mul_add(STAR_MINIMUM_BRIGHTNESS_FRACTION, start);
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
    star_settings: Res<StarSettings>,
    mut rotation_state: ResMut<StarRotationState>,
    mut stars: Query<(&Star, &mut Transform)>,
) {
    if star_settings.rotation_cycle_minutes < STAR_ROTATION_CYCLE_MINIMUM_MINUTES {
        return;
    }

    // Calculate rotation speed (radians per second)
    let rotation_speed = (2.0 * PI) / (star_settings.rotation_cycle_minutes * SECONDS_PER_MINUTE);

    // Update current angle (negative for clockwise rotation when viewed from above)
    rotation_state.current_angle -= rotation_speed * time.delta_secs();

    // Apply rotation to each star around the configured axis
    let rotation = Quat::from_axis_angle(star_settings.rotation_axis, rotation_state.current_angle);

    for (star, mut transform) in &mut stars {
        transform.translation = rotation * *star.position;
    }
}
