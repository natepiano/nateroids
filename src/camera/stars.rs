use std::any::TypeId;
use std::f32::consts::PI;
use std::ops::Range;

use bevy::camera::visibility::RenderLayers;
use bevy::camera::visibility::VisibleEntities;
use bevy::diagnostic::FrameCount;
use bevy::mesh::Mesh3d;
use bevy::mesh::MeshTag;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use hana_kana::Position;
use hana_kana::ToU32;
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
use super::constants::STAR_ROTATION_CYCLE_MIN;
use super::constants::STAR_ROTATION_CYCLE_MINIMUM_MINUTES;
use super::constants::STAR_ROTATION_CYCLE_MINUTES;
use super::constants::STAR_TWINKLE_AMPLITUDE;
use super::constants::STAR_TWINKLE_AMPLITUDE_MAX;
use super::constants::STAR_TWINKLE_AMPLITUDE_MIN;
use super::constants::STAR_TWINKLE_SPEED;
use super::constants::STAR_TWINKLE_SPEED_MAX;
use super::constants::STAR_TWINKLE_SPEED_MIN;
use super::star::StarCamera;
use super::star_material::StarData;
use super::star_material::StarMaterial;
use super::star_material::StarMaterialExtension;
use super::star_twinkling::StarTwinkling;
use crate::input::InspectStarSwitch;
use crate::playfield::Boundary;
use crate::state::GameState;
use crate::switches;
use crate::switches::Switch;

pub(super) struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        // `Range<f32>` is reflected as opaque with no serialization type data, which
        // blocks BRP mutation of `StarSettings` range fields. Registering the generic
        // `Range<f32>` plus `ReflectSerialize`/`ReflectDeserialize` makes them mutable
        // at runtime.
        app.register_type::<Range<f32>>()
            .register_type_data::<Range<f32>, ReflectSerialize>()
            .register_type_data::<Range<f32>, ReflectDeserialize>();
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

event!(InspectStarEvent);

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
    /// Master brightness swing applied uniformly to every star each frame; each
    /// star scales it by its own `StarData::twinkle` amplitude fraction.
    #[inspector(
        min = STAR_TWINKLE_AMPLITUDE_MIN,
        max = STAR_TWINKLE_AMPLITUDE_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) amplitude: f32,
    /// Master cycle rate applied uniformly to every star; each star scales it by
    /// its own `StarData::twinkle` speed fraction.
    #[inspector(
        min = STAR_TWINKLE_SPEED_MIN,
        max = STAR_TWINKLE_SPEED_MAX,
        display = NumberDisplay::Slider
    )]
    pub(super) speed:     f32,
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
        min = STAR_ROTATION_CYCLE_MIN,
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
                amplitude: STAR_TWINKLE_AMPLITUDE,
                speed:     STAR_TWINKLE_SPEED,
            },
            rotation_cycle_minutes: STAR_ROTATION_CYCLE_MINUTES,
            rotation_axis:          Vec3::Y,
        }
    }
}

#[derive(Reflect, Component, Default)]
pub(super) struct Star {
    position: Position,
    radius:   f32,
}

#[derive(Resource)]
struct StarRotationState {
    current_angle: f32,
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
    mut materials: ResMut<Assets<StarMaterial>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut star_twinkling: ResMut<StarTwinkling>,
) {
    debug!("spawning stars");
    star_twinkling.reset();
    star_twinkling.stars.clear();
    if star_settings.count == 0 {
        star_twinkling.star_buffer = default();
        star_twinkling.clock_buffer = default();
        return;
    }
    let longest_diagonal = boundary.longest_diagonal();
    let inner_sphere_radius = longest_diagonal + star_settings.field_diameter.start;
    let outer_sphere_radius = inner_sphere_radius + star_settings.field_diameter.end;
    let mesh = meshes.add(Rectangle::new(2.0, 2.0));
    let mut rng = rng();
    let mut stars = Vec::with_capacity(star_settings.count);

    for _ in 0..star_settings.count {
        let position = get_star_position(inner_sphere_radius, outer_sphere_radius, &mut rng);
        let radius = rng.random_range(star_settings.radius.clone());
        let emissive = get_star_color(&star_settings, &mut rng);
        star_twinkling
            .stars
            .push(StarData::random(emissive, &mut rng));
        stars.push(Star { position, radius });
    }
    star_twinkling.star_buffer = buffers.add(ShaderBuffer::from(star_twinkling.stars.clone()));
    star_twinkling.clock_buffer = buffers.add(ShaderBuffer::from(vec![Vec4::new(
        0.0,
        star_settings.twinkle.amplitude,
        0.0,
        0.0,
    )]));
    let material = materials.add(StarMaterial {
        base:      StandardMaterial {
            base_color: Color::BLACK,
            reflectance: 0.0,
            alpha_mode: AlphaMode::Add,
            ..default()
        },
        extension: StarMaterialExtension {
            stars: star_twinkling.star_buffer.clone(),
            clock: star_twinkling.clock_buffer.clone(),
        },
    });
    for (index, star) in stars.into_iter().enumerate() {
        let transform = Transform {
            translation: *star.position,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::splat(star.radius),
        };
        commands.spawn((
            star,
            RenderLayer::Stars.layers(),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            MeshTag::new(index.to_u32()),
            transform,
        ));
    }
}

