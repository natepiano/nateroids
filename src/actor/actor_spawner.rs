use std::fmt;
use std::ops::Range;

use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use rand::Rng;

use super::Aabb;
use super::Missile;
use super::aabb;
use super::actor_template::MissileConfig;
use super::actor_template::NateroidConfig;
use super::actor_template::SpaceshipConfig;
use super::nateroid::Nateroid;
use super::spaceship::Spaceship;
use crate::asset_loader::AssetsState;
use crate::asset_loader::SceneAssets;
use crate::camera::RenderLayer;
use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;
use crate::playfield::Boundary;

// this is how far off we are from blender for the assets we're loading
// we need to get them scaled up to generate a usable aabb
const BLENDER_SCALE: f32 = 100.;

// call flow is to initialize the ensemble config which has the defaults
// for an actor - configure defaults in initial_actor_config.rs
pub struct ActorSpawner;

impl Plugin for ActorSpawner {
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
    //   Mesh,
}

#[derive(Reflect, Debug, Clone)]
pub enum SpawnPosition {
    Spaceship(Vec3),
    Nateroid { scale_factor: Vec3 },
    Missile { forward_distance_scalar: f32 },
}

#[derive(Reflect, Debug, Clone)]
pub enum VelocityBehavior {
    /// fixed
    Spaceship(Vec3),
    /// random
    Nateroid { linvel: f32, angvel: f32 },
    /// relative to parent
    Missile { base_velocity: f32 },
}

