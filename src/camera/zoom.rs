use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::DEFAULT_MARGIN;
use bevy_panorbit_camera_ext::ZoomToFit as ZoomToFitEvent;

use super::selection::ZoomTarget;
use crate::playfield::BoundaryVolume;

/// System that triggers zoom-to-fit when the user presses the zoom action.
/// Zooms to the selected entity if one exists, otherwise to the `BoundaryVolume`.
pub fn start_zoom_to_fit(
    mut commands: Commands,
    zoom_target: Res<ZoomTarget>,
    boundary_volume: Query<Entity, With<BoundaryVolume>>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let target = if let Some(selected) = zoom_target.0 {
        selected
    } else {
        let Ok(boundary) = boundary_volume.single() else {
            warn!("No BoundaryVolume entity found");
            return;
        };
        boundary
    };

    commands.trigger(ZoomToFitEvent::new(camera_entity, target, DEFAULT_MARGIN));
    debug!("Triggered zoom-to-fit to {target:?}");
}
