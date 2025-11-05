mod aabb;
mod actor_spawner;
mod actor_template;
mod collision_detection;
pub mod missile;
mod nateroid;
mod spaceship;
mod spaceship_control;
mod teleport;

use bevy::prelude::*;

pub use crate::actor::aabb::Aabb;
use crate::actor::aabb::AabbPlugin;
pub use crate::actor::aabb::get_scene_aabb;
use crate::actor::actor_spawner::ActorSpawner;
pub use crate::actor::actor_spawner::ColliderType;
pub use crate::actor::actor_spawner::Health;
use crate::actor::collision_detection::CollisionDetectionPlugin;
use crate::actor::missile::MissilePlugin;
use crate::actor::nateroid::NateroidPlugin;
use crate::actor::spaceship::SpaceshipPlugin;
use crate::actor::spaceship_control::SpaceshipControlPlugin;
use crate::actor::teleport::TeleportPlugin;
pub use crate::actor::teleport::Teleporter;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AabbPlugin)
            .add_plugins(ActorSpawner)
            .add_plugins(CollisionDetectionPlugin)
            .add_plugins(MissilePlugin)
            .add_plugins(NateroidPlugin)
            .add_plugins(SpaceshipPlugin)
            .add_plugins(SpaceshipControlPlugin)
            .add_plugins(TeleportPlugin);
    }
}
