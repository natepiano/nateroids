use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;
use hana_lading::AllSetsLoaded;
use hana_lading::AssetSetLoadFailed;
use hana_lading::DiskAssetLoader;
use hana_lading::DiskAssets;
use hana_lading::DiskAssetsPlugin;

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
        // `DiskAssetsPlugin` loads `SceneAssets` during `PreStartup` and tracks
        // recursive dependency completion, emitting `Loaded<SceneAssets>` and
        // `AllSetsLoaded` when every tracked handle resolves.
        app.add_plugins(DiskAssetsPlugin::<SceneAssets>::default())
            .add_observer(mark_startup_assets_ready)
            .add_observer(exit_on_load_failure);
    }
}

/// Marker resource inserted once every startup asset set has loaded. The splash
/// `Splash` → `InGame` transition (including the keypress skip) waits for it.
#[derive(Resource)]
pub(crate) struct StartupAssetsReady;

/// PBR texture handles for one nateroid mesh (donut or icing).
#[derive(Clone, Debug)]
pub(crate) struct NateroidTextures {
    albedo:             Handle<Image>,
    normal:             Handle<Image>,
    metallic_roughness: Handle<Image>,
    occlusion:          Handle<Image>,
}

impl NateroidTextures {
    /// Builds the `StandardMaterial` shared by the donut and icing meshes;
    /// `SceneAssets::nateroid_icing_material` layers reflectance on top.
    fn base_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color_texture: Some(self.albedo.clone()),
            normal_map_texture: Some(self.normal.clone()),
            metallic_roughness_texture: Some(self.metallic_roughness.clone()),
            occlusion_texture: Some(self.occlusion.clone()),
            // `StandardMaterial::metallic` and `StandardMaterial::perceptual_roughness`
            // use their texture channels without scalar attenuation.
            metallic: NATEROID_MATERIAL_TEXTURE_SCALAR,
            perceptual_roughness: NATEROID_MATERIAL_TEXTURE_SCALAR,
            cull_mode: None,
            ..default()
        }
    }
}

// `SceneAssets` stores GLTF `Handle<WorldAsset>` values whose scene graphs can
// spawn multiple child meshes from a single actor asset, plus the environment
// maps and nateroid PBR textures.
#[derive(Resource, Clone, Debug)]
pub(crate) struct SceneAssets {
    pub(crate) missile:                  Handle<WorldAsset>,
    pub(crate) nateroid:                 Handle<WorldAsset>,
    pub(crate) spaceship:                Handle<WorldAsset>,
    pub(crate) environment_diffuse_map:  Handle<Image>,
    pub(crate) environment_specular_map: Handle<Image>,
    nateroid_donut:                      NateroidTextures,
    nateroid_icing:                      NateroidTextures,
}

impl DiskAssets for SceneAssets {
    fn load(loader: &mut DiskAssetLoader<'_>) -> Self {
        Self {
            missile:                  loader.load(MISSILE_SCENE_ASSET_PATH),
            nateroid:                 loader.load(NATEROID_SCENE_ASSET_PATH),
            spaceship:                loader.load(SPACESHIP_SCENE_ASSET_PATH),
            environment_diffuse_map:  loader.load(ENVIRONMENT_DIFFUSE_MAP_ASSET_PATH),
            environment_specular_map: loader.load(ENVIRONMENT_SPECULAR_MAP_ASSET_PATH),
            nateroid_donut:           NateroidTextures {
                albedo:             loader.load(NATEROID_DONUT_ALBEDO_ASSET_PATH),
                normal:             loader.load(NATEROID_DONUT_NORMAL_ASSET_PATH),
                metallic_roughness: loader.load(NATEROID_DONUT_METALLIC_ROUGHNESS_ASSET_PATH),
                occlusion:          loader.load(NATEROID_DONUT_AO_ASSET_PATH),
            },
            nateroid_icing:           NateroidTextures {
                albedo:             loader.load(NATEROID_ICING_ALBEDO_ASSET_PATH),
                normal:             loader.load(NATEROID_ICING_NORMAL_ASSET_PATH),
                metallic_roughness: loader.load(NATEROID_ICING_METALLIC_ROUGHNESS_ASSET_PATH),
                occlusion:          loader.load(NATEROID_ICING_AO_ASSET_PATH),
            },
        }
    }
}

impl SceneAssets {
    /// Custom PBR material for the nateroid donut mesh.
    pub(crate) fn nateroid_donut_material(&self) -> StandardMaterial {
        self.nateroid_donut.base_material()
    }

    /// Custom PBR material for the nateroid icing mesh.
    pub(crate) fn nateroid_icing_material(&self) -> StandardMaterial {
        StandardMaterial {
            reflectance: NATEROID_MATERIAL_REFLECTANCE,
            ..self.nateroid_icing.base_material()
        }
    }
}

fn mark_startup_assets_ready(_loaded: On<AllSetsLoaded>, mut commands: Commands) {
    debug!("All startup asset sets loaded");
    commands.insert_resource(StartupAssetsReady);
}

/// Every tracked asset is gameplay-required, so a failed set exits the app.
/// `hana_lading` has already logged the failed path and error.
fn exit_on_load_failure(failed: On<AssetSetLoadFailed>, mut app_exit: MessageWriter<AppExit>) {
    error!("exiting: startup asset set {} failed", failed.set_name());
    app_exit.write(AppExit::error());
}
