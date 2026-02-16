use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::ZoomToFit as ZoomToFitEvent;

use crate::playfield::BoundaryVolume;

/// System that triggers zoom-to-fit when the user presses the zoom action
pub fn start_zoom_to_fit(
    mut commands: Commands,
    boundary_volume: Query<Entity, With<BoundaryVolume>>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume.single() else {
        warn!("No BoundaryVolume entity found");
        return;
    };

    commands.trigger(ZoomToFitEvent::new(camera_entity, boundary_entity));
    debug!("Triggered zoom-to-fit to boundary");
}
