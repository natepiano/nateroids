use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::CameraConfig;

/// Extension trait for `PanOrbitCamera` providing convenience methods.
pub trait PanOrbitCameraExt {
    /// Allows for precise control during animations.
    fn disable_interpolation(&mut self);

    /// Enables interpolation for smooth transitions.
    fn enable_interpolation(&mut self, camera_config: &CameraConfig);

    /// Sets the home position of the camera.
    fn set_home_position(&mut self, camera_config: &CameraConfig, target_radius: f32);
}

impl PanOrbitCameraExt for PanOrbitCamera {
    fn disable_interpolation(&mut self) {
        self.zoom_smoothness = 0.0;
        self.pan_smoothness = 0.0;
        self.orbit_smoothness = 0.0;
    }

    fn enable_interpolation(&mut self, camera_config: &CameraConfig) {
        self.zoom_smoothness = camera_config.zoom_smoothness;
        self.pan_smoothness = camera_config.pan_smoothness;
        self.orbit_smoothness = camera_config.orbit_smoothness;
    }

    fn set_home_position(&mut self, camera_config: &CameraConfig, target_radius: f32) {
        self.target_focus = camera_config.splash_start_focus;
        self.target_yaw = 0.0;
        self.target_pitch = 0.0;
        self.target_radius = target_radius;
        self.force_update = true;
    }
}
