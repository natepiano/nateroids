use crate::asset_loader::{
    AssetsState,
    SceneAssets,
};
use bevy::{
    prelude::*,
    render::mesh::{
        Mesh,
        VertexAttributeValues,
    },
    scene::Scene,
};
use bevy_rapier3d::prelude::{
    CoefficientCombineRule,
    Collider,
    ColliderMassProperties,
    ColliderMassProperties::Mass,
    Restitution,
};
use rand::Rng;
use std::{
    f32::consts::PI,
    ops::Range,
};

// todo: #bevyquestion - where should this go
const BLENDER_SCALE: f32 = 100.;

pub struct ColliderConfigPlugin;
impl Plugin for ColliderConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetsState>()
            .add_systems(OnEnter(AssetsState::Loaded), initialize_configuration);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColliderType {
    Ball,
    Cuboid,
}

#[derive(Debug, Clone, Resource)]
struct InitialColliderConfig {
    missile:   InitialColliderConstant,
    nateroid:  InitialColliderConstant,
    spaceship: InitialColliderConstant,
}

#[derive(Debug, Clone)]
struct InitialColliderConstant {
    acceleration:        Option<f32>,
    angvel:              Option<f32>,
    collider_type:       ColliderType,
    damage:              f32,
    health:              f32,
    mass:                ColliderMassProperties,
    name:                &'static str,
    restitution:         f32,
    rotation_speed:      f32,
    scalar:              f32,
    spawn_point:         Vec3,
    spawn_timer_seconds: Option<f32>,
    spawnable:           bool,
    velocity:            f32,
}

impl Default for InitialColliderConfig {
    fn default() -> Self {
        Self {
            missile:   InitialColliderConstant {
                acceleration:        None,
                angvel:              None,
                collider_type:       ColliderType::Cuboid,
                damage:              50.,
                health:              1.,
                mass:                Mass(0.001),
                name:                "missile",
                restitution:         0.,
                rotation_speed:      0.,
                scalar:              2.5,
                spawn_point:         Vec3::new(0.5, 0., 0.),
                spawn_timer_seconds: Some(1.0 / 20.0),
                spawnable:           true,
                velocity:            85.,
            },
            nateroid:  InitialColliderConstant {
                acceleration:        None,
                angvel:              Some(4.),
                collider_type:       ColliderType::Ball,
                damage:              10.,
                health:              200.,
                mass:                Mass(1.0),
                name:                "nateroid",
                restitution:         1.0,
                rotation_speed:      0.,
                scalar:              1.,
                spawn_point:         Vec3::ZERO,
                spawn_timer_seconds: Some(2.),
                spawnable:           true,
                velocity:            30.,
            },
            spaceship: InitialColliderConstant {
                acceleration:        Some(60.),
                angvel:              None,
                collider_type:       ColliderType::Cuboid,
                damage:              50.,
                health:              500.,
                mass:                Mass(3.0),
                name:                "spaceship",
                restitution:         0.3,
                rotation_speed:      5.,
                scalar:              0.8,
                spawn_point:         Vec3::new(0.0, -20.0, 0.0),
                spawn_timer_seconds: None,
                spawnable:           true,
                velocity:            80.,
            },
        }
    }
}

impl InitialColliderConstant {
    fn initialize(&self, aabb: Aabb) -> ColliderConstant {
        let original_aabb = aabb;
        let adjusted_aabb = original_aabb.scale(BLENDER_SCALE);

        // Calculate the size based on the adjusted AABB
        let size = adjusted_aabb.size();
        let half_extents = adjusted_aabb.half_extents();

        let collider = match self.collider_type {
            ColliderType::Ball => {
                let radius = size.length() / 3.;
                Collider::ball(radius)
            },
            ColliderType::Cuboid => {
                Collider::cuboid(half_extents.x, half_extents.y, half_extents.z)
            },
        };

        let spawn_timer = self
            .spawn_timer_seconds
            .map(|seconds| Timer::from_seconds(seconds, TimerMode::Repeating));

        let restitution = Restitution {
            coefficient:  self.restitution,
            combine_rule: CoefficientCombineRule::Min,
        };

        ColliderConstant {
            aabb: adjusted_aabb,
            acceleration: self.acceleration,
            angular_velocity: self.angvel,
            collider,
            damage: self.damage,
            health: self.health,
            mass: self.mass,
            name: self.name.to_string(),
            restitution,
            rotation_speed: self.rotation_speed,
            scalar: self.scalar,
            spawn_point: self.spawn_point,
            spawn_timer,
            spawnable: self.spawnable,
            velocity: self.velocity,
        }
    }
}
#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct ColliderConfig {
    pub missile:   ColliderConstant,
    pub nateroid:  ColliderConstant,
    pub spaceship: ColliderConstant,
}

#[derive(Debug, Clone, Reflect, Resource)]
#[reflect(Resource)]
pub struct ColliderConstant {
    pub aabb:             Aabb,
    pub acceleration:     Option<f32>,
    pub angular_velocity: Option<f32>,
    #[reflect(ignore)]
    pub collider:         Collider,
    pub damage:           f32,
    pub health:           f32,
    pub mass:             ColliderMassProperties,
    pub name:             String,
    #[reflect(ignore)]
    pub restitution:      Restitution,
    pub rotation_speed:   f32,
    pub scalar:           f32,
    pub spawn_point:      Vec3,
    #[reflect(ignore)]
    pub spawn_timer:      Option<Timer>,
    pub spawnable:        bool,
    pub velocity:         f32,
}

