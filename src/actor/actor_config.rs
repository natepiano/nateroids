use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use super::Aabb;
use super::aabb;
use super::actor_template::MissileConfig;
use super::actor_template::NateroidConfig;
use super::actor_template::SpaceshipConfig;
use super::missile::Missile;
use super::nateroid::Nateroid;
use super::spaceship::Spaceship;
use crate::asset_loader::AssetsState;
use crate::asset_loader::SceneAssets;
use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;

// Spaceship model orientation correction: rotates the model so nose points +Y
// Shared between initial spawn and runtime 2D enforcement
pub const GLTF_ROTATION_X: f32 = std::f32::consts::FRAC_PI_2; // +90°

// call flow is to initialize the ensemble config which has the defaults
// for an actor - configure defaults in initial_actor_config.rs
pub struct ActorConfigPlugin;

impl Plugin for ActorConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetsState::Loaded), initialize_actor_configs)
            .add_observer(propagate_render_layers_on_spawn)
            .add_plugins(
                ResourceInspectorPlugin::<MissileConfig>::default()
                    .run_if(toggle_active(false, GameAction::MissileInspector)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<NateroidConfig>::default()
                    .run_if(toggle_active(false, GameAction::NateroidInspector)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<SpaceshipConfig>::default()
                    .run_if(toggle_active(false, GameAction::SpaceshipInspector)),
            );
    }
}

#[derive(Reflect, InspectorOptions, Clone, Debug)]
#[reflect(InspectorOptions)]
pub struct ActorConfig {
    pub spawnable:                bool,
    #[reflect(ignore)]
    pub aabb:                     Aabb,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub angular_damping:          Option<f32>,
    #[reflect(ignore)]
    pub collider:                 Collider,
    #[inspector(min = 0.1, max = 3.0, display = NumberDisplay::Slider)]
    pub collider_margin:          f32,
    pub collider_type:            ColliderType,
    pub collision_damage:         f32,
    pub collision_layers:         CollisionLayers,
    pub gravity_scale:            f32,
    pub health:                   f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub linear_damping:           Option<f32>,
    pub locked_axes:              LockedAxes,
    #[inspector(min = 0.0, max = 20.0, display = NumberDisplay::Slider)]
    pub mass:                     f32,
    #[inspector(min = 0.0, max = 500.0, display = NumberDisplay::Slider)]
    pub max_angular_velocity:     f32,
    #[inspector(min = 0.0, max = 500.0, display = NumberDisplay::Slider)]
    pub max_linear_velocity:      f32,
    pub render_layer:             RenderLayer,
    #[inspector(min = 0.1, max = 1.0, display = NumberDisplay::Slider)]
    pub restitution:              f32,
    pub restitution_combine_rule: CoefficientCombine,
    pub rigid_body:               RigidBody,
    #[reflect(ignore)]
    pub scene:                    Handle<Scene>,
    pub spawn_timer_seconds:      Option<f32>,
    pub transform:                Transform,
    #[reflect(ignore)]
    pub spawn_timer:              Option<Timer>,
}

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component)]
pub struct Health(pub f32);

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component)]
pub struct CollisionDamage(pub f32);

#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub enum ColliderType {
    Ball,
    Cuboid,
}

type ActorRenderLayersQuery<'w, 'a> =
    Query<'w, 'a, &'static RenderLayers, Or<(With<Missile>, With<Nateroid>, With<Spaceship>)>>;

/// ensures that the game camera can see the spawned actor and that shadows are cast
fn propagate_render_layers_on_spawn(
    add: On<Add, Children>,
    q_parents: ActorRenderLayersQuery,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    // Only process if this entity has one of our actor marker components (scene
    // children added to actor parent)
    if let Ok(parent_layers) = q_parents.get(add.entity) {
        // Propagate to all descendants using Bevy's built-in iterator
        for descendant in children_query.iter_descendants(add.entity) {
            commands.entity(descendant).insert(parent_layers.clone());
        }
    }
}

