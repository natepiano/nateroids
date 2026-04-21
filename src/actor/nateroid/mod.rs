mod death_materials;
mod spawn;

use avian3d::prelude::*;
use bevy::prelude::*;
pub use death_materials::Deaderoid;
pub use death_materials::NateroidDeathMaterials;
pub(super) use spawn::NateroidSpawnStats;

use super::Teleporter;
use super::constants::LOCKED_AXES_2D;
use crate::asset_loader::AssetsState;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;
use super::actor_settings;

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
            .add_systems(
                Update,
                (
                    death_materials::apply_nateroid_materials_to_children,
                    death_materials::debug_mesh_components
                        .after(death_materials::apply_nateroid_materials_to_children),
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
