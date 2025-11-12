use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::AmbientLight;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::TrackpadBehavior;
use leafwing_input_manager::prelude::*;

use crate::camera::CameraOrder;
use crate::camera::RenderLayer;
use crate::camera::calculate_camera_radius;
use crate::camera::config::CameraConfig;
use crate::camera::config::ZoomConfig;
use crate::game_input::GameAction;
use crate::playfield::Boundary;

pub struct CamerasPlugin;

impl Plugin for CamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_systems(Startup, spawn_star_camera.before(spawn_panorbit_camera))
            .add_systems(Startup, spawn_panorbit_camera)
            .add_systems(Update, move_camera_system)
            .add_systems(
                Update,
                (toggle_stars, update_bloom_settings, update_clear_color),
            );
    }
}

/// Component for programmatically moving the camera to a specific position
/// Used for testing zoom-to-fit from various camera angles
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MoveMe {
    pub target_focus:  Vec3,
    pub target_radius: f32,
    pub target_yaw:    f32, // Rotation around vertical axis (in radians)
    pub target_pitch:  f32, // Rotation up/down (in radians)
    pub speed:         f32, // Interpolation speed (0.0-1.0, higher = faster)
}

/// Screen-space margin information for a boundary
pub struct ScreenSpaceBoundary {
    /// Distance from left edge (positive = inside, negative = outside)
    pub left_margin:     f32,
    /// Distance from right edge (positive = inside, negative = outside)
    pub right_margin:    f32,
    /// Distance from top edge (positive = inside, negative = outside)
    pub top_margin:      f32,
    /// Distance from bottom edge (positive = inside, negative = outside)
    pub bottom_margin:   f32,
    /// Target margin for horizontal (in screen-space units)
    pub target_margin_x: f32,
    /// Target margin for vertical (in screen-space units)
    pub target_margin_y: f32,
    /// Minimum normalized x coordinate in screen space
    pub min_norm_x:      f32,
    /// Maximum normalized x coordinate in screen space
    pub max_norm_x:      f32,
    /// Minimum normalized y coordinate in screen space
    pub min_norm_y:      f32,
    /// Maximum normalized y coordinate in screen space
    pub max_norm_y:      f32,
    /// Average depth of boundary corners from camera
    pub avg_depth:       f32,
}

impl ScreenSpaceBoundary {
    /// Creates screen space margins from a camera's view of a boundary.
    /// Returns `None` if any boundary corner is behind the camera.
    pub fn from_camera_view(
        boundary: &Boundary,
        cam_global: &GlobalTransform,
        perspective: &PerspectiveProjection,
        viewport_aspect: f32,
        zoom_multiplier: f32,
    ) -> Option<Self> {
        let half_tan_vfov = (perspective.fov * 0.5).tan();
        let half_tan_hfov = half_tan_vfov * viewport_aspect;

        // Get boundary corners
        let boundary_corners = boundary.corners();

        // Get camera basis vectors from global transform (world position, not local)
        let cam_pos = cam_global.translation();
        let cam_rot = cam_global.rotation();
        let cam_forward = cam_rot * Vec3::NEG_Z;
        let cam_right = cam_rot * Vec3::X;
        let cam_up = cam_rot * Vec3::Y;

        // Project corners to screen space
        let mut min_norm_x = f32::INFINITY;
        let mut max_norm_x = f32::NEG_INFINITY;
        let mut min_norm_y = f32::INFINITY;
        let mut max_norm_y = f32::NEG_INFINITY;
        let mut avg_depth = 0.0;

        for corner in &boundary_corners {
            let relative = *corner - cam_pos;
            let depth = relative.dot(cam_forward);

            // Check if corner is behind camera
            if depth <= 0.1 {
                return None;
            }

            let x = relative.dot(cam_right);
            let y = relative.dot(cam_up);

            let norm_x = x / depth;
            let norm_y = y / depth;

            min_norm_x = min_norm_x.min(norm_x);
            max_norm_x = max_norm_x.max(norm_x);
            min_norm_y = min_norm_y.min(norm_y);
            max_norm_y = max_norm_y.max(norm_y);
            avg_depth += depth;
        }
        avg_depth /= boundary_corners.len() as f32;

        // Screen edges are at ±half_tan_hfov and ±half_tan_vfov
        // Target edges (with margin) are at ±(half_tan_hfov / zoom_multiplier)
        let target_edge_x = half_tan_hfov / zoom_multiplier;
        let target_edge_y = half_tan_vfov / zoom_multiplier;

        // Calculate margins as distance from bounds to screen edges
        // Positive = within screen, negative = outside
        let left_margin = min_norm_x - (-half_tan_hfov);
        let right_margin = half_tan_hfov - max_norm_x;
        let bottom_margin = min_norm_y - (-half_tan_vfov);
        let top_margin = half_tan_vfov - max_norm_y;

        // Target margins are the difference between screen edge and target edge
        let target_margin_x = half_tan_hfov - target_edge_x;
        let target_margin_y = half_tan_vfov - target_edge_y;

        Some(Self {
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            target_margin_x,
            target_margin_y,
            min_norm_x,
            max_norm_x,
            min_norm_y,
            max_norm_y,
            avg_depth,
        })
    }

