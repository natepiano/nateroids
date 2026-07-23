mod constants;
mod death_materials;
mod settings;
mod spawn;

use avian3d::prelude::*;
use bevy::prelude::*;
pub(crate) use death_materials::NateroidDeathMaterials;
pub(super) use death_materials::initialize_materials;
pub(crate) use settings::DeathCorner;
pub(crate) use settings::NateroidSettings;

use super::Teleporter;
use super::constants::LOCKED_AXES_2D;
use super::spawn_stats::NateroidSpawnStats;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;

pub(super) struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NateroidSpawnStats>().add_systems(
            Update,
            (
                death_materials::debug_mesh_components,
                spawn::spawn_nateroid.in_set(InGameSet::EntityUpdates),
            ),
        );
    }
}

#[derive(Component, Reflect, Debug, Default, Clone)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    LockedAxes = LOCKED_AXES_2D
)]
pub(crate) struct Nateroid;
