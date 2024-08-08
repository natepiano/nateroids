use crate::config::{AppearanceConfig, StarConfig};
use crate::input::GlobalAction;
use crate::{
    boundary::Boundary,
    camera::spawn_camera,
    config::{CameraOrder, RenderLayer},
};
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
    render::view::RenderLayers,
};
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera.before(spawn_camera))
            .add_systems(Update, toggle_stars)
            .init_resource::<StarBloom>()
            .insert_resource(StarSpawnTimer(Timer::from_seconds(
                0.05,
                TimerMode::Repeating,
            )))
            // replace with a vector of stars or handles or something and then despawn the oldest
            // and replace with new ones...
            .insert_resource(StarCounter(0))
            .add_systems(Update, spawn_star_tasks)
            .add_systems(Update, (rotate_sphere, update_bloom_settings));
    }
}

#[derive(Resource, Clone)]
struct StarBloom {
    settings: BloomSettings,
}

impl Default for StarBloom {
    fn default() -> Self {
        let config = AppearanceConfig::default();
        let mut bloom_settings = BloomSettings::NATURAL;
        bloom_settings.intensity = config.bloom_intensity;
        bloom_settings.low_frequency_boost = config.bloom_low_frequency_boost;
        bloom_settings.high_pass_frequency = config.bloom_high_pass_frequency;
        Self {
            settings: bloom_settings,
        }
    }
}

// allows Appearance settings to propagate back to StarBloom.settings so
// that on changes we can apply a clone of those settings back to
// the camera
impl StarBloom {
    fn update_from_config(&mut self, config: &AppearanceConfig) {
        self.settings.intensity = config.bloom_intensity;
        self.settings.low_frequency_boost = config.bloom_low_frequency_boost;
        self.settings.high_pass_frequency = config.bloom_high_pass_frequency;
    }
}

// if Appearance changes (ignore the fact that anything can change - that's fine)
// then propagate bloom settings back to the resource, and then clone it
// back onto the camera
fn update_bloom_settings(
    mut star_bloom: ResMut<StarBloom>,
    appearance_config: Res<AppearanceConfig>,
    mut query: Query<&mut BloomSettings, With<StarsCamera>>,
) {
    if appearance_config.is_changed() {
        star_bloom.update_from_config(&appearance_config);
        for mut bloom_settings in query.iter_mut() {
            *bloom_settings = star_bloom.settings.clone();
        }
    }
}

fn setup_camera(mut commands: Commands, star_bloom: Res<StarBloom>) {
    let camera3d = Camera3dBundle {
        camera: Camera {
            order: CameraOrder::Stars.order(),
            hdr: true, // 1. HDR is required for bloom
            ..default()
        },
        tonemapping: Tonemapping::BlenderFilmic,
        ..default()
    };

    commands
        .spawn(camera3d)
        .insert(RenderLayers::from_layers(RenderLayer::Stars.layers()))
        .insert(star_bloom.settings.clone())
        .insert(StarsCamera);
}

#[derive(Component)]
struct GameSphere;

// currently we're not displaying this - it's called but it never gets an Ok
// so it's a no-op
// keep this if you want to put the bounding sphere back in play...
fn rotate_sphere(mut query: Query<&mut Transform, With<GameSphere>>) {
    if let Ok(mut transform) = query.get_single_mut() {
        let delta_rotation = Quat::from_rotation_y(0.001);

        transform.rotation *= delta_rotation;
    }
}

#[derive(Component)]
pub struct StarsCamera;

fn toggle_stars(
    mut commands: Commands,
    mut camera: Query<(Entity, Option<&mut BloomSettings>), With<StarsCamera>>,
    user_input: Res<ActionState<GlobalAction>>,
    star_bloom: Res<StarBloom>,
) {
    let current_bloom_settings = camera.single_mut();

    match current_bloom_settings {
        (entity, Some(_)) => {
            if user_input.just_pressed(&GlobalAction::Stars) {
                println!("stars off");
                commands.entity(entity).remove::<BloomSettings>();
            }
        }
        (entity, None) => {
            if user_input.just_pressed(&GlobalAction::Stars) {
                println!("stars on");
                commands.entity(entity).insert(star_bloom.settings.clone());
            }
        }
    }
}

#[derive(Resource)]
struct StarSpawnTimer(Timer);

#[derive(Resource)]
struct StarCounter(usize);

#[derive(Component)]
pub struct Stars;

// generate BATCH_SIZE stars at a time until you get to the desired number of stars
// this will fill them in as the program starts and looks cooler that way
#[allow(clippy::too_many_arguments)]
fn spawn_star_tasks(
    mut commands: Commands,
    config: Res<StarConfig>,
    boundary: Res<Boundary>,
    time: Res<Time>,
    mut timer: ResMut<StarSpawnTimer>,
    mut counter: ResMut<StarCounter>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if timer.0.tick(time.delta()).just_finished() && counter.0 < config.star_count {
        let longest_diagonal = boundary.longest_diagonal;
        let inner_sphere_radius = longest_diagonal + config.star_field_inner_diameter;
        let outer_sphere_radius = inner_sphere_radius + config.star_field_outer_diameter;

        let stars_to_spawn = (config.star_count - counter.0).min(config.star_spawn_batch_size);

        for _ in 0..stars_to_spawn {
            let mut rng = rand::thread_rng();
            let point = {
                let u: f32 = rng.gen_range(0.0..1.0);
                let v: f32 = rng.gen_range(0.0..1.0);
                let theta = u * std::f32::consts::PI * 2.0;
                let phi = (2.0 * v - 1.0).acos();
                let r = rng.gen_range(inner_sphere_radius..outer_sphere_radius);

                let x = r * theta.cos() * phi.sin();
                let y = r * theta.sin() * phi.sin();
                let z = r * phi.cos();

                Vec3::new(x, y, z)
            };

            // Increase the likelihood of generating higher values for R, G, B
            let emissive_r = rng.gen_range(8.0..15.0);
            let emissive_g = rng.gen_range(8.0..15.0);
            let emissive_b = rng.gen_range(8.0..15.0);
            let emissive_a = rng.gen_range(8.0..15.0);

            let transform = Transform::from_translation(point);

            let material = materials.add(StandardMaterial {
                emissive: LinearRgba::new(emissive_r, emissive_g, emissive_b, emissive_a),
                ..default()
            });

            let min = config.star_radius / 10.;

            let radius = rng.gen_range(min..config.star_radius);
            let star_mesh_handle = meshes.add(Sphere::new(radius).mesh());

            commands
                .spawn(PbrBundle {
                    mesh: star_mesh_handle.clone(),
                    material,
                    transform,
                    ..default()
                })
                .insert(Stars)
                .insert(RenderLayers::from_layers(RenderLayer::Stars.layers()));
        }

        counter.0 += stars_to_spawn;
    }
}
