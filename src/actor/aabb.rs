use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::playfield::Boundary;
use crate::traits::TransformExt;

pub struct AabbPlugin;
impl Plugin for AabbPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<AabbGizmo>()
            .add_systems(Startup, init_aabb_gizmo_config)
            .add_systems(
                Update,
                draw_aabb_system.run_if(toggle_active(false, GameAction::AABBs)),
            )
            .add_systems(
                Update,
                draw_screen_aligned_boundary_box
                    .run_if(toggle_active(false, GameAction::BoundaryBox)),
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
/// used to draw a yellow screen-aligned box around the boundary
/// used for troubleshooting camera movement logic
fn draw_screen_aligned_boundary_box(
    mut gizmos: Gizmos<AabbGizmo>,
    boundary: Res<Boundary>,
    camera_query: Query<(&Transform, &GlobalTransform, &Projection), With<PanOrbitCamera>>,
) {
    let Ok((cam_transform, cam_global, projection)) = camera_query.single() else {
        return;
    };

    let Projection::Perspective(_perspective) = projection else {
        return;
    };

    // Get boundary corners
    let grid_size = boundary.scale();
    let half_size = grid_size / 2.0;
    let corners = [
        Vec3::new(-half_size.x, -half_size.y, -half_size.z),
        Vec3::new(half_size.x, -half_size.y, -half_size.z),
        Vec3::new(-half_size.x, half_size.y, -half_size.z),
        Vec3::new(half_size.x, half_size.y, -half_size.z),
        Vec3::new(-half_size.x, -half_size.y, half_size.z),
        Vec3::new(half_size.x, -half_size.y, half_size.z),
        Vec3::new(-half_size.x, half_size.y, half_size.z),
        Vec3::new(half_size.x, half_size.y, half_size.z),
    ];

    // Get camera basis vectors
    let cam_rot = cam_global.rotation();
    let cam_forward = cam_rot * Vec3::NEG_Z; // Camera looks down -Z
    let cam_right = cam_rot * Vec3::X;
    let cam_up = cam_rot * Vec3::Y;
    let cam_pos = cam_transform.translation;

    // For each corner, compute its normalized screen-space position (accounting for perspective)
    // Screen position = (x/depth, y/depth)
    let mut min_norm_x = f32::INFINITY;
    let mut max_norm_x = f32::NEG_INFINITY;
    let mut min_norm_y = f32::INFINITY;
    let mut max_norm_y = f32::NEG_INFINITY;
    let mut avg_depth = 0.0;

    for corner in &corners {
        let relative = *corner - cam_pos;
        let depth = relative.dot(cam_forward).max(0.1); // Ensure positive depth
        let x = relative.dot(cam_right);
        let y = relative.dot(cam_up);

        // Normalized screen coordinates (perspective-correct)
        let norm_x = x / depth;
        let norm_y = y / depth;

        min_norm_x = min_norm_x.min(norm_x);
        max_norm_x = max_norm_x.max(norm_x);
        min_norm_y = min_norm_y.min(norm_y);
        max_norm_y = max_norm_y.max(norm_y);
        avg_depth += depth;
    }
    avg_depth /= 8.0;

    // Draw the rectangle at average depth, scaling the normalized coords back to world coords
    let draw_depth = avg_depth;
    let world_min_x = min_norm_x * draw_depth;
    let world_max_x = max_norm_x * draw_depth;
    let world_min_y = min_norm_y * draw_depth;
    let world_max_y = max_norm_y * draw_depth;

    // Create the 4 corners of the screen-aligned rectangle in world space
    let rect_corners_world = [
        cam_pos + cam_right * world_min_x + cam_up * world_min_y + cam_forward * draw_depth,
        cam_pos + cam_right * world_max_x + cam_up * world_min_y + cam_forward * draw_depth,
        cam_pos + cam_right * world_max_x + cam_up * world_max_y + cam_forward * draw_depth,
        cam_pos + cam_right * world_min_x + cam_up * world_max_y + cam_forward * draw_depth,
    ];

    // Draw the rectangle with thicker lines
    for i in 0..4 {
        let next = (i + 1) % 4;
        gizmos.line(
            rect_corners_world[i],
            rect_corners_world[next],
            Color::from(tailwind::YELLOW_400),
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