// Public constants for physics configuration (used by missile.rs, spaceship.rs, nateroid.rs)
pub const LOCKED_AXES_2D: LockedAxes = LockedAxes::new().lock_translation_z();
pub const LOCKED_AXES_SPACESHIP: LockedAxes = LockedAxes::new()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_translation_z();

pub fn initialize_actor_configs(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    scenes: Res<Assets<Scene>>,
    scene_assets: Res<SceneAssets>,
) {
    let mut nateroid_defaults = NateroidConfig::default();
    let nateroid_actor_config = initialize_actor_config(
        nateroid_defaults.actor_config,
        &scenes,
        &meshes,
        &scene_assets.nateroid,
    );
    nateroid_defaults.actor_config = nateroid_actor_config;
    commands.insert_resource(nateroid_defaults);

    let mut missile_defaults = MissileConfig::default();
    let missile_actor_config = initialize_actor_config(
        missile_defaults.actor_config,
        &scenes,
        &meshes,
        &scene_assets.missile,
    );
    missile_defaults.actor_config = missile_actor_config;
    commands.insert_resource(missile_defaults);

    let mut spaceship_defaults = SpaceshipConfig::default();
    let spaceship_actor_config = initialize_actor_config(
        spaceship_defaults.actor_config.clone(),
        &scenes,
        &meshes,
        &scene_assets.spaceship,
    );
    spaceship_defaults.actor_config = spaceship_actor_config;
    commands.insert_resource(spaceship_defaults);
}

pub fn create_spawn_timer(spawn_timer_seconds: Option<f32>) -> Option<Timer> {
    spawn_timer_seconds.map(|seconds| Timer::from_seconds(seconds, TimerMode::Repeating))
}

fn initialize_actor_config(
    mut config: ActorConfig,
    scenes: &Assets<Scene>,
    meshes: &Assets<Mesh>,
    scene_handle: &Handle<Scene>,
) -> ActorConfig {
    let aabb = aabb::get_scene_aabb(scenes, meshes, scene_handle);

    // Use raw AABB size - transform scale will handle sizing
    let size = aabb.size();

    let collider = match config.collider_type {
        ColliderType::Ball => {
            let radius = size.length() * config.collider_margin;
            Collider::sphere(radius)
        },
        ColliderType::Cuboid => Collider::cuboid(
            size.x * config.collider_margin,
            size.y * config.collider_margin,
            size.z * config.collider_margin,
        ),
    };

    config.aabb = aabb;
    config.collider = collider;
    config.spawn_timer = create_spawn_timer(config.spawn_timer_seconds);
    config.scene = scene_handle.clone();
    config
}

/// use config values so inspectors can provide new defaults
pub fn insert_configured_components(
    commands: &mut Commands,
    config: &mut ActorConfig,
    actor_entity: Entity,
) {
    // Insert all components on the actor entity
    commands.entity(actor_entity).insert((
        config.aabb.clone(),
        config.collider.clone(),
        CollisionDamage(config.collision_damage),
        config.collision_layers,
        GravityScale(config.gravity_scale),
        Health(config.health),
        Restitution {
            coefficient:  config.restitution,
            combine_rule: config.restitution_combine_rule,
        },
        Mass(config.mass),
        MaxAngularSpeed(config.max_angular_velocity),
        MaxLinearSpeed(config.max_linear_velocity),
        RenderLayers::from_layers(config.render_layer.layers()),
        SceneRoot(config.scene.clone()),
    ));

    // Apply damping if configured
    if let Some(linear) = config.linear_damping {
        commands.entity(actor_entity).insert(LinearDamping(linear));
    }
    if let Some(angular) = config.angular_damping {
        commands
            .entity(actor_entity)
            .insert(AngularDamping(angular));
    }

    // reset the timer if there is a configured spawn_timer_seconds
    config.spawn_timer = create_spawn_timer(config.spawn_timer_seconds);
}
