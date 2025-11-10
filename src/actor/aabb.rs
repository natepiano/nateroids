use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use super::spaceship::SpaceshipSpawnBuffer;
use crate::camera::RenderLayer;
use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;
use crate::traits::TransformExt;

pub struct AabbPlugin;
impl Plugin for AabbPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<AabbGizmo>()
            .add_systems(Startup, init_aabb_gizmo_config)
            .add_systems(
                Update,
                draw_aabb_system.run_if(toggle_active(false, GlobalAction::AABBs)),
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

fn draw_aabb_system(
    mut gizmos: Gizmos<AabbGizmo>,
    normal_aabbs: Query<(&Transform, &Aabb), Without<SpaceshipSpawnBuffer>>,
    buffer_aabbs: Query<(&GlobalTransform, &Aabb), With<SpaceshipSpawnBuffer>>,
) {
    // Draw normal AABBs in green
    for (transform, aabb) in normal_aabbs.iter() {
        let center = transform.transform_point(aabb.center());

        gizmos.cuboid(
            Transform::from_trs(center, transform.rotation, aabb.size() * transform.scale),
            Color::from(tailwind::GREEN_800),
        );
    }

    // Draw spaceship spawn buffer AABBs in yellow
    for (global_transform, aabb) in buffer_aabbs.iter() {
        let center = global_transform.transform_point(aabb.center());

        gizmos.cuboid(
            Transform::from_trs(
                center,
                global_transform.rotation(),
                aabb.size() * global_transform.scale(),
            ),
            Color::from(tailwind::YELLOW_600),
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

// todo: #bevyqestion - attempt to try to draw what rapier is drawing but
// couldn't get       it to draw the same aabb that rapier actually draws - the
// issue is that       for cuboids, rapier is off by some pixels whereas,
// visually, my aabb is perfectly aligned       the question is why
// fn debug_spaceship(
//     query: Query<(Entity, &Transform, &Aabb), With<Spaceship>>,
//     rapier_context: Res<RapierContext>,
//     mut gizmos: Gizmos,
// ) {
//     for (entity, transform, your_aabb) in query.iter() {
//         // Draw your calculated AABB
//         let your_center = transform.transform_point(your_aabb.center());
//         gizmos.cuboid(
//             Transform::from_translation(your_center)
//                 .with_scale(your_aabb.half_extents() * 2.0 * transform.scale)
//                 .with_rotation(transform.rotation),
//             Color::from(tailwind::GREEN_800).with_alpha(0.3),
//         );
//
//         // Get the collider from the entity and draw Rapier's AABB
//         if let Some(collider_handle) =
// rapier_context.entity2collider().get(&entity) {             if let
// Some(collider) = rapier_context.colliders.get(*collider_handle) {
// let rapier_aabb = collider.compute_aabb();
//
//                 // Convert Rapier's AABB to Bevy types
//                 let aabb_half_extents = Vec3::new(
//                     rapier_aabb.half_extents().x,
//                     rapier_aabb.half_extents().y,
//                     rapier_aabb.half_extents().z
//                 );
//
//                 // Apply initial correction to align with your coordinate
// system                 let correction_z =
// Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2);                 let
// correction_y = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
//
//                 let rotation =  transform.rotation; // correction_z *
// transform.rotation * correction_y;
//
//                 // Draw Rapier's AABB
//                 gizmos.cuboid(
//                     Transform::from_translation(transform.translation)
//                         .with_rotation(rotation)
//
// .with_scale(Vec3::new(aabb_half_extents.y,aabb_half_extents.z,
// aabb_half_extents.x ) * 2.0 * transform.scale),
// Color::from(tailwind::RED_800).with_alpha(0.3),                 );
//
//                 println!("your_aabb.half_extents() {}, {}, {}, rapier
// half_extents {}, {}, {}", your_aabb.half_extents().x,
// your_aabb.half_extents().y, your_aabb.half_extents().z,
// aabb_half_extents.x, aabb_half_extents.y, aabb_half_extents.z)             }
//         }
//     }
// }
