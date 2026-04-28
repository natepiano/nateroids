mod aabb;
mod actor_settings;
mod collision_detection;
mod constants;
mod flame_gizmo;
mod game_layer;
mod missile;
mod nateroid;
mod spaceship;
mod spaceship_control;
mod teleport;

use aabb::AabbPlugin;
pub(crate) use aabb::max_dimension as aabb_max_dimension;
use actor_settings::ActorSettingsPlugin;
pub(crate) use actor_settings::Health;
use bevy::prelude::*;
use collision_detection::CollisionDetectionPlugin;
pub(crate) use constants::NATEROID_DEATH_ALPHA_STEP;
use flame_gizmo::FlameGizmoPlugin;
use missile::MissilePlugin;
pub(crate) use nateroid::Deaderoid;
pub(crate) use nateroid::DeathCorner;
pub(crate) use nateroid::Nateroid;
pub(crate) use nateroid::NateroidDeathMaterials;
use nateroid::NateroidPlugin;
pub(crate) use nateroid::NateroidSettings;
pub(crate) use spaceship::Spaceship;
use spaceship::SpaceshipPlugin;
use spaceship_control::SpaceshipControlPlugin;
use teleport::TeleportPlugin;
pub(crate) use teleport::TeleportStatus;
pub(crate) use teleport::Teleporter;

pub(crate) struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AabbPlugin)
            .add_plugins(ActorSettingsPlugin)
            .add_plugins(CollisionDetectionPlugin)
            .add_plugins(FlameGizmoPlugin)
            .add_plugins(MissilePlugin)
            .add_plugins(NateroidPlugin)
            .add_plugins(SpaceshipPlugin)
            .add_plugins(SpaceshipControlPlugin)
            .add_plugins(TeleportPlugin);
    }
}