impl VelocityBehavior {
    pub fn calculate_velocity(
        &self,
        parent_linear_velocity: Option<&LinearVelocity>,
        parent_transform: Option<&Transform>,
    ) -> (LinearVelocity, AngularVelocity) {
        match self {
            VelocityBehavior::Spaceship(velocity) => {
                (LinearVelocity(*velocity), AngularVelocity::ZERO)
            },
            VelocityBehavior::Nateroid { linvel, angvel } => (
                LinearVelocity(random_vec3(-*linvel..*linvel, -*linvel..*linvel, 0.0..0.0)),
                AngularVelocity(random_vec3(
                    -*angvel..*angvel,
                    -*angvel..*angvel,
                    -*angvel..*angvel,
                )),
            ),
            VelocityBehavior::Missile { base_velocity } => {
                if let (Some(parent_linear_velocity), Some(parent_transform)) =
                    (parent_linear_velocity, parent_transform)
                {
                    let forward = -parent_transform.forward();
                    let mut velocity = forward * *base_velocity;
                    velocity += **parent_linear_velocity;
                    (LinearVelocity(velocity), AngularVelocity::ZERO)
                } else {
                    (LinearVelocity::ZERO, AngularVelocity::ZERO)
                }
            },
        }
    }
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct ActorConfig {
    pub spawnable: bool,
    #[reflect(ignore)]
    pub aabb: Aabb,
    #[reflect(ignore)]
    pub actor_kind: ActorKind,
    #[reflect(ignore)]
    pub collider: Collider,
    pub collider_type: ColliderType,
    pub collision_damage: f32,
    #[reflect(ignore)]
    pub collision_layers: CollisionLayers,
    pub gravity_scale: f32,
    pub health: f32,
    pub locked_axes: LockedAxes,
    #[inspector(min = 0.0, max = 20.0, display = NumberDisplay::Slider)]
    pub mass: f32,
    pub render_layer: RenderLayer,
    #[inspector(min = 0.1, max = 1.0, display = NumberDisplay::Slider)]
    pub restitution: f32,
    pub restitution_combine_rule: CoefficientCombine,
    pub rigid_body: RigidBody,
    pub rotation: Option<Quat>,
    #[inspector(min = 0.1, max = 10.0, display = NumberDisplay::Slider)]
    pub mesh_scalar: f32,
    #[reflect(ignore)]
    pub scene: Handle<Scene>,
    pub spawn_position: SpawnPosition,
    pub spawn_timer_seconds: Option<f32>,
    #[reflect(ignore)]
    pub spawn_timer: Option<Timer>,
    pub velocity_behavior: VelocityBehavior,
}

impl Default for ActorConfig {
    fn default() -> Self {
        Self {
            spawnable: true,
            actor_kind: ActorKind::default(),
            aabb: Aabb::default(),
            collider: Collider::cuboid(1., 1., 1.),
            collider_type: ColliderType::Cuboid,
            collision_damage: 0.,
            collision_layers: CollisionLayers::default(),
            gravity_scale: 0.,
            health: 0.,
            locked_axes: LockedAxes::new().lock_translation_z(),
            mass: 1.,
            render_layer: RenderLayer::Game,
            restitution: 1.,
            restitution_combine_rule: CoefficientCombine::Max,
            rigid_body: RigidBody::Dynamic,
            rotation: None,
            mesh_scalar: 1.,
            scene: Handle::default(),
            spawn_position: SpawnPosition::Spaceship(Vec3::ZERO),
            spawn_timer_seconds: None,
            spawn_timer: None,
            velocity_behavior: VelocityBehavior::Spaceship(Vec3::ZERO),
        }
    }
}

impl ActorConfig {
    pub fn calculate_spawn_transform(
        &self,
        parent: Option<&Transform>,
        boundary: Option<Res<Boundary>>,
    ) -> Transform {
        let transform = match &self.spawn_position {
            SpawnPosition::Spaceship(position) => Transform::from_translation(*position),

            SpawnPosition::Nateroid { scale_factor } => {
                let boundary = boundary
                    .as_ref()
                    .expect("Boundary is required for RandomWithinBounds spawn behavior");

                let bounds = Transform {
                    translation: boundary.transform.translation,
                    scale: boundary.transform.scale * *scale_factor,
                    ..default()
                };
                let position = get_random_position_within_bounds(&bounds);

                let mut transform = Transform::from_translation(position);

                transform.rotation = get_random_rotation();

                transform
            },

            SpawnPosition::Missile {
                forward_distance_scalar: distance,
            } => {
                if let Some(parent_transform) = parent {
                    let forward = -parent_transform.forward();

                    // determined the buffer by eyeballing it up close to just make it 'look right'
                    let spawn_position = parent_transform.translation + forward * (*distance);

                    Transform::from_translation(spawn_position)
                } else {
                    Transform::from_translation(Vec3::ZERO)
                }
            },
        };

        if let Some(rotation) = self.rotation {
            transform
                .with_rotation(rotation)
                .with_scale(Vec3::splat(self.mesh_scalar))
        } else {
            transform.with_scale(Vec3::splat(self.mesh_scalar))
        }
    }
}

// Combine rotations from optional parent with optional supplied rotation
// missiles need this to get oriented correctly
// both parent and actor_config.rotation are optional so we have to unpack both
// and use one, both or none
// extracted here for readability
pub fn apply_rotations(
    config: &ActorConfig,
    parent_transform: Option<&Transform>,
    transform: &mut Transform,
) {
    let final_rotation = parent_transform
        .map(|t| t.rotation)
        .map(|parent_rot| {
            config
                .rotation
                .map(|initial_rot| parent_rot * initial_rot)
                .unwrap_or(parent_rot)
        })
        .or(config.rotation);

    if let Some(rotation) = final_rotation {
        transform.rotation = rotation;
    }
}

fn get_random_position_within_bounds(bounds: &Transform) -> Vec3 {
    let mut rng = rand::rng();
    let half_scale = bounds.scale.abs() / 2.0; // Use absolute value to ensure positive scale
    let min = bounds.translation - half_scale;
    let max = bounds.translation + half_scale;

    Vec3::new(
        get_random_component(min.x, max.x, &mut rng),
        get_random_component(min.y, max.y, &mut rng),
        get_random_component(min.z, max.z, &mut rng),
    )
}

fn get_random_component(min: f32, max: f32, rng: &mut impl Rng) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        min // If the range is effectively zero, just return the min value
    } else {
        rng.random_range(min.min(max)..=min.max(max)) // Ensure min is always
        // less
        // than max
    }
}

fn get_random_rotation() -> Quat {
    let mut rng = rand::rng();
    Quat::from_euler(
        EulerRot::XYZ,
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
    )
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
pub enum ActorKind {
    #[default]
    Missile,
    Nateroid,
    Spaceship,
}

impl fmt::Display for ActorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorKind::Missile => write!(f, "Missile"),
            ActorKind::Nateroid => write!(f, "Nateroid"),
            ActorKind::Spaceship => write!(f, "Spaceship"),
        }
    }
}

