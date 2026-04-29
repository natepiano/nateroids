mod camera;
mod star;
mod zoom;

use bevy::prelude::*;
pub(crate) use camera::CameraSettings;
use camera::CameraSettingsInspectorPlugin;
pub(super) use star::StarSettings;
use star::StarSettingsInspectorPlugin;
use zoom::ZoomSettingsInspectorPlugin;

pub(super) struct CameraSettingsPlugin;

impl Plugin for CameraSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraSettingsInspectorPlugin)
            .add_plugins(StarSettingsInspectorPlugin)
            .add_plugins(ZoomSettingsInspectorPlugin);
    }
}