    /// Returns the minimum margin across all sides
    pub fn min_margin(&self) -> f32 {
        self.left_margin
            .min(self.right_margin)
            .min(self.top_margin)
            .min(self.bottom_margin)
    }

    /// Returns true if the margins are balanced (opposite sides are equal)
    pub fn is_balanced(&self, tolerance: f32) -> bool {
        let horizontal_balanced = (self.left_margin - self.right_margin).abs() < tolerance;
        let vertical_balanced = (self.top_margin - self.bottom_margin).abs() < tolerance;
        horizontal_balanced && vertical_balanced
    }

    /// Returns true if the constraining dimension has reached its target margin.
    /// The constraining dimension is whichever has the smaller margin (tighter fit).
    pub fn is_fitted(&self, at_target_tolerance: f32) -> bool {
        let h_min = self.left_margin.min(self.right_margin);
        let v_min = self.top_margin.min(self.bottom_margin);

        // Check if horizontal dimension is at target
        let h_at_target = (h_min - self.target_margin_x).abs() < at_target_tolerance;

        // Check if vertical dimension is at target
        let v_at_target = (v_min - self.target_margin_y).abs() < at_target_tolerance;

        // At least one dimension should be at target (the constraining dimension)
        h_at_target || v_at_target
    }

    /// Returns true if horizontal is the constraining dimension
    /// The constraining dimension is the one with larger span (takes up more screen space)
    pub fn is_horizontal_constraining(&self) -> bool {
        let h_span = self.max_norm_x - self.min_norm_x;
        let v_span = self.max_norm_y - self.min_norm_y;
        h_span > v_span
    }

    /// Returns the center of the boundary in normalized screen space
    pub fn center(&self) -> (f32, f32) {
        let center_x = (self.min_norm_x + self.max_norm_x) * 0.5;
        let center_y = (self.min_norm_y + self.max_norm_y) * 0.5;
        (center_x, center_y)
    }

    /// Returns the span (width, height) of the boundary in normalized screen space
    pub fn span(&self) -> (f32, f32) {
        let span_x = self.max_norm_x - self.min_norm_x;
        let span_y = self.max_norm_y - self.min_norm_y;
        (span_x, span_y)
    }

