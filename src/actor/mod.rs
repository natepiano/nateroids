mod aabb;
mod actor_settings;
mod actor_template;
mod collision_detection;
mod constants;
mod flame_gizmo;
mod missile;
mod nateroid;
mod spaceship;
mod spaceship_control;
mod teleport;

use aabb::AabbPlugin;
pub use aabb::max_dimension as aabb_max_dimension;
use actor_settings::ActorSettingsPlugin;
pub use actor_settings::Health;
pub use actor_template::DeathCorner;
pub use actor_template::NateroidSettings;
use bevy::prelude::*;
use collision_detection::CollisionDetectionPlugin;
pub use constants::NATEROID_DEATH_ALPHA_STEP;
use flame_gizmo::FlameGizmoPlugin;
use missile::MissilePlugin;
pub use nateroid::Deaderoid;
pub use nateroid::Nateroid;
pub use nateroid::NateroidDeathMaterials;
use nateroid::NateroidPlugin;
pub use spaceship::Spaceship;
use spaceship::SpaceshipPlugin;
use spaceship_control::SpaceshipControlPlugin;
use teleport::TeleportPlugin;
pub use teleport::TeleportStatus;
pub use teleport::Teleporter;

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
