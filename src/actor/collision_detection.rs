use bevy::prelude::*;
use bevy_rapier3d::prelude::CollisionEvent;

use crate::{
    actor::{
        Health,
        actor_spawner::CollisionDamage,
    },
    schedule::InGameSet,
};

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
    mut collision_events: EventReader<CollisionEvent>,
    mut health_query: Query<&mut Health>,
    name_query: Query<&Name>,
    collision_damage_query: Query<&CollisionDamage>,
) {
    for &collision_event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, ..) = collision_event
            && let Ok(name1) = name_query.get(entity1)
            && let Ok(name2) = name_query.get(entity2)
        {
            apply_collision_damage(
                &mut health_query,
                &collision_damage_query,
                entity1,
                name1,
                entity2,
                name2,
            );
            apply_collision_damage(
                &mut health_query,
                &collision_damage_query,
                entity2,
                name2,
                entity1,
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
