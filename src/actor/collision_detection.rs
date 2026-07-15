use avian3d::prelude::*;
use bevy::prelude::*;

use super::Health;
use super::Teleporter;
use super::constants::INSTANT_DEATH_HEALTH;
use super::settings::CollisionDamage;
use super::spaceship::Spaceship;
use super::teleport::TeleportStatus;

enum InvincibleCollisionSide {
    Collider1,
    Collider2,
    Neither,
}

impl InvincibleCollisionSide {
    fn from_collision(event: &CollisionStart, spaceship_just_teleported: Option<Entity>) -> Self {
        match spaceship_just_teleported {
            Some(ship_entity) if event.collider1 == ship_entity => Self::Collider1,
            Some(ship_entity) if event.collider2 == ship_entity => Self::Collider2,
            _ => Self::Neither,
        }
    }
}

pub(super) fn handle_collision_events(
    mut collision_events: MessageReader<CollisionStart>,
    mut health_query: Query<&mut Health>,
    collision_damage_query: Query<&CollisionDamage>,
    spaceship_query: Query<(Entity, &Teleporter), With<Spaceship>>,
) {
    // `Teleporter::teleport_status` identifies the `Spaceship` entity protected
    // by post-teleport collision handling.
    let spaceship_just_teleported =
        spaceship_query
            .single()
            .ok()
            .and_then(|(entity, teleporter)| {
                (teleporter.teleport_status == TeleportStatus::JustTeleported).then_some(entity)
            });

    for event in collision_events.read() {
        match InvincibleCollisionSide::from_collision(event, spaceship_just_teleported) {
            InvincibleCollisionSide::Collider1 => {
                // A protected `CollisionStart::collider1` transfers instant death
                // to `CollisionStart::collider2`.
                if let Ok(mut health) = health_query.get_mut(event.collider2) {
                    info!(
                        "💀 Spaceship invincibility: killing nateroid that collided with just-teleported spaceship"
                    );
                    health.0 = INSTANT_DEATH_HEALTH;
                }
                apply_collision_damage(
                    &mut health_query,
                    &collision_damage_query,
                    event.collider2,
                    event.collider1,
                );
            },
            InvincibleCollisionSide::Collider2 => {
                // A protected `CollisionStart::collider2` transfers instant death
                // to `CollisionStart::collider1`.
                if let Ok(mut health) = health_query.get_mut(event.collider1) {
                    info!(
                        "💀 Spaceship invincibility: killing nateroid that collided with just-teleported spaceship"
                    );
                    health.0 = INSTANT_DEATH_HEALTH;
                }
                apply_collision_damage(
                    &mut health_query,
                    &collision_damage_query,
                    event.collider1,
                    event.collider2,
                );
            },
            InvincibleCollisionSide::Neither => {
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
            },
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