    /// Returns the world-space offset to add to camera focus to center the boundary on screen.
    /// Uses the boundary's current screen-space center position and average depth to compute
    /// the 3D offset vector that will move the boundary to screen center.
    pub fn focus_offset_to_center(
        &self,
        cam_global: &GlobalTransform,
        half_tan_hfov: f32,
        half_tan_vfov: f32,
    ) -> Vec3 {
        let (center_x, center_y) = self.center();
        let cam_rot = cam_global.rotation();
        let cam_right = cam_rot * Vec3::X;
        let cam_up = cam_rot * Vec3::Y;

        let offset_x = center_x * self.avg_depth * half_tan_hfov;
        let offset_y = center_y * self.avg_depth * half_tan_vfov;
        cam_right * offset_x + cam_up * offset_y
    }
}

fn move_camera_system(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut PanOrbitCamera, &MoveMe), With<Camera>>,
) {
    for (entity, mut pan_orbit, move_me) in camera_query.iter_mut() {
        // Interpolate towards target values
        let focus_diff = move_me.target_focus - pan_orbit.target_focus;
        let radius_diff = move_me.target_radius - pan_orbit.target_radius;
        let yaw_diff = move_me.target_yaw - pan_orbit.target_yaw;
        let pitch_diff = move_me.target_pitch - pan_orbit.target_pitch;

        // Check if we're close enough to target
        let close_enough = focus_diff.length() < 0.001
            && radius_diff.abs() < 0.1
            && yaw_diff.abs() < 0.001
            && pitch_diff.abs() < 0.001;

        if close_enough {
            // Snap to exact target
            pan_orbit.target_focus = move_me.target_focus;
            pan_orbit.target_radius = move_me.target_radius;
            pan_orbit.target_yaw = move_me.target_yaw;
            pan_orbit.target_pitch = move_me.target_pitch;
            pan_orbit.force_update = true;

            println!(
                "Camera reached target position: focus={:?}, radius={:.1}, yaw={:.2}, pitch={:.2}",
                move_me.target_focus,
                move_me.target_radius,
                move_me.target_yaw,
                move_me.target_pitch
            );

            // Remove MoveMe component
            commands.entity(entity).remove::<MoveMe>();
        } else {
            // Interpolate towards target
            pan_orbit.target_focus += focus_diff * move_me.speed;
            pan_orbit.target_radius += radius_diff * move_me.speed;
            pan_orbit.target_yaw += yaw_diff * move_me.speed;
            pan_orbit.target_pitch += pitch_diff * move_me.speed;
            pan_orbit.force_update = true;
        }
    }
}

#[derive(Component, Reflect)]
pub struct StarsCamera;

// star camera uses bloom so it needs to be in its own layer as we don't
// want that effect on the colliders
fn spawn_star_camera(mut commands: Commands, camera_config: Res<CameraConfig>) {
    commands
        .spawn(Camera3d::default())
        .insert(Camera {
            order: CameraOrder::Stars.order(),
            clear_color: ClearColorConfig::Default,
            ..default()
        })
        .insert(Tonemapping::BlenderFilmic)
        .insert(RenderLayers::from_layers(RenderLayer::Stars.layers()))
        .insert(get_bloom_settings(camera_config))
        // CRITICAL: Adding an `AmbientLight` component to the stars camera overrides the
        // global `AmbientLight` resource for this camera only. Without this, the global
        // ambient light (used for lighting game objects) washes out the stars completely,
        // making the background appear black. The brightness value doesn't matter since
        // stars are emissive (self-illuminating), but the component must be present.
        .insert(AmbientLight {
            brightness: 0.0,
            ..default()
        })
        .insert(StarsCamera);
}

// propagate bloom settings back to the camera
fn update_bloom_settings(
    camera_config: Res<CameraConfig>,
    mut q_current_settings: Query<&mut Bloom, With<StarsCamera>>,
) {
    if camera_config.is_changed()
        && let Ok(mut old_bloom_settings) = q_current_settings.single_mut()
    {
        *old_bloom_settings = get_bloom_settings(camera_config);
    }
}

