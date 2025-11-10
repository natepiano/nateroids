use bevy::prelude::*;

use super::Aabb;
use super::Nateroid;
use super::SpaceshipSpawnBuffer;
use super::actor_template::NateroidConfig;
use super::spaceship::Spaceship;
use crate::schedule::InGameSet;

pub struct SpaceshipDiagnosticsPlugin;

impl Plugin for SpaceshipDiagnosticsPlugin {
    fn build(&self, app: &mut App) { app.add_observer(detect_close_nateroid_spawn); }
}

fn debug_spaceship_visibility(
    spaceship_query: Query<
        (
            Entity,
            &Transform,
            &InheritedVisibility,
            &ViewVisibility,
            &Visibility,
            Option<&Children>,
        ),
        With<Spaceship>,
    >,
    nateroid_query: Query<
        (Entity, &InheritedVisibility, &ViewVisibility, &Visibility),
        With<Nateroid>,
    >,
    children_query: Query<(
        Entity,
        &InheritedVisibility,
        &ViewVisibility,
        &Visibility,
        Option<&SpaceshipSpawnBuffer>,
    )>,
) {
    for (entity, transform, inherited_vis, view_vis, vis, children) in spaceship_query.iter() {
        error!(
            "Spaceship Entity {entity:?} at {:.2?}:\n  \
             Visibility: {vis:?}\n  \
             InheritedVisibility: {inherited_vis:?}\n  \
             ViewVisibility: {view_vis:?}",
            transform.translation
        );

        if let Some(children) = children {
            for child_entity in children.iter() {
                if let Ok((child_e, child_inherited, child_view, child_vis, is_buffer)) =
                    children_query.get(child_entity)
                {
                    if is_buffer.is_some() {
                        error!(
                            "  SpaceshipSpawnBuffer Child {child_e:?}:\n    \
                             Visibility: {child_vis:?}\n    \
                             InheritedVisibility: {child_inherited:?}\n    \
                             ViewVisibility: {child_view:?}"
                        );
                    }
                }
            }
        }
    }

    // Compare with a nateroid
    if let Some((entity, inherited_vis, view_vis, vis)) = nateroid_query.iter().next() {
        error!(
            "Nateroid Entity {entity:?} (for comparison):\n  \
             Visibility: {vis:?}\n  \
             InheritedVisibility: {inherited_vis:?}\n  \
             ViewVisibility: {view_vis:?}"
        );
    }
}

fn detect_close_nateroid_spawn(
    nateroid: On<Add, Nateroid>,
    nateroid_query: Query<&Transform>,
    nateroid_config: Res<NateroidConfig>,
    spawn_buffers: Query<(&GlobalTransform, &Aabb), With<SpaceshipSpawnBuffer>>,
) {
    let Ok(nateroid_transform) = nateroid_query.get(nateroid.entity) else {
        return; // Shouldn't happen, but guard against it
    };

    // Get nateroid's AABB in world space
    let nateroid_aabb = &nateroid_config.actor_config.aabb;
    let nateroid_world_aabb =
        nateroid_aabb.transform(nateroid_transform.translation, nateroid_transform.scale);

    // Check if nateroid intersects with any spawn buffer
    for (buffer_global_transform, buffer_aabb) in spawn_buffers.iter() {
        let buffer_world_aabb = buffer_aabb.transform(
            buffer_global_transform.translation(),
            buffer_global_transform.scale(),
        );

        if nateroid_world_aabb.intersects(&buffer_world_aabb) {
            error!(
                "🚨 NATEROID SPAWNED INSIDE SPAWN BUFFER 🚨\n\
                 Nateroid position: {:.2?}\n\
                 Buffer center: {:.2?}",
                nateroid_transform.translation,
                buffer_global_transform.translation()
            );
        }
    }
}
