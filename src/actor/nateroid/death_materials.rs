use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_kana::ToF32;
use bevy_kana::ToUsize;

use super::Nateroid;
use super::NateroidSettings;
use crate::actor::constants::NATEROID_DEATH_ALPHA_STEP;
use crate::asset_loader::SceneAssets;

#[derive(Component, Debug)]
pub struct Deaderoid {
    pub initial_scale:          Vec3,
    pub target_shrink:          f32,
    pub shrink_duration:        f32,
    pub elapsed_time:           f32,
    pub current_shrink:         f32,
    pub current_material_index: usize,
}

/// Precomputed materials for `Nateroid` death animation at different transparency levels
#[derive(Resource)]
pub struct NateroidDeathMaterials {
    pub materials: Vec<Vec<Handle<StandardMaterial>>>,
}

/// System that applies custom materials to `Nateroid` mesh children (donut and icing)
pub(super) fn apply_nateroid_materials_to_children(
    mut commands: Commands,
    nateroid_query: Query<Entity, (With<Nateroid>, Added<Children>)>,
    mesh_query: Query<(Entity, Option<&Name>), With<Mesh3d>>,
    children_query: Query<&Children>,
    scene_assets: Res<SceneAssets>,
) {
    let Some(donut_material) = &scene_assets.nateroid_donut_material else {
        return;
    };
    let Some(icing_material) = &scene_assets.nateroid_icing_material else {
        return;
    };

    for nateroid_entity in nateroid_query.iter() {
        debug!("Applying materials to nateroid {nateroid_entity:?} mesh children");

        let mut donut_count = 0;
        let mut icing_count = 0;

        // Iterate over all descendants to find mesh entities
        for descendant in children_query.iter_descendants(nateroid_entity) {
            if let Ok((mesh_entity, name)) = mesh_query.get(descendant) {
                // Debug: log the actual mesh name
                if let Some(name) = name {
                    debug!("Found mesh with name: '{}'", name.as_str());
                } else {
                    info!("Found mesh with no Name component");
                }

                // Match mesh name to appropriate material
                let material = if let Some(name) = name {
                    let name_str = name.as_str().to_lowercase();
                    if name_str.contains("donut") {
                        debug!("  -> Matched as donut");
                        donut_count += 1;
                        donut_material.clone()
                    } else if name_str.contains("icing") {
                        debug!("  -> Matched as icing");
                        icing_count += 1;
                        icing_material.clone()
                    } else {
                        info!("  -> Unknown mesh name, defaulting to donut material");
                        donut_count += 1;
                        donut_material.clone()
                    }
                } else {
                    info!("  -> No name, defaulting to donut material");
                    donut_count += 1;
                    donut_material.clone()
                };

                commands
                    .entity(mesh_entity)
                    .insert(MeshMaterial3d(material));
            }
        }

        debug!("Applied materials: {donut_count} donut, {icing_count} icing");
    }
}

/// Diagnostic system to check mesh entity components
pub(super) fn debug_mesh_components(
    nateroid_query: Query<Entity, With<Nateroid>>,
    mesh_query: Query<
        (
            Entity,
            &Mesh3d,
            Option<&MeshMaterial3d<StandardMaterial>>,
            Option<&ViewVisibility>,
            Option<&RenderLayers>,
            Option<&Transform>,
            Option<&GlobalTransform>,
        ),
        With<Mesh3d>,
    >,
    all_children_query: Query<&Children>,
    meshes: Res<Assets<Mesh>>,
) {
    for nateroid_entity in nateroid_query.iter() {
        for descendant in all_children_query.iter_descendants(nateroid_entity) {
            if let Ok((
                entity,
                mesh3d,
                material,
                visibility,
                render_layers,
                transform,
                global_transform,
            )) = mesh_query.get(descendant)
            {
                // Check if the mesh asset actually has data
                let mesh_data = meshes.get(&mesh3d.0);
                let vertex_count = mesh_data.map_or(0, Mesh::count_vertices);

                debug!(
                    "Mesh entity {entity:?}: has_material={}, visible={:?}, vertices={vertex_count}, render_layers={render_layers:?}, scale={:?}, global_pos={:?}",
                    material.is_some(),
                    visibility.copied().map(ViewVisibility::get),
                    transform.map(|t| t.scale),
                    global_transform.map(GlobalTransform::translation)
                );

                if vertex_count == 0 {
                    warn!("Mesh entity {:?} has ZERO vertices!", entity);
                }
            }
        }
    }
}

/// System that precomputes death materials when assets are loaded
pub(super) fn precompute_death_materials(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    scenes: Res<Assets<Scene>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nateroid_settings: Res<NateroidSettings>,
) {
    // Get the nateroid scene
    let Some(nateroid_scene) = scenes.get(&scene_assets.nateroid) else {
        warn!("Nateroid scene not loaded yet");
        return;
    };

    let initial_alpha = nateroid_settings.initial_alpha;
    let target_alpha = nateroid_settings.target_alpha;
    // Safe: alpha values are 0.0-1.0, result is small positive integer (~30-40)
    let num_levels =
        ((initial_alpha - target_alpha) * (1.0 / NATEROID_DEATH_ALPHA_STEP)).to_usize() + 1;

    // Collect material handles from the scene's world using try_query
    let mut material_handles = Vec::new();
    if let Some(mut query_state) = nateroid_scene
        .world
        .try_query::<&MeshMaterial3d<StandardMaterial>>()
    {
        for mesh_material in query_state.iter(&nateroid_scene.world) {
            material_handles.push(mesh_material.0.clone());
        }
    }

    if material_handles.is_empty() {
        warn!("No materials found in nateroid scene");
        return;
    }

    debug!(
        "Collected {} material handles from nateroid scene",
        material_handles.len()
    );

    // Precompute materials for each alpha level
    let mut precomputed_materials = Vec::with_capacity(num_levels);
    for level in 0..num_levels {
        // FMA optimization (faster + more precise): initial_alpha - (level as f32 * step)
        let alpha = level
            .to_f32()
            .mul_add(-NATEROID_DEATH_ALPHA_STEP, initial_alpha);
        let mut level_materials = Vec::with_capacity(material_handles.len());

        for material_handle in &material_handles {
            if let Some(original_material) = materials.get(material_handle) {
                let mut cloned_material = original_material.clone();
                cloned_material.base_color.set_alpha(alpha);
                cloned_material.alpha_mode = AlphaMode::Blend;
                level_materials.push(materials.add(cloned_material));
            }
        }

        precomputed_materials.push(level_materials);
    }

    let num_material_sets = precomputed_materials.len();
    let num_materials_per_set = material_handles.len();

    // Insert the resource
    commands.insert_resource(NateroidDeathMaterials {
        materials: precomputed_materials,
    });

    debug!(
        "Precomputed {num_material_sets} material sets with {num_materials_per_set} materials each"
    );
}
