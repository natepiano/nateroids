use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy_kana::ToF32;
use bevy_kana::ToUsize;

use super::Nateroid;
use super::NateroidSettings;
use super::constants::DONUT_MESH_NAME;
use super::constants::ICING_MESH_NAME;
use crate::actor::constants::NATEROID_DEATH_ALPHA_STEP;
use crate::asset_loader::SceneAssets;

/// Precomputed `Nateroid` death-animation materials: one faded donut+icing set
/// per transparency level. Fades the donut/icing materials the meshes actually
/// use (applied by `apply_nateroid_materials_to_children`), matched per mesh.
#[derive(Resource)]
pub(crate) struct NateroidDeathMaterials {
    levels: Vec<NateroidDeathLevel>,
}

/// Faded donut and icing materials for a single transparency level.
struct NateroidDeathLevel {
    donut: Handle<StandardMaterial>,
    icing: Handle<StandardMaterial>,
}

impl NateroidDeathLevel {
    const fn handle(&self, mesh: NateroidMesh) -> &Handle<StandardMaterial> {
        match mesh {
            NateroidMesh::Donut => &self.donut,
            NateroidMesh::Icing => &self.icing,
        }
    }
}

impl NateroidDeathMaterials {
    /// Number of precomputed transparency levels.
    pub const fn level_count(&self) -> usize { self.levels.len() }

    /// Faded material for the given alpha `level`, matched to `mesh_name`
    /// (donut/icing). Unknown mesh names fall back to the donut material.
    pub fn material_for(
        &self,
        level: usize,
        mesh_name: Option<&str>,
    ) -> Option<Handle<StandardMaterial>> {
        let level = self.levels.get(level)?;
        let mesh = mesh_name
            .and_then(NateroidMesh::classify)
            .unwrap_or(NateroidMesh::Donut);
        Some(level.handle(mesh).clone())
    }
}

#[derive(Clone, Copy)]
enum NateroidMesh {
    Donut,
    Icing,
}

impl NateroidMesh {
    fn classify(name: &str) -> Option<Self> {
        let name = name.to_lowercase();
        [
            (Self::Donut, DONUT_MESH_NAME),
            (Self::Icing, ICING_MESH_NAME),
        ]
        .into_iter()
        .find_map(|(mesh, mesh_name)| name.contains(mesh_name).then_some(mesh))
    }
}

/// System that applies custom materials to `Nateroid` mesh children (donut and icing)
pub(super) fn apply_nateroid_materials_to_children(
    children_added: On<Add, Children>,
    mut commands: Commands,
    nateroid_query: Query<(), With<Nateroid>>,
    mesh_query: Query<(Entity, Option<&Name>), With<Mesh3d>>,
    children_query: Query<&Children>,
    scene_assets: Res<SceneAssets>,
) {
    if nateroid_query.get(children_added.entity).is_err() {
        return;
    }

    let Some(donut_material) = &scene_assets.nateroid_donut_material else {
        return;
    };
    let Some(icing_material) = &scene_assets.nateroid_icing_material else {
        return;
    };

    let nateroid_entity = children_added.entity;
    debug!("Applying materials to nateroid {nateroid_entity:?} mesh children");

    let mut donut_count = 0;
    let mut icing_count = 0;

    for descendant in children_query.iter_descendants(nateroid_entity) {
        if let Ok((mesh_entity, name)) = mesh_query.get(descendant) {
            if let Some(name) = name {
                debug!("Found mesh with name: '{}'", name.as_str());
            } else {
                info!("Found mesh with no Name component");
            }

            let material = if let Some(name) = name {
                match NateroidMesh::classify(name.as_str()) {
                    Some(NateroidMesh::Donut) => {
                        debug!("  -> Matched as donut");
                        donut_count += 1;
                        donut_material.clone()
                    },
                    Some(NateroidMesh::Icing) => {
                        debug!("  -> Matched as icing");
                        icing_count += 1;
                        icing_material.clone()
                    },
                    None => {
                        info!("  -> Unknown mesh name, defaulting to donut material");
                        donut_count += 1;
                        donut_material.clone()
                    },
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
                let mesh_data = meshes.get(&mesh3d.0);
                let vertex_count = mesh_data.map_or(0, Mesh::count_vertices);

                debug!(
                    "Mesh entity {entity:?}: has_material={}, visible={:?}, vertices={vertex_count}, render_layers={render_layers:?}, scale={:?}, global_position={:?}",
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

/// Precomputes faded donut/icing materials for the death animation once assets
/// are loaded. Fades the custom materials the meshes use rather than the GLTF
/// originals (the GLTF scene's embedded `World` is no longer queryable in 0.19).
pub(super) fn precompute_death_materials(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nateroid_settings: Res<NateroidSettings>,
) {
    let Some(donut_handle) = scene_assets.nateroid_donut_material.clone() else {
        warn!("Nateroid donut material not created yet");
        return;
    };
    let Some(icing_handle) = scene_assets.nateroid_icing_material.clone() else {
        warn!("Nateroid icing material not created yet");
        return;
    };
    let (Some(donut_base), Some(icing_base)) = (
        materials.get(&donut_handle).cloned(),
        materials.get(&icing_handle).cloned(),
    ) else {
        warn!("Nateroid base materials not loaded yet");
        return;
    };

    let initial_alpha = nateroid_settings.initial_alpha;
    let target_alpha = nateroid_settings.target_alpha;
    // `NateroidSettings::initial_alpha`, `NateroidSettings::target_alpha`, and
    // `NATEROID_DEATH_ALPHA_STEP` bound `level_count` to roughly 30–40 entries.
    let level_count =
        ((initial_alpha - target_alpha) * (1.0 / NATEROID_DEATH_ALPHA_STEP)).to_usize() + 1;

    let mut levels = Vec::with_capacity(level_count);
    for level in 0..level_count {
        // `f32::mul_add` computes `initial_alpha - level * NATEROID_DEATH_ALPHA_STEP`.
        let alpha = level
            .to_f32()
            .mul_add(-NATEROID_DEATH_ALPHA_STEP, initial_alpha);
        levels.push(NateroidDeathLevel {
            donut: materials.add(faded_material(&donut_base, alpha)),
            icing: materials.add(faded_material(&icing_base, alpha)),
        });
    }

    let level_count = levels.len();
    commands.insert_resource(NateroidDeathMaterials { levels });
    debug!("Precomputed {level_count} nateroid death-material levels (donut + icing)");
}

/// Clones `base` into a translucent material at the given `alpha`.
fn faded_material(base: &StandardMaterial, alpha: f32) -> StandardMaterial {
    let mut material = base.clone();
    material.base_color.set_alpha(alpha);
    material.alpha_mode = AlphaMode::Blend;
    material
}
