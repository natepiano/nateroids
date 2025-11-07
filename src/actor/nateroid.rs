use avian3d::prelude::*;
use bevy::prelude::*;

use super::Teleporter;
use super::actor_spawner::LOCKED_AXES_2D;
use super::actor_spawner::ZERO_GRAVITY;
use super::actor_spawner::create_spawn_timer;
use super::actor_spawner::spawn_actor;
use super::actor_template::NateroidConfig;
use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;

pub struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            spawn_nateroid
                .in_set(InGameSet::EntityUpdates)
                .run_if(toggle_active(true, GlobalAction::SuppressNateroids)),
        );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Nateroid;

fn spawn_nateroid(
    mut commands: Commands,
    mut config: ResMut<NateroidConfig>,
    boundary: Res<Boundary>,
    time: Res<Time>,
) {
    let nateroid_config = &mut config.0;

    if !nateroid_config.spawnable {
        return;
    }

    let spawn_timer = nateroid_config.spawn_timer.as_mut().unwrap();
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return;
    }

    spawn_actor(&mut commands, nateroid_config, Some(boundary), None);

    // Recreate timer from spawn_timer_seconds to pick up inspector changes
    nateroid_config.spawn_timer = create_spawn_timer(nateroid_config.spawn_timer_seconds);
}
