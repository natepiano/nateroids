use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::traits::TransformExt;

pub struct AabbPlugin;
impl Plugin for AabbPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<AabbGizmo>()
            .add_systems(Startup, init_aabb_gizmo_config)
            .add_systems(
                Update,
                draw_aabb_system.run_if(toggle_active(false, GameAction::AABBs)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct AabbGizmo {}

fn init_aabb_gizmo_config(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<AabbGizmo>();
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

#[derive(Component, Debug, Clone, Reflect, Default)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn size(&self) -> Vec3 { self.max - self.min }

    pub fn center(&self) -> Vec3 { (self.min + self.max) / 2.0 }

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

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn transform(&self, position: Vec3, scale: Vec3) -> Self {
        Self {
            min: (self.min * scale) + position,
            max: (self.max * scale) + position,
        }
    }
}

fn draw_aabb_system(mut gizmos: Gizmos<AabbGizmo>, aabbs: Query<(&Transform, &Aabb)>) {
    // Draw all AABBs in green
    for (transform, aabb) in aabbs.iter() {
        let center = transform.transform_point(aabb.center());

        gizmos.cuboid(
            Transform::from_trs(center, transform.rotation, aabb.size() * transform.scale),
            Color::from(tailwind::GREEN_800),
        );
    }
}

pub fn get_scene_aabb(
    scenes: &Assets<Scene>,
    meshes: &Assets<Mesh>,
    handle: &Handle<Scene>,
) -> Aabb {
    if let Some(scene) = scenes.get(handle) {
        let mut aabb = None;
        if let Some(mut query_state) = scene.world.try_query::<EntityRef>() {
            for entity in query_state.iter(&scene.world) {
                if let Some(mesh_handle) = entity.get::<Mesh3d>()
                    && let Some(mesh) = meshes.get(mesh_handle)
                {
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
    if let Some(positions) = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|attr| attr.as_float3())
    {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for position in positions {
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