impl ColliderConstant {
    pub fn get_forward_spawn_point(
        &self,
        spaceship_transform: Transform,
        spaceship_aabb: &Aabb,
    ) -> Vec3 {
        // Step 1: Determine the forward vector of the box in world space
        let forward = -spaceship_transform.forward();

        // Step 2: Get the half extents of the AABB
        let half_extents = spaceship_aabb.half_extents();

        // Step 3: Transform the half extents to world space
        let world_half_extents =
            spaceship_transform.rotation * (half_extents * spaceship_transform.scale);

        // Step 4: Project the world half extents onto the forward vector
        let forward_extent = forward.dot(world_half_extents);

        // Step 5: Compute the point on the edge of the box in the forward direction + a
        // buffer from the missile         we're overloading the spawn_point
        // from the missile as it is not otherwise used
        spaceship_transform.translation + forward * (forward_extent + self.spawn_point.length())
    }

    // todo: #rustquestion - i wanted to centralize construction of collider params
    //                      these are specific to the nateroid and just put here to
    //                      beautify the spawn code for nateroid
    //                      not sure i love the choices about storing the limits
    //                      as an f32 and then constructing ranges here
    //                      also - it's the only use of angvel out of the 3
    // colliders                      should this be an Option<f32>?
    pub fn random_angular_velocity(&self) -> Vec3 {
        if let Some(angvel) = self.angular_velocity {
            random_vec3(-angvel..angvel, -angvel..angvel, -angvel..angvel)
        } else {
            Vec3::ZERO
        }
    }

    pub fn random_velocity(&self) -> Vec3 {
        random_vec3(
            -self.velocity..self.velocity,
            -self.velocity..self.velocity,
            //todo: #handle3d
            0.0..0.0,
        )
    }

    pub fn random_rotation() -> Quat {
        const ROTATION_RANGE: Range<f32> = 0.0..2.0 * PI;

        let mut rng = rand::thread_rng();
        let x_angle = rng.gen_range(ROTATION_RANGE);
        let y_angle = rng.gen_range(ROTATION_RANGE);
        let z_angle = rng.gen_range(ROTATION_RANGE);

        Quat::from_euler(EulerRot::XYZ, x_angle, y_angle, z_angle)
    }
}

fn initialize_configuration(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    scenes: Res<Assets<Scene>>,
    scene_assets: Res<SceneAssets>,
) {
    let initial_config = InitialColliderConfig::default();

    let collider_config = ColliderConfig {
        missile:   initial_config.missile.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.missile,
        )),
        nateroid:  initial_config.nateroid.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.nateroid,
        )),
        spaceship: initial_config.spaceship.initialize(get_scene_aabb(
            &scenes,
            &meshes,
            &scene_assets.spaceship,
        )),
    };

    commands.insert_resource(collider_config);
}

#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn size(&self) -> Vec3 { self.max - self.min }

    pub fn center(&self) -> Vec3 { (self.min + self.max) / 2.0 }

    pub fn half_extents(&self) -> Vec3 { self.size() / 2.0 }

    pub fn max_dimension(&self) -> f32 {
        let size = self.size();
        size.x.max(size.y).max(size.z)
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self {
            min: self.min * scale,
            max: self.max * scale,
        }
    }
}

fn get_scene_aabb(scenes: &Assets<Scene>, meshes: &Assets<Mesh>, handle: &Handle<Scene>) -> Aabb {
    if let Some(scene) = scenes.get(handle) {
        let mut aabb = None;
        for entity in scene.world.iter_entities() {
            if let Some(mesh_handle) = entity.get::<Handle<Mesh>>() {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    let mesh_aabb = get_mesh_aabb(mesh);
                    aabb = Some(match aabb {
                        Some(existing) => combine_aabb(existing, mesh_aabb),
                        None => mesh_aabb,
                    });
                }
            }
        }
        aabb.unwrap_or(Aabb {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        })
    } else {
        Aabb {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        }
    }
}

fn get_mesh_aabb(mesh: &Mesh) -> Aabb {
    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for position in positions.iter() {
            min = min.min(Vec3::from(*position));
            max = max.max(Vec3::from(*position));
        }
        Aabb { min, max }
    } else {
        // Default to a unit cube if no vertex data is found
        Aabb {
            min: Vec3::splat(-0.5),
            max: Vec3::splat(0.5),
        }
    }
}

fn combine_aabb(a: Aabb, b: Aabb) -> Aabb {
    Aabb {
        min: a.min.min(b.min),
        max: a.max.max(b.max),
    }
}

fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = if range_x.start < range_x.end {
        rng.gen_range(range_x)
    } else {
        0.0
    };
    let y = if range_y.start < range_y.end {
        rng.gen_range(range_y)
    } else {
        0.0
    };
    let z = if range_z.start < range_z.end {
        rng.gen_range(range_z)
    } else {
        0.0
    };

    Vec3::new(x, y, z)
}