// Public constants for physics configuration (used by missile.rs, spaceship.rs, nateroid.rs)
pub const ZERO_GRAVITY: GravityScale = GravityScale(0.0);
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
    let nateroid_config = initialize_actor_config(
        NateroidConfig::default().0,
        &scenes,
        &meshes,
        &scene_assets.nateroid,
    );
    commands.insert_resource(NateroidConfig(nateroid_config));

    let missile_config = initialize_actor_config(
        MissileConfig::default().0,
        &scenes,
        &meshes,
        &scene_assets.missile,
    );
    commands.insert_resource(MissileConfig(missile_config));

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
        // ColliderType::Mesh => {
        //     Collider::trimesh_from_mesh(meshes.get(&scene_handle).unwrap().mesh.clone())
        // },
    };

    config.aabb = adjusted_aabb;
    config.collider = collider;
    config.spawn_timer = create_spawn_timer(config.spawn_timer_seconds);
    config.scene = scene_handle.clone();
    config
}

pub fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rand::rng();
    let x = if range_x.start < range_x.end {
        rng.random_range(range_x)
    } else {
        0.0
    };
    let y = if range_y.start < range_y.end {
        rng.random_range(range_y)
    } else {
        0.0
    };
    let z = if range_z.start < range_z.end {
        rng.random_range(range_z)
    } else {
        0.0
    };

    Vec3::new(x, y, z)
}

pub fn spawn_actor<'a>(
    commands: &'a mut Commands,
    config: &ActorConfig,
    boundary: Option<Res<Boundary>>,
    parent: Option<(&Transform, &LinearVelocity, &Aabb)>,
) -> EntityCommands<'a> {
    // Extract parent components
    let parent_transform = parent.map(|(t, _, _)| t);
    let parent_velocity = parent.map(|(_, v, _)| v);

    // Calculate spawn transform
    let mut transform = config.calculate_spawn_transform(parent_transform, boundary);

    // Apply rotation logic using existing helper function
    // NOTE: This preserves current behavior where rotation application happens in
    // two phases:
    // 1. calculate_spawn_transform applies config.rotation (if present)
    // 2. apply_rotations may overwrite it when combining with parent rotation
    // For missiles: calculate_spawn_transform sets config rotation, then
    // apply_rotations overwrites with (spaceship_rotation * config_rotation).
    // The intermediate application is redundant but functionally correct - this
    // is how the current code works.
    apply_rotations(config, parent_transform, &mut transform);

    // Calculate velocities (from ActorBundle::new)
    let (linear_velocity, angular_velocity) = config
        .velocity_behavior
        .calculate_velocity(parent_velocity, parent_transform);

    // Spawn with marker component which brings required components automatically:
    // Transform, Teleporter, ActorPortals, CollisionEventsEnabled, RigidBody,
    // GravityScale, LockedAxes
    //
    // Note: When we provide components explicitly (like Transform), they override
    // the required component defaults
    let entity = match config.actor_kind {
        ActorKind::Missile => commands.spawn((
            Missile::new(0.0), // Placeholder - caller should insert proper Missile component
            config.actor_kind,
            transform,
            config.aabb.clone(),
            config.collider.clone(),
            CollisionDamage(config.collision_damage),
            config.collision_layers,
            Health(config.health),
            Restitution {
                coefficient: config.restitution,
                combine_rule: config.restitution_combine_rule,
            },
            Mass(config.mass),
            RenderLayers::from_layers(config.render_layer.layers()),
            SceneRoot(config.scene.clone()),
            linear_velocity,
            angular_velocity,
            Name::new("Missile"),
        )),
        ActorKind::Nateroid => commands.spawn((
            Nateroid,
            config.actor_kind,
            transform,
            config.aabb.clone(),
            config.collider.clone(),
            CollisionDamage(config.collision_damage),
            config.collision_layers,
            Health(config.health),
            Restitution {
                coefficient: config.restitution,
                combine_rule: config.restitution_combine_rule,
            },
            Mass(config.mass),
            RenderLayers::from_layers(config.render_layer.layers()),
            SceneRoot(config.scene.clone()),
            linear_velocity,
            angular_velocity,
            Name::new("Nateroid"),
        )),
        ActorKind::Spaceship => commands.spawn((
            Spaceship,
            config.actor_kind,
            transform,
            config.aabb.clone(),
            config.collider.clone(),
            CollisionDamage(config.collision_damage),
            config.collision_layers,
            Health(config.health),
            Restitution {
                coefficient: config.restitution,
                combine_rule: config.restitution_combine_rule,
            },
            Mass(config.mass),
            RenderLayers::from_layers(config.render_layer.layers()),
            SceneRoot(config.scene.clone()),
            linear_velocity,
            angular_velocity,
            Name::new("Spaceship"),
        )),
    }
    .id();

    commands.entity(entity)
}
