use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::constants::ENVIRONMENT_DIFFUSE_MAP_ASSET_PATH;
use crate::constants::ENVIRONMENT_SPECULAR_MAP_ASSET_PATH;
use crate::constants::MISSILE_SCENE_ASSET_PATH;
use crate::constants::NATEROID_DONUT_ALBEDO_ASSET_PATH;
use crate::constants::NATEROID_DONUT_AO_ASSET_PATH;
use crate::constants::NATEROID_DONUT_METALLIC_ROUGHNESS_ASSET_PATH;
use crate::constants::NATEROID_DONUT_NORMAL_ASSET_PATH;
use crate::constants::NATEROID_ICING_ALBEDO_ASSET_PATH;
use crate::constants::NATEROID_ICING_AO_ASSET_PATH;
use crate::constants::NATEROID_ICING_METALLIC_ROUGHNESS_ASSET_PATH;
use crate::constants::NATEROID_ICING_NORMAL_ASSET_PATH;
use crate::constants::NATEROID_MATERIAL_REFLECTANCE;
use crate::constants::NATEROID_MATERIAL_TEXTURE_SCALAR;
use crate::constants::NATEROID_SCENE_ASSET_PATH;
use crate::constants::SPACESHIP_SCENE_ASSET_PATH;

pub(crate) struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        // `AssetsState` reports when handles in `SceneAssets` have finished loading.
        app.init_state::<AssetsState>()
            .init_resource::<SceneAssets>()
            // Run `load_assets` in `PreStartup` before spaceship setup reads
            // `SceneAssets` during `Startup`.
            .add_systems(PreStartup, load_assets)
            .add_systems(
                Update,
                (create_nateroid_material, check_asset_loading)
                    .run_if(in_state(AssetsState::Loading)),
            );
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub(crate) enum AssetsState {
    #[default]
    Loading,
    Loaded,
}

// all the models are loaded via `SceneBundle` - the models
// can have multiple elements and scene makes all that possible
#[derive(Resource, Clone, Debug, Default)]
pub(crate) struct SceneAssets {
    pub(crate) missile:                  Handle<Scene>,
    pub(crate) nateroid:                 Handle<Scene>,
    pub(crate) nateroid_donut_material:  Option<Handle<StandardMaterial>>,
    pub(crate) nateroid_icing_material:  Option<Handle<StandardMaterial>>,
    pub(crate) spaceship:                Handle<Scene>,
    pub(crate) environment_diffuse_map:  Handle<Image>,
    pub(crate) environment_specular_map: Handle<Image>,
}

fn load_assets(
    //    mut commands: Commands,
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
) {
    *scene_assets = SceneAssets {
        missile:                  asset_server.load(MISSILE_SCENE_ASSET_PATH),
        nateroid:                 asset_server.load(NATEROID_SCENE_ASSET_PATH),
        nateroid_donut_material:  None,
        nateroid_icing_material:  None,
        spaceship:                asset_server.load(SPACESHIP_SCENE_ASSET_PATH),
        environment_diffuse_map:  asset_server.load(ENVIRONMENT_DIFFUSE_MAP_ASSET_PATH),
        environment_specular_map: asset_server.load(ENVIRONMENT_SPECULAR_MAP_ASSET_PATH),
    };
}

/// Create custom PBR materials with baked textures for `Nateroid` (donut and icing)
fn create_nateroid_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
) {
    if scene_assets.nateroid_donut_material.is_some() {
        return;
    }

    // Load the donut texture files
    let donut_albedo: Handle<Image> = asset_server.load(NATEROID_DONUT_ALBEDO_ASSET_PATH);
    let donut_normal: Handle<Image> = asset_server.load(NATEROID_DONUT_NORMAL_ASSET_PATH);
    let donut_metallic_roughness: Handle<Image> =
        asset_server.load(NATEROID_DONUT_METALLIC_ROUGHNESS_ASSET_PATH);
    let donut_ao: Handle<Image> = asset_server.load(NATEROID_DONUT_AO_ASSET_PATH);

    // Load the icing texture files
    let icing_albedo: Handle<Image> = asset_server.load(NATEROID_ICING_ALBEDO_ASSET_PATH);
    let icing_normal: Handle<Image> = asset_server.load(NATEROID_ICING_NORMAL_ASSET_PATH);
    let icing_metallic_roughness: Handle<Image> =
        asset_server.load(NATEROID_ICING_METALLIC_ROUGHNESS_ASSET_PATH);
    let icing_ao: Handle<Image> = asset_server.load(NATEROID_ICING_AO_ASSET_PATH);

    // Create donut PBR material
    let donut_material = materials.add(StandardMaterial {
        base_color_texture: Some(donut_albedo),
        normal_map_texture: Some(donut_normal),
        metallic_roughness_texture: Some(donut_metallic_roughness),
        occlusion_texture: Some(donut_ao),
        // Set scalars to 1.0 so texture values are used directly
        metallic: NATEROID_MATERIAL_TEXTURE_SCALAR,
        perceptual_roughness: NATEROID_MATERIAL_TEXTURE_SCALAR,
        cull_mode: None,
        ..default()
    });

    // Create icing PBR material
    let icing_material = materials.add(StandardMaterial {
        base_color_texture: Some(icing_albedo),
        normal_map_texture: Some(icing_normal),
        metallic_roughness_texture: Some(icing_metallic_roughness),
        occlusion_texture: Some(icing_ao),
        // Set scalars to 1.0 so texture values are used directly
        metallic: NATEROID_MATERIAL_TEXTURE_SCALAR,
        perceptual_roughness: NATEROID_MATERIAL_TEXTURE_SCALAR,
        reflectance: NATEROID_MATERIAL_REFLECTANCE,
        cull_mode: None,
        ..default()
    });

    scene_assets.nateroid_donut_material = Some(donut_material);
    scene_assets.nateroid_icing_material = Some(icing_material);
}

fn check_asset_loading(
    mut next_state: ResMut<NextState<AssetsState>>,
    asset_server: Res<AssetServer>,
    scene_assets: Res<SceneAssets>,
) {
    // Check scene assets
    let scenes_loaded = [
        scene_assets.missile.id(),
        scene_assets.nateroid.id(),
        scene_assets.spaceship.id(),
    ]
    .iter()
    .all(|&id| matches!(asset_server.get_load_state(id), Some(LoadState::Loaded)));

    // Check environment map images
    let environment_maps_loaded = [
        scene_assets.environment_diffuse_map.id(),
        scene_assets.environment_specular_map.id(),
    ]
    .iter()
    .all(|&id| matches!(asset_server.get_load_state(id), Some(LoadState::Loaded)));

    // Check that both nateroid materials have been created
    let materials_ready = scene_assets.nateroid_donut_material.is_some()
        && scene_assets.nateroid_icing_material.is_some();

    // Transition to the Loaded state if all assets are loaded (including environment maps)
    if scenes_loaded && environment_maps_loaded && materials_ready {
        debug!("All assets loaded (including environment maps)!");
        next_state.set(AssetsState::Loaded);
    }
}