fn get_star_position(
    inner_sphere_radius: f32,
    outer_sphere_radius: f32,
    rng: &mut ThreadRng,
) -> Position {
    // `azimuth_norm` and `polar_norm` sample a uniform point on the spherical shell.
    let azimuth_norm: f32 = rng.random_range(0.0..1.0);
    let polar_norm: f32 = rng.random_range(0.0..1.0);

    let theta = azimuth_norm * PI * 2.0;
    // `f32::mul_add` maps `polar_norm` from `[0, 1]` to `[-1, 1]` before `acos`.
    let phi = 2.0f32.mul_add(polar_norm, -1.0).acos();
    let radius = rng.random_range(inner_sphere_radius..outer_sphere_radius);

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

    let mut r = rng.random_range(start..end);
    let mut g = rng.random_range(start..end);
    let mut b = rng.random_range(start..end);

    // `min_brightness` scales RGB together so the sampled color ratios are retained.
    let min_brightness = (end - start).mul_add(STAR_MINIMUM_BRIGHTNESS_FRACTION, start);
    let current_brightness = r.max(g).max(b);

    if current_brightness < min_brightness {
        let scale = min_brightness / current_brightness;
        r *= scale;
        g *= scale;
        b *= scale;
    }

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

    let rotation_speed = (2.0 * PI) / (star_settings.rotation_cycle_minutes * SECONDS_PER_MINUTE);

    // Decreasing `StarRotationState::current_angle` rotates clockwise when viewed
    // from above.
    rotation_state.current_angle =
        rotation_speed.mul_add(-time.delta_secs(), rotation_state.current_angle);

    // Each `Star::position` rotates around `StarSettings::rotation_axis`.
    let rotation = Quat::from_axis_angle(star_settings.rotation_axis, rotation_state.current_angle);

    for (star, mut transform) in &mut stars {
        transform.translation = rotation * *star.position;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use bevy::mesh::MeshTag;
    use bevy::prelude::*;
    use bevy::render::storage::ShaderBuffer;

    use super::Star;
    use super::StarRotationState;
    use super::StarSettings;
    use super::despawn_stars;
    use super::spawn_stars;
    use crate::camera::star_material::StarMaterial;
    use crate::camera::star_twinkling::StarTwinkling;
    use crate::playfield::Boundary;

    const STAR_COUNT: usize = 3;

    #[test]
    fn stars_share_material_and_mesh_with_distinct_buffer_indices() {
        let mut app = App::new();
        app.init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StarMaterial>>()
            .init_resource::<Assets<ShaderBuffer>>()
            .init_resource::<Boundary>()
            .init_resource::<StarTwinkling>()
            .insert_resource(StarRotationState { current_angle: 0.0 })
            .insert_resource(StarSettings {
                count: STAR_COUNT,
                ..default()
            })
            .add_systems(Update, (despawn_stars, spawn_stars).chain());
        app.update();

        let world = app.world_mut();
        let mut query = world
            .query_filtered::<(&Mesh3d, &MeshMaterial3d<StarMaterial>, &MeshTag), With<Star>>();
        let mut meshes = HashSet::new();
        let mut materials = HashSet::new();
        let mut tags = Vec::new();
        for (mesh, material, tag) in query.iter(world) {
            meshes.insert(mesh.0.id());
            materials.insert(material.0.id());
            tags.push(tag.value);
        }
        tags.sort_unstable();
        assert_eq!(tags, [0, 1, 2]);
        assert_eq!(meshes.len(), 1);
        assert_eq!(materials.len(), 1);
        assert_eq!(world.resource::<Assets<StarMaterial>>().len(), 1);
        assert_eq!(world.resource::<StarTwinkling>().stars.len(), STAR_COUNT);

        world.resource_mut::<StarSettings>().count = 0;
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<Star>>()
                .iter(world)
                .count(),
            0
        );
        assert!(world.resource::<StarTwinkling>().stars.is_empty());

        world.resource_mut::<StarSettings>().count = 1;
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<Star>>()
                .iter(world)
                .count(),
            1
        );
        assert_eq!(world.resource::<StarTwinkling>().stars.len(), 1);
    }
}
