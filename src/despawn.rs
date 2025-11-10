use bevy::prelude::*;

use crate::actor::Health;
use crate::actor::MissilePosition;
use crate::schedule::InGameSet;
use crate::state::GameState;

pub struct DespawnPlugin;

impl Plugin for DespawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (despawn_dead_entities, despawn_missiles).in_set(InGameSet::DespawnEntities),
        )
        .add_systems(OnEnter(GameState::GameOver), despawn_all_entities)
        .add_systems(OnExit(GameState::Splash), despawn_splash);
    }
}

fn despawn_missiles(mut commands: Commands, query: Query<(Entity, &MissilePosition)>) {
    for (entity, missile) in query.iter() {
        if missile.traveled_distance >= missile.total_distance {
            despawn(&mut commands, entity);
        }
    }
}

/// Uses `try_despawn` because entities can be queued for despawn multiple times in a frame
/// (e.g., missile reaching max distance AND taking lethal damage simultaneously)
pub fn despawn(commands: &mut Commands, entity: Entity) { commands.entity(entity).try_despawn(); }

fn despawn_dead_entities(mut commands: Commands, query: Query<(Entity, &Health, &Name)>) {
    for (entity, health, _name) in query.iter() {
        if health.0 <= 0.0 {
            despawn(&mut commands, entity);
        }
    }
}

fn despawn_all_entities(mut commands: Commands, query: Query<Entity, With<Health>>) {
    println!("GameOver");
    for entity in query.iter() {
        despawn(&mut commands, entity);
    }
}

fn despawn_splash(mut commands: Commands, query: Query<Entity, With<crate::splash::SplashText>>) {
    for entity in query.iter() {
        despawn(&mut commands, entity);
    }
}
