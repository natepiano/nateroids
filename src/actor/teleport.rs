use bevy::prelude::*;

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

fn teleport_at_boundary(
    boundary: Res<Boundary>,
    mut teleporting_entities: Query<(&mut Transform, &mut Teleporter)>,
) {
    for (mut transform, mut teleporter) in teleporting_entities.iter_mut() {
        let original_position = transform.translation;

        let teleported_position = boundary.calculate_teleport_position(original_position);

        if teleported_position != original_position {
            transform.translation = teleported_position;
            teleporter.just_teleported = true;
            teleporter.last_teleported_position = Some(teleported_position);
            teleporter.last_teleported_normal =
                Some(boundary.get_normal_for_position(teleported_position));
        } else {
            teleporter.just_teleported = false;
            teleporter.last_teleported_position = None;
            teleporter.last_teleported_normal = None;
        }
    }
}
