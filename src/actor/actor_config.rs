use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_enhanced_input::action::events as input_events;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

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
use crate::input::MissileInspectorToggle;
use crate::input::NateroidInspectorToggle;
use crate::input::SpaceshipInspectorToggle;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

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
                    .run_if(switches::is_switch_on(Switch::InspectMissile)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<NateroidConfig>::default()
                    .run_if(switches::is_switch_on(Switch::InspectNateroid)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<SpaceshipConfig>::default()
                    .run_if(switches::is_switch_on(Switch::InspectSpaceship)),
            );
        app.add_observer(on_toggle_missile_inspector_input)
            .add_observer(on_toggle_nateroid_inspector_input)
            .add_observer(on_toggle_spaceship_inspector_input);
    }
}

#[derive(Reflect, InspectorOptions, Clone, Debug)]
#[reflect(InspectorOptions)]
pub struct ActorConfig {
    pub spawnable:                bool,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub angular_damping:          Option<f32>,
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

pub fn initialize_actor_configs(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let mut nateroid_defaults = NateroidConfig::default();
    initialize_actor_config(&mut nateroid_defaults.actor_config, &scene_assets.nateroid);
    commands.insert_resource(nateroid_defaults);

    let mut missile_defaults = MissileConfig::default();
    initialize_actor_config(&mut missile_defaults.actor_config, &scene_assets.missile);
    commands.insert_resource(missile_defaults);

    let mut spaceship_defaults = SpaceshipConfig::default();
    initialize_actor_config(
        &mut spaceship_defaults.actor_config,
        &scene_assets.spaceship,
    );
    commands.insert_resource(spaceship_defaults);
}

pub fn create_spawn_timer(spawn_timer_seconds: Option<f32>) -> Option<Timer> {
    spawn_timer_seconds.map(|seconds| Timer::from_seconds(seconds, TimerMode::Repeating))
}

fn initialize_actor_config(config: &mut ActorConfig, scene_handle: &Handle<Scene>) {
    config.spawn_timer = create_spawn_timer(config.spawn_timer_seconds);
    config.scene = scene_handle.clone();
}

/// use config values so inspectors can provide new defaults
pub fn insert_configured_components(
    commands: &mut Commands,
    config: &mut ActorConfig,
    actor_entity: Entity,
) {
    // Insert all components on the actor entity
    commands.entity(actor_entity).insert((
        aabb::PendingCollider {
            collider_type: config.collider_type.clone(),
            margin:        config.collider_margin,
        },
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
        config.render_layer.layers(),
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

fn on_toggle_missile_inspector_input(
    _trigger: On<input_events::Start<MissileInspectorToggle>>,
    mut switches: ResMut<Switches>,
) {
    switches.toggle_switch(Switch::InspectMissile);
}

fn on_toggle_nateroid_inspector_input(
    _trigger: On<input_events::Start<NateroidInspectorToggle>>,
    mut switches: ResMut<Switches>,
) {
    switches.toggle_switch(Switch::InspectNateroid);
}

fn on_toggle_spaceship_inspector_input(
    _trigger: On<input_events::Start<SpaceshipInspectorToggle>>,
    mut switches: ResMut<Switches>,
) {
    switches.toggle_switch(Switch::InspectSpaceship);
}
