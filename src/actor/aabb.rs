use avian3d::prelude::Collider;
use avian3d::prelude::RigidBody;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::VisibilitySystems;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use super::actor_settings::ColliderType;
use crate::camera::RenderLayer;
use crate::input::AabbsSwitch;
use crate::input::InspectAabbSwitch;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;
use crate::traits::TransformExt;

event!(AabbInspectorEvent);
event!(AabbsEvent);

pub(super) struct AabbPlugin;
impl Plugin for AabbPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<AabbGizmo>()
            .init_resource::<AabbSettings>()
            .add_plugins(
                ResourceInspectorPlugin::<AabbSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectAabb)),
            )
            .add_systems(
                Update,
                apply_aabb_settings.run_if(resource_changed::<AabbSettings>),
            )
            .add_systems(
                Update,
                draw_aabb_system.run_if(switches::is_switch_on(Switch::ShowAabbs)),
            )
            .add_systems(
                PostUpdate,
                compute_actor_aabb.after(VisibilitySystems::CalculateBounds),
            );
        bind_action_switch!(
            app,
            InspectAabbSwitch,
            AabbInspectorEvent,
            Switch::InspectAabb
        );
        bind_action_switch!(app, AabbsSwitch, AabbsEvent, Switch::ShowAabbs);
    }
}

/// Inserted at spawn time to defer `Aabb`, `Collider`, and `RigidBody`
/// creation until Bevy has computed `Aabb` on child mesh entities via
/// `calculate_bounds`.
#[derive(Component)]
pub(super) struct PendingCollider {
    pub collider_type: ColliderType,
    pub margin:        f32,
    pub rigid_body:    RigidBody,
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct AabbGizmo {}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct AabbSettings {
    color:      Color,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    line_width: f32,
}

impl Default for AabbSettings {
    fn default() -> Self {
        Self {
            color:      Color::from(tailwind::GREEN_800),
            line_width: 1.0,
        }
    }
}

fn apply_aabb_settings(mut config_store: ResMut<GizmoConfigStore>, config: Res<AabbSettings>) {
    let (gizmo_config, _) = config_store.config_mut::<AabbGizmo>();
    gizmo_config.line.width = config.line_width;
    gizmo_config.render_layers = RenderLayer::Game.layers();
}

/// Returns the full size of the `Aabb` as a `Vec3`.
fn size(aabb: &Aabb) -> Vec3 { Vec3::from(aabb.half_extents * 2.0) }

/// Returns the largest axis dimension of the `Aabb`.
pub fn max_dimension(aabb: &Aabb) -> f32 {
    let he = aabb.half_extents;
    he.x.max(he.y).max(he.z) * 2.0
}

/// Draws AABB gizmos for root actor entities only. `Without<ChildOf>` excludes
/// child mesh entities which also have Bevy-computed `Aabb` components.
fn draw_aabb_system(
    mut gizmos: Gizmos<AabbGizmo>,
    aabbs: Query<(&Transform, &Aabb), Without<ChildOf>>,
    settings: Res<AabbSettings>,
) {
    for (transform, aabb) in aabbs.iter() {
        let center = transform.transform_point(Vec3::from(aabb.center));

        gizmos.cube(
            Transform::from_trs(center, transform.rotation, size(aabb) * transform.scale),
            settings.color,
        );
    }
}

/// Runs after `CalculateBounds` to combine Bevy's per-mesh `Aabb` components
/// into a single root-local `Aabb` on actor entities, then creates the
/// `Collider` and `RigidBody` together.
fn compute_actor_aabb(
    mut commands: Commands,
    actors: Query<(Entity, &PendingCollider, &GlobalTransform), Without<Aabb>>,
    children_query: Query<&Children>,
    child_aabbs: Query<(&GlobalTransform, &Aabb), With<Mesh3d>>,
) {
    for (entity, pending, root_global) in &actors {
        let root_inverse = root_global.affine().inverse();

        let mut all_points = Vec::new();

        for descendant in children_query.iter_descendants(entity) {
            if let Ok((child_global, child_aabb)) = child_aabbs.get(descendant) {
                let child_to_root = root_inverse * child_global.affine();
                for corner in aabb_corners(child_aabb) {
                    all_points.push(child_to_root.transform_point3(corner));
                }
            }
        }

        if all_points.is_empty() {
            continue;
        }

        let Some(aabb) = Aabb::enclosing(all_points) else {
            continue;
        };
        let aabb_size = size(&aabb);

        let collider = match pending.collider_type {
            ColliderType::Ball => {
                let radius = aabb_size.length() * pending.margin;
                Collider::sphere(radius)
            },
            ColliderType::Cuboid => Collider::cuboid(
                aabb_size.x * pending.margin,
                aabb_size.y * pending.margin,
                aabb_size.z * pending.margin,
            ),
        };

        commands
            .entity(entity)
            .insert((aabb, collider, pending.rigid_body))
            .remove::<PendingCollider>();
    }
}

/// Returns the 8 corner points of an `Aabb`.
fn aabb_corners(aabb: &Aabb) -> [Vec3; 8] {
    let min = Vec3::from(aabb.min());
    let max = Vec3::from(aabb.max());
    [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ]
}
