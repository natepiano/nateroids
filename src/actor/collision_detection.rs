use avian3d::prelude::*;
use bevy::prelude::*;

use super::Health;
use super::Teleporter;
use super::actor_config::CollisionDamage;
use super::spaceship::Spaceship;
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
    spaceship_query: Query<(Entity, &Teleporter), With<Spaceship>>,
) {
    // Check if spaceship just teleported
    let spaceship_just_teleported = spaceship_query
        .single()
        .map(|(entity, teleporter)| (entity, teleporter.just_teleported))
        .ok();

    for event in collision_events.read() {
        // Check if either entity is the spaceship that just teleported
        let entity1_is_invincible_spaceship =
            spaceship_just_teleported.is_some_and(|(ship_entity, just_teleported)| {
                just_teleported && event.collider1 == ship_entity
            });
        let entity2_is_invincible_spaceship =
            spaceship_just_teleported.is_some_and(|(ship_entity, just_teleported)| {
                just_teleported && event.collider2 == ship_entity
            });

        if entity1_is_invincible_spaceship {
            // Spaceship just teleported - instantly kill entity2
            if let Ok(mut health) = health_query.get_mut(event.collider2) {
                info!(
                    "💀 Spaceship invincibility: killing nateroid that collided with just-teleported spaceship"
                );
                health.0 = -1.0; // Instant death
            }
            // Spaceship still takes normal damage
            apply_collision_damage(
                &mut health_query,
                &collision_damage_query,
                event.collider2,
                event.collider1,
            );
        } else if entity2_is_invincible_spaceship {
            // Spaceship just teleported - instantly kill entity1
            if let Ok(mut health) = health_query.get_mut(event.collider1) {
                info!(
                    "💀 Spaceship invincibility: killing nateroid that collided with just-teleported spaceship"
                );
                health.0 = -1.0; // Instant death
            }
            // Spaceship still takes normal damage
            apply_collision_damage(
                &mut health_query,
                &collision_damage_query,
                event.collider1,
                event.collider2,
            );
        } else {
            // Normal collision handling
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
