use bevy::asset::LoadState;
/// let's use just load assets once, amigos
use bevy::prelude::*;

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetsState>() // necessary to tell if they've finished loading
            .init_resource::<SceneAssets>()
            // make sure this loads before the spaceship uses it - right now that is
            // handled by running this PreStartup and spaceship in Startup
            .add_systems(PreStartup, load_assets)
            .add_systems(
                Update,
                (create_nateroid_material, check_asset_loading)
                    .run_if(in_state(AssetsState::Loading)),
            );
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AssetsState {
    #[default]
    Loading,
    Loaded,
}

// all the models are loaded via SceneBundle - the models
// can have multiple elements and scene makes all that possible
#[derive(Resource, Clone, Debug, Default)]
pub struct SceneAssets {
    pub missile:                 Handle<Scene>,
    pub nateroid:                Handle<Scene>,
    pub nateroid_donut_material: Option<Handle<StandardMaterial>>,
    pub nateroid_icing_material: Option<Handle<StandardMaterial>>,
    pub spaceship:               Handle<Scene>, // pub sphere: Handle<Scene>,
}

pub fn load_assets(
    //    mut commands: Commands,
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
) {
    *scene_assets = SceneAssets {
        missile:                 asset_server.load("models/Bullets Pickup.glb#Scene0"),
        nateroid:                asset_server.load("nateroid/nateroid.glb#Scene0"),
        nateroid_donut_material: None,
        nateroid_icing_material: None,
        spaceship:               asset_server.load("models/Spaceship.glb#Scene0"),
    };
}

/// Create custom PBR materials with baked textures for nateroid (donut and icing)
fn create_nateroid_material(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
) {
    if scene_assets.nateroid_donut_material.is_some() {
        return;
    }

    info!("Loading baked PBR textures for nateroid (donut and icing)...");

    // Load the donut texture files
    let donut_albedo: Handle<Image> =
        asset_server.load("nateroid/textures/nateroid_donut_albedo.png");
    let donut_normal: Handle<Image> =
        asset_server.load("nateroid/textures/nateroid_donut_normal.png");
    let donut_metallic_roughness: Handle<Image> =
        asset_server.load("nateroid/textures/nateroid_donut_metallic_roughness.png");
    let donut_ao: Handle<Image> = asset_server.load("nateroid/textures/nateroid_donut_ao.png");

    // Load the icing texture files
    let icing_albedo: Handle<Image> =
        asset_server.load("nateroid/textures/nateroid_icing_albedo.png");
    let icing_normal: Handle<Image> =
        asset_server.load("nateroid/textures/nateroid_icing_normal.png");
    let icing_metallic_roughness: Handle<Image> =
        asset_server.load("nateroid/textures/nateroid_icing_metallic_roughness.png");
    let icing_ao: Handle<Image> = asset_server.load("nateroid/textures/nateroid_icing_ao.png");

    // Create donut PBR material
    let donut_material = materials.add(StandardMaterial {
        base_color_texture:        Some(donut_albedo),
        normal_map_texture:        Some(donut_normal),
        metallic_roughness_texture: Some(donut_metallic_roughness),
        occlusion_texture:         Some(donut_ao),
        cull_mode:                 None,
        ..default()
    });

    // Create icing PBR material
    let icing_material = materials.add(StandardMaterial {
        base_color_texture:        Some(icing_albedo),
        normal_map_texture:        Some(icing_normal),
        metallic_roughness_texture: Some(icing_metallic_roughness),
        occlusion_texture:         Some(icing_ao),
        cull_mode:                 None,
        ..default()
    });

    scene_assets.nateroid_donut_material = Some(donut_material);
    scene_assets.nateroid_icing_material = Some(icing_material);
    info!("Nateroid PBR materials created for donut and icing with baked textures");
}


pub fn check_asset_loading(
    mut next_state: ResMut<NextState<AssetsState>>,
    asset_server: Res<AssetServer>,
    scene_assets: Res<SceneAssets>,
) {
    // Collect all asset IDs to check their load states
    let all_assets_loaded = [
        scene_assets.missile.id(),
        scene_assets.nateroid.id(),
        scene_assets.spaceship.id(),
    ]
    .iter()
    .all(|&id| matches!(asset_server.get_load_state(id), Some(LoadState::Loaded)));

    // Check that both nateroid materials have been created
    let materials_ready = scene_assets.nateroid_donut_material.is_some()
        && scene_assets.nateroid_icing_material.is_some();

    // Transition to the Loaded state if all assets are loaded
    if all_assets_loaded && materials_ready {
        info!("All assets loaded!");
        next_state.set(AssetsState::Loaded);
    }
}
