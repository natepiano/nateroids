mod aabb;
mod actor_spawner;
mod actor_template;
mod collision_detection;
pub mod missile;
mod nateroid;
mod spaceship;
mod spaceship_control;
mod teleport;

use crate::actor::{
    aabb::AabbPlugin,
    actor_spawner::ActorSpawner,
    collision_detection::CollisionDetectionPlugin,
    missile::MissilePlugin,
    nateroid::NateroidPlugin,
    spaceship::SpaceshipPlugin,
    spaceship_control::SpaceshipControlPlugin,
    teleport::TeleportPlugin,
};
pub use crate::actor::{
    aabb::{
        Aabb,
        get_scene_aabb,
    },
    actor_spawner::{
        ColliderType,
        Health,
    },
    teleport::Teleporter,
};

use bevy::prelude::*;

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
