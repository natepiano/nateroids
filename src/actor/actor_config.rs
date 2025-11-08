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
use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;

// this is how far off we are from blender for the assets we're loading
// we need to get them scaled up to generate a usable aabb
const BLENDER_SCALE: f32 = 100.;

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
                    .run_if(toggle_active(false, GlobalAction::MissileInspector)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<NateroidConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::NateroidInspector)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<SpaceshipConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::SpaceshipInspector)),
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

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            spawnable:                true,
            aabb:                     Aabb::default(),
            angular_damping:          None,
            collider:                 Collider::cuboid(1., 1., 1.),
            collider_type:            ColliderType::Cuboid,
            collision_damage:         0.,
            collision_layers:         CollisionLayers::default(),
            gravity_scale:            0.,
            health:                   0.,
            linear_damping:           None,
            locked_axes:              LockedAxes::new().lock_translation_z(),
            mass:                     1.,
            render_layer:             RenderLayer::Game,
            restitution:              0.1,
            restitution_combine_rule: CoefficientCombine::Max,
            rigid_body:               RigidBody::Dynamic,
            scene:                    Handle::default(),
            spawn_timer_seconds:      None,
            transform:                Transform::default(),
            spawn_timer:              None,
        }
    }
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

/// ensures that the game camera can see the spawned actor
fn propagate_render_layers_on_spawn(
    add: On<Add, Children>,
    q_parents: ActorRenderLayersQuery,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    // Only process if this entity has one of our actor marker components (scene
    // children added to actor parent)
    if let Ok(parent_layers) = q_parents.get(add.entity) {
        // Recursively propagate to all descendants
        propagate_to_descendants(add.entity, parent_layers, &children_query, &mut commands);
    }
}

fn propagate_to_descendants(
    entity: Entity,
    parent_layers: &RenderLayers,
    children_query: &Query<&Children>,
    commands: &mut Commands,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            commands.entity(child).insert(parent_layers.clone());

            // Recursively propagate to grandchildren
            propagate_to_descendants(child, parent_layers, children_query, commands);
        }
    }
}

// Public constants for physics configuration (used by missile.rs, spaceship.rs, nateroid.rs)
pub const LOCKED_AXES_2D: LockedAxes = LockedAxes::new().lock_translation_z();
pub const LOCKED_AXES_SPACESHIP: LockedAxes = LockedAxes::new()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_translation_z();

fn initialize_actor_configs(
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

    let spaceship_config = initialize_actor_config(
        SpaceshipConfig::default().0,
        &scenes,
        &meshes,
        &scene_assets.spaceship,
    );
    commands.insert_resource(SpaceshipConfig(spaceship_config));
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
    let adjusted_aabb = aabb.scale(BLENDER_SCALE);

    // Calculate the size based on the adjusted AABB
    let size = adjusted_aabb.size();

    let collider = match config.collider_type {
        ColliderType::Ball => {
            let radius = size.length() / 3.;
            Collider::sphere(radius)
        },
        ColliderType::Cuboid => Collider::cuboid(size.x, size.y, size.z),
    };

    config.aabb = adjusted_aabb;
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
