use avian3d::prelude::*;
use bevy::prelude::*;

use super::Health;
use super::actor_config::CollisionDamage;
use crate::schedule::InGameSet;

pub struct CollisionDetectionPlugin;

impl Plugin for CollisionDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            handle_collision_events.in_set(InGameSet::CollisionDetection),
        );
    }
}

fn handle_collision_events(
    mut collision_events: MessageReader<CollisionStart>,
    mut health_query: Query<&mut Health>,
    collision_damage_query: Query<&CollisionDamage>,
) {
    for event in collision_events.read() {
        apply_collision_damage(
            &mut health_query,
            &collision_damage_query,
            event.collider1,
            event.collider2,
        );
        apply_collision_damage(
            &mut health_query,
            &collision_damage_query,
            event.collider2,
            event.collider1,
        );
    }
}

fn apply_collision_damage(
    health_query: &mut Query<&mut Health>,
    collision_damage_query: &Query<&CollisionDamage>,
    applying_entity: Entity,
    receiving_entity: Entity,
) {
    if let Ok(mut health) = health_query.get_mut(receiving_entity)
        && let Ok(collision_damage) = collision_damage_query.get(applying_entity)
    {
        health.0 -= collision_damage.0;
    }
}
