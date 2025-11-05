use avian3d::prelude::*;
use bevy::prelude::*;

use crate::actor::Health;
use crate::actor::actor_spawner::CollisionDamage;
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
    name_query: Query<&Name>,
    collision_damage_query: Query<&CollisionDamage>,
) {
    for event in collision_events.read() {
        if let Ok(name1) = name_query.get(event.collider1)
            && let Ok(name2) = name_query.get(event.collider2)
        {
            apply_collision_damage(
                &mut health_query,
                &collision_damage_query,
                event.collider1,
                name1,
                event.collider2,
                name2,
            );
            apply_collision_damage(
                &mut health_query,
                &collision_damage_query,
                event.collider2,
                name2,
                event.collider1,
                name1,
            );
        }
    }
}

fn apply_collision_damage(
    health_query: &mut Query<&mut Health>,
    collision_damage_query: &Query<&CollisionDamage>,
    applying_entity: Entity,
    _applying_entity_name: &Name,
    receiving_entity: Entity,
    _receiving_entity_name: &Name,
) {
    if let Ok(mut health) = health_query.get_mut(receiving_entity)
        && let Ok(collision_damage) = collision_damage_query.get(applying_entity)
    {
        health.0 -= collision_damage.0;
    }
}