fn get_bloom_settings(camera_config: Res<CameraConfig>) -> Bloom {
    let mut new_bloom_settings = Bloom::NATURAL;

    new_bloom_settings.intensity = camera_config.bloom_intensity;
    new_bloom_settings.low_frequency_boost = camera_config.bloom_low_frequency_boost;
    new_bloom_settings.high_pass_frequency = camera_config.bloom_high_pass_frequency;
    new_bloom_settings.clone()
}

// remove and insert BloomSettings to toggle them off and on
// this can probably be removed now that bloom is pretty well working...
fn toggle_stars(
    mut commands: Commands,
    mut camera: Query<(Entity, Option<&mut Bloom>), With<StarsCamera>>,
    user_input: Res<ActionState<GameAction>>,
    camera_config: Res<CameraConfig>,
) {
    if let Ok(current_bloom_settings) = camera.single_mut() {
        match current_bloom_settings {
            (entity, Some(_)) => {
                if user_input.just_pressed(&GameAction::Stars) {
                    println!("stars off");
                    commands.entity(entity).remove::<Bloom>();
                }
            },
            (entity, None) => {
                if user_input.just_pressed(&GameAction::Stars) {
                    println!("stars on");
                    commands
                        .entity(entity)
                        .insert(get_bloom_settings(camera_config));
                }
            },
        }
    }
}

pub fn spawn_panorbit_camera(
    config: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    mut commands: Commands,
    mut q_stars_camera: Query<Entity, With<StarsCamera>>,
) {
    // we know we have one because we spawn the stars camera prior to this system
    // we're going to attach it to the primary as a child so it always has the same
    // view as the primary camera but can show the stars with bloom while the
    // primary shows everything else
    let stars_camera_entity = q_stars_camera
        .single_mut()
        .expect("why in god's name is there no star's camera?");

    // Use default FOV and aspect ratio values since the camera doesn't exist yet
    // values determined from home_camera
    // i tried having home_camera run on first frame using a run condition but it
    // didn't work it set the correct radius but it didn't actually move the
    // camera - this doesn't make sense hard coding the initial values here
    // sucks but I can live with it for now
    let default_fov = std::f32::consts::FRAC_PI_4;
    let default_aspect_ratio = 1.7777778;
    let grid_size = config.scale();
    let initial_radius = calculate_camera_radius(
        grid_size,
        default_fov,
        default_aspect_ratio,
        zoom_config.zoom_margin_multiplier(),
    );

    commands
        .spawn(PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: Some(initial_radius), // Some(config.scale().z * 2.),
            button_orbit: MouseButton::Middle,
            button_pan: MouseButton::Middle,
            modifier_pan: Some(KeyCode::ShiftLeft),
            zoom_sensitivity: 0.2,
            zoom_lower_limit: 0.001, // Allow zoom-to-fit to get very close
            trackpad_behavior: TrackpadBehavior::BlenderLike {
                modifier_pan:  Some(KeyCode::ShiftLeft),
                modifier_zoom: Some(KeyCode::ControlLeft),
            },
            trackpad_pinch_to_zoom_enabled: true,
            ..default()
        })
        //  .insert(transform)
        .insert(Camera {
            order: CameraOrder::Game.order(),
            // transparent because the game sits on top of the stars
            // this (speculative) clears the depth buffer of bloom information still - allowing
            // the game entities to render correctly without bloom
            clear_color: ClearColorConfig::Custom(Color::Srgba(Srgba::new(0.0, 0.0, 0.0, 0.0))),
            ..default()
        })
        .insert(Tonemapping::TonyMcMapface)
        .insert(RenderLayers::from_layers(RenderLayer::Game.layers()))
        .add_child(stars_camera_entity);
}

// this allows us to use Inspector reflection to manually update ClearColor to
// different values while the game is running from the ui_for_resources provided
// by bevy_inspector_egui
fn update_clear_color(camera_config: Res<CameraConfig>, mut clear_color: ResMut<ClearColor>) {
    if camera_config.is_changed() {
        clear_color.0 = camera_config
            .clear_color
            .darker(camera_config.darkening_factor);
    }
}
