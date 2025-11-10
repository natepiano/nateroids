use avian3d::prelude::*;
use bevy::prelude::*;

use super::actor_template::SpaceshipConfig;
use super::spaceship::Spaceship;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;

pub struct TeleportPlugin;

impl Plugin for TeleportPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            teleport_at_boundary.in_set(InGameSet::EntityUpdates),
        );
    }
}

#[derive(Component, Reflect, Debug, Default, Clone)]
pub struct Teleporter {
    pub just_teleported:          bool,
    pub last_teleported_position: Option<Vec3>,
    pub last_teleported_normal:   Option<Dir3>,
}

pub fn teleport_at_boundary(
    boundary: Res<Boundary>,
    mut commands: Commands,
    spaceship_config: Res<SpaceshipConfig>,
    mut teleporting_entities: Query<(
        Entity,
        &mut Transform,
        &mut Teleporter,
        Option<&Name>,
        Option<&Spaceship>,
    )>,
) {
    for (entity, mut transform, mut teleporter, name, is_spaceship) in
        teleporting_entities.iter_mut()
    {
        let original_position = transform.translation;

        let teleported_position = boundary.calculate_teleport_position(original_position);

        if teleported_position != original_position {
            // Only log spaceship teleports
            if is_spaceship.is_some() {
                let entity_name = name.map(|n| (*n).as_str()).unwrap_or("Spaceship");
                info!(
                    "🔄 {} teleporting: from ({:.1}, {:.1}, {:.1}) to ({:.1}, {:.1}, {:.1})",
                    entity_name,
                    original_position.x,
                    original_position.y,
                    original_position.z,
                    teleported_position.x,
                    teleported_position.y,
                    teleported_position.z
                );

                // Disable collisions for spaceship during teleport
                commands.entity(entity).insert(CollisionLayers::NONE);
            }

            transform.translation = teleported_position;
            teleporter.just_teleported = true;
            teleporter.last_teleported_position = Some(teleported_position);
            teleporter.last_teleported_normal =
                Some(boundary.get_normal_for_position(teleported_position));
        } else {
            // Restore collisions for spaceship
            if is_spaceship.is_some() {
                commands
                    .entity(entity)
                    .insert(spaceship_config.actor_config.collision_layers);
            }

            teleporter.just_teleported = false;
            teleporter.last_teleported_position = None;
            teleporter.last_teleported_normal = None;
        }
    }
}
