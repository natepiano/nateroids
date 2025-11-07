mod aabb;
mod actor_spawner;
mod actor_template;
mod collision_detection;
mod missile;
mod nateroid;
mod spaceship;
mod spaceship_control;
mod teleport;

pub use aabb::Aabb;
use aabb::AabbPlugin;
use actor_spawner::ActorSpawnerPlugin;
pub use actor_spawner::Health;
use bevy::prelude::*;
use collision_detection::CollisionDetectionPlugin;
use missile::MissilePlugin;
pub use missile::MissilePosition;
use nateroid::NateroidPlugin;
use spaceship::SpaceshipPlugin;
use spaceship_control::SpaceshipControlPlugin;
use teleport::TeleportPlugin;
pub use teleport::Teleporter;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AabbPlugin)
            .add_plugins(ActorSpawnerPlugin)
            .add_plugins(CollisionDetectionPlugin)
            .add_plugins(MissilePlugin)
            .add_plugins(NateroidPlugin)
            .add_plugins(SpaceshipPlugin)
            .add_plugins(SpaceshipControlPlugin)
            .add_plugins(TeleportPlugin);
    }
}
