mod aabb;
mod actor_config;
mod actor_template;
mod collision_detection;
mod constants;
mod missile;
mod nateroid;
mod spaceship;
mod spaceship_control;
mod teleport;

pub use aabb::Aabb;
use aabb::AabbPlugin;
use actor_config::ActorConfigPlugin;
pub use actor_config::Health;
pub use actor_template::DeathCorner;
pub use actor_template::NateroidConfig;
use bevy::prelude::*;
use collision_detection::CollisionDetectionPlugin;
use missile::MissilePlugin;
pub use missile::MissilePosition;
pub use nateroid::Deaderoid;
pub use nateroid::Nateroid;
pub use nateroid::NateroidDeathMaterials;
use nateroid::NateroidPlugin;
pub use nateroid::NateroidSpawnStats;
use spaceship::SpaceshipPlugin;
use spaceship_control::SpaceshipControlPlugin;
use teleport::TeleportPlugin;
pub use teleport::Teleporter;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AabbPlugin)
            .add_plugins(ActorConfigPlugin)
            .add_plugins(CollisionDetectionPlugin)
            .add_plugins(MissilePlugin)
            .add_plugins(NateroidPlugin)
            .add_plugins(SpaceshipPlugin)
            .add_plugins(SpaceshipControlPlugin)
            .add_plugins(TeleportPlugin);
    }
}
