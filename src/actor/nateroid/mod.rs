mod death_materials;
mod settings;
mod spawn;

use avian3d::prelude::*;
use bevy::prelude::*;
pub use death_materials::Deaderoid;
pub use death_materials::NateroidDeathMaterials;
pub use settings::DeathCorner;
pub use settings::NateroidSettings;
pub(super) use spawn::NateroidSpawnStats;

use super::Teleporter;
use super::actor_settings;
use super::constants::LOCKED_AXES_2D;
use crate::asset_loader::AssetsState;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;

pub(super) struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NateroidSpawnStats>()
            .add_systems(
                OnEnter(AssetsState::Loaded),
                death_materials::precompute_death_materials
                    .after(actor_settings::initialize_actors),
            )
            .add_observer(spawn::initialize_nateroid)
            .add_observer(death_materials::apply_nateroid_materials_to_children)
            .add_systems(
                Update,
                (
                    death_materials::debug_mesh_components,
                    spawn::spawn_nateroid.in_set(InGameSet::EntityUpdates),
                ),
            );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Nateroid;
