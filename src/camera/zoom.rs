use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::ZoomToFit as ZoomToFitEvent;

use crate::playfield::Boundary;

/// System that triggers zoom-to-fit when the user presses the zoom action
pub fn start_zoom_to_fit(
    mut commands: Commands,
    boundary: Res<Boundary>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    // Trigger ZoomToFit event with the boundary transform
    commands.trigger(ZoomToFitEvent::new(camera_entity, boundary.transform));
    debug!("Triggered zoom-to-fit animation");
}
