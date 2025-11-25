use bevy::camera::visibility::RenderLayers;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::TrackpadBehavior;

use super::PanOrbitCameraExt;
use super::constants::CAMERA_ZOOM_LOWER_LIMIT;
use super::constants::CAMERA_ZOOM_SENSITIVITY;
use super::constants::EDGE_MARKER_FONT_SIZE;
use super::constants::EDGE_MARKER_SPHERE_RADIUS;
use super::lights::LightConfig;
use super::move_queue::CameraMoveList;
use super::zoom::ZoomToFit;
use crate::asset_loader::SceneAssets;
use crate::camera::CameraOrder;
use crate::camera::RenderLayer;
use crate::camera::config::CameraConfig;
use crate::camera::config::ZoomConfig;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::game_input::toggle_active;
use crate::playfield::Boundary;
use crate::traits::UsizeExt;

pub struct CamerasPlugin;

impl Plugin for CamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .init_gizmo_group::<FocusGizmo>()
            .init_resource::<FocusConfig>()
            .init_resource::<FocusGizmoState>()
            .add_plugins(
                ResourceInspectorPlugin::<FocusConfig>::default()
                    .run_if(toggle_active(false, GameAction::FocusConfigInspector)),
            )
            .add_observer(reset_camera_after_moves)
            .add_systems(
                Startup,
                (spawn_ui_camera, spawn_star_camera, spawn_panorbit_camera).chain(),
            )
            .add_systems(Update, home_camera.run_if(just_pressed(GameAction::Home)))
            .add_systems(
                Update,
                apply_focus_config.run_if(resource_changed::<FocusConfig>),
            )
            .add_systems(Update, update_focus_gizmo_state)
            .add_systems(Update, update_focus_gizmo_state)
            .add_systems(
                Update,
                (
                    update_bloom_settings,
                    update_clear_color,
                    update_environment_map_intensity,
                    draw_camera_focus_gizmo.run_if(toggle_active(false, GameAction::ShowFocus)),
                    cleanup_focus_labels.run_if(toggle_active(true, GameAction::ShowFocus)),
                ),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct FocusGizmo {}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct FocusConfig {
    color:         Color,
    #[inspector(min = 0.1, max = 10.0, display = NumberDisplay::Slider)]
    line_width:    f32,
    #[inspector(min = 0.1, max = 50.0, display = NumberDisplay::Slider)]
    sphere_radius: f32,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self {
            color:         Color::srgb(1.0, 0.0, 0.0),
            line_width:    2.0,
            sphere_radius: EDGE_MARKER_SPHERE_RADIUS,
        }
    }
}

/// Stores the calculated world-space sphere radius that maintains constant screen-space size
#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
struct FocusGizmoState {
    /// World-space radius scaled to appear constant size on screen
    sphere_radius: f32,
}

/// Marker component for the focus distance label
#[derive(Component)]
struct FocusDistanceLabel;

fn apply_focus_config(mut config_store: ResMut<GizmoConfigStore>, config: Res<FocusConfig>) {
    let (gizmo_config, _) = config_store.config_mut::<FocusGizmo>();
    gizmo_config.line.width = config.line_width;
    gizmo_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

/// Updates the focus gizmo state when camera or config changes to maintain constant screen-space
/// size
fn update_focus_gizmo_state(
    camera_query: Query<&PanOrbitCamera, With<Camera>>,
    camera_changed: Query<(), (With<Camera>, Changed<PanOrbitCamera>)>,
    config: Res<FocusConfig>,
    mut state: ResMut<FocusGizmoState>,
) {
    // Only update if camera or config actually changed
    if camera_changed.is_empty() && !config.is_changed() {
        return;
    }

    if let Ok(pan_orbit) = camera_query.single() {
        // Scale sphere radius proportionally to camera distance to maintain constant screen size
        let camera_radius = pan_orbit.radius.unwrap_or(100.0);
        state.sphere_radius = config.sphere_radius * (camera_radius / 100.0);
    }
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
    /// Half tangent of vertical field of view
    pub half_tan_vfov:   f32,
    /// Half tangent of horizontal field of view (vfov * `aspect_ratio`)
    pub half_tan_hfov:   f32,
}

impl ScreenSpaceBoundary {
    /// Creates screen space margins from a camera's view of a boundary.
    /// Returns `None` if any boundary corner is behind the camera.
    #[allow(clippy::similar_names)] // half_tan_hfov vs half_tan_vfov are standard FOV terms
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
        avg_depth /= boundary_corners.len().to_f32();

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
            half_tan_vfov,
            half_tan_hfov,
        })
    }

    /// Returns true if the margins are balanced (opposite sides are equal)
    pub fn is_balanced(&self, tolerance: f32) -> bool {
        let horizontal_balanced = (self.left_margin - self.right_margin).abs() < tolerance;
        let vertical_balanced = (self.top_margin - self.bottom_margin).abs() < tolerance;
        horizontal_balanced && vertical_balanced
    }

    /// Returns true if horizontal margins are balanced (left == right)
    pub fn is_horizontally_balanced(&self, tolerance: f32) -> bool {
        (self.left_margin - self.right_margin).abs() < tolerance
    }

    /// Returns true if vertical margins are balanced (top == bottom)
    pub fn is_vertically_balanced(&self, tolerance: f32) -> bool {
        (self.top_margin - self.bottom_margin).abs() < tolerance
    }

    /// Returns true if the constraining dimension has reached its target margin.
    /// The constraining dimension is whichever has the smaller margin (tighter fit).
    pub fn is_fitted(&self, at_target_tolerance: f32) -> bool {
        let h_min = self.left_margin.min(self.right_margin);
        let v_min = self.top_margin.min(self.bottom_margin);

        // The constraining dimension is the one with smaller margin
        let (constraining_margin, target_margin) = if h_min < v_min {
            (h_min, self.target_margin_x)
        } else {
            (v_min, self.target_margin_y)
        };

        // Check if constraining dimension is at target
        (constraining_margin - target_margin).abs() < at_target_tolerance
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

    /// Returns the screen edges in normalized space (left, right, top, bottom)
    pub fn screen_edges_normalized(&self) -> (f32, f32, f32, f32) {
        (
            -self.half_tan_hfov,
            self.half_tan_hfov,
            self.half_tan_vfov,
            -self.half_tan_vfov,
        )
    }

    /// Returns the center of a boundary edge in normalized space, clipped to visible portion
    /// Returns None if the edge is not visible (entirely off-screen)
    pub fn boundary_edge_center(&self, edge: Edge) -> Option<(f32, f32)> {
        let (left_edge, right_edge, top_edge, bottom_edge) = self.screen_edges_normalized();

        match edge {
            Edge::Left if self.min_norm_x > left_edge => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                Some((self.min_norm_x, y))
            },
            Edge::Right if self.max_norm_x < right_edge => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                Some((self.max_norm_x, y))
            },
            Edge::Top if self.max_norm_y < top_edge => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                Some((x, self.max_norm_y))
            },
            Edge::Bottom if self.min_norm_y > bottom_edge => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                Some((x, self.min_norm_y))
            },
            _ => None,
        }
    }

    /// Returns the center of a screen edge in normalized space, clipped to visible boundary portion
    pub fn screen_edge_center(&self, edge: Edge) -> (f32, f32) {
        let (left_edge, right_edge, top_edge, bottom_edge) = self.screen_edges_normalized();

        match edge {
            Edge::Left => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                (left_edge, y)
            },
            Edge::Right => {
                let y = (self.min_norm_y.max(bottom_edge) + self.max_norm_y.min(top_edge)) * 0.5;
                (right_edge, y)
            },
            Edge::Top => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                (x, top_edge)
            },
            Edge::Bottom => {
                let x = (self.min_norm_x.max(left_edge) + self.max_norm_x.min(right_edge)) * 0.5;
                (x, bottom_edge)
            },
        }
    }

    /// Converts normalized screen-space coordinates to world space
    pub fn normalized_to_world(
        &self,
        norm_x: f32,
        norm_y: f32,
        cam_pos: Vec3,
        cam_right: Vec3,
        cam_up: Vec3,
        cam_forward: Vec3,
    ) -> Vec3 {
        let world_x = norm_x * self.avg_depth;
        let world_y = norm_y * self.avg_depth;
        cam_pos + cam_right * world_x + cam_up * world_y + cam_forward * self.avg_depth
    }

    /// Returns the margin percentage for a given edge.
    /// Percentage represents how much of the screen width/height is margin.
    pub fn margin_percentage(&self, edge: Edge) -> f32 {
        let screen_width = 2.0 * self.half_tan_hfov;
        let screen_height = 2.0 * self.half_tan_vfov;

        match edge {
            Edge::Left => (self.left_margin / screen_width) * 100.0,
            Edge::Right => (self.right_margin / screen_width) * 100.0,
            Edge::Top => (self.top_margin / screen_height) * 100.0,
            Edge::Bottom => (self.bottom_margin / screen_height) * 100.0,
        }
    }
}

/// Boundary box edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component, Reflect)]
pub struct StarCamera;

/// Spawns a dedicated UI camera for `egui`/`bevy_inspector_egui` to attach to.
///
/// **Why this exists:**
/// - `bevy_egui` automatically attaches to the "first found camera" during setup
/// - By spawning this camera first, we control which camera egui uses
/// - This camera has the highest order (2) so egui renders **after** all game content
///
/// **Without this camera:**
/// - egui would attach to the Stars camera (order 0, spawned first among 3D cameras)
/// - The Game camera (order 1) would then render on top, covering the UI
/// - Result: inspectors and UI would be invisible beneath game objects
///
/// **Camera configuration:**
/// - `Camera2d`: egui renders 2D UI overlays, doesn't need 3D projection
/// - `order: 2`: Highest order ensures this renders last (on top)
/// - `ClearColorConfig::None`: Preserves the 3D scene rendered by lower-order cameras
fn spawn_ui_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: CameraOrder::Ui.order(),
            ..default()
        },
    ));
}

// star camera uses bloom so it needs to be in its own layer as we don't
// want that effect on the colliders
fn spawn_star_camera(mut commands: Commands, camera_config: Res<CameraConfig>) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: CameraOrder::Stars.order(),
            ..default()
        },
        StarCamera,
        get_bloom_settings(camera_config),
        RenderLayers::from_layers(RenderLayer::Stars.layers()),
        Tonemapping::BlenderFilmic,
    ));
}

// propagate bloom settings back to the camera
fn update_bloom_settings(
    camera_config: Res<CameraConfig>,
    mut q_current_settings: Query<&mut Bloom, With<StarCamera>>,
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
    new_bloom_settings
}

fn update_environment_map_intensity(
    light_config: Res<LightConfig>,
    mut query: Query<&mut EnvironmentMapLight, With<Camera3d>>,
) {
    if !light_config.is_changed() {
        return;
    }

    for mut env_light in &mut query {
        env_light.intensity = light_config.environment_map_intensity;
    }
}

fn spawn_panorbit_camera(
    camera_config: Res<CameraConfig>,
    scene_assets: Res<SceneAssets>,
    light_config: Res<LightConfig>,
    mut commands: Commands,
    stars_camera_entity: Single<Entity, With<StarCamera>>,
) {
    commands
        .spawn((
            PanOrbitCamera {
                focus: Vec3::ZERO,
                target_radius: camera_config.splash_start_radius,
                button_orbit: MouseButton::Middle,
                button_pan: MouseButton::Middle,
                modifier_pan: Some(KeyCode::ShiftLeft),
                zoom_sensitivity: CAMERA_ZOOM_SENSITIVITY,
                zoom_lower_limit: CAMERA_ZOOM_LOWER_LIMIT,
                trackpad_behavior: TrackpadBehavior::BlenderLike {
                    modifier_pan:  Some(KeyCode::ShiftLeft),
                    modifier_zoom: Some(KeyCode::ControlLeft),
                },
                trackpad_pinch_to_zoom_enabled: true,
                ..default()
            },
            Camera {
                order: CameraOrder::Game.order(),
                // transparent because the game sits on top of the stars
                // this (speculative) clears the depth buffer of bloom information still - allowing
                // the game entities to render correctly without bloom
                clear_color: ClearColorConfig::Custom(Color::Srgba(Srgba::new(
                    0.0, 0.0, 0.0, 0.01,
                ))),
                ..default()
            },
            RenderLayers::from_layers(RenderLayer::Game.layers()),
            EnvironmentMapLight {
                diffuse_map: scene_assets.env_diffuse_map.clone(),
                specular_map: scene_assets.env_specular_map.clone(),
                intensity: light_config.environment_map_intensity,
                ..default()
            },
            Tonemapping::TonyMcMapface,
        ))
        .add_child(*stars_camera_entity);
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

/// Draws a gizmo sphere at the `PanOrbit` camera's focus point
/// and an arrow from world origin to the focus
fn draw_camera_focus_gizmo(
    mut commands: Commands,
    mut gizmos: Gizmos<FocusGizmo>,
    camera_query: Query<(&Camera, &GlobalTransform, &PanOrbitCamera)>,
    config: Res<FocusConfig>,
    state: Res<FocusGizmoState>,
    mut label_query: Query<(&mut Text, &mut Node, &mut TextColor), With<FocusDistanceLabel>>,
) {
    if let Ok((cam, cam_transform, pan_orbit)) = camera_query.single() {
        let focus = pan_orbit.target_focus;

        // Draw sphere at focus point with screen-space constant size
        gizmos.sphere(focus, state.sphere_radius, config.color);

        // Draw arrow from world origin to focus
        gizmos.arrow(Vec3::ZERO, focus, config.color);

        // Calculate distance from origin to focus
        let distance = focus.length();
        let text = format!("{distance:.1}");

        // Position label directly along arrow line so arrow points at the number
        let arrow_dir = focus.normalize_or_zero();

        // Offset along arrow direction, far enough to clear the sphere
        // Use generous offset to prevent occlusion from any camera angle
        let along_arrow_offset = state.sphere_radius.mul_add(2.0, 20.0);

        let label_world_pos = focus + (arrow_dir * along_arrow_offset);

        // Convert to screen space
        if let Ok(label_screen_pos) = cam.world_to_viewport(cam_transform, label_world_pos) {
            // Update existing label or create new one
            if let Ok((mut label_text, mut node, mut text_color)) = label_query.single_mut() {
                label_text.0.clone_from(&text);
                text_color.0 = config.color;
                node.left = Val::Px(label_screen_pos.x);
                node.top = Val::Px(label_screen_pos.y);
            } else {
                // Create new label
                commands.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: EDGE_MARKER_FONT_SIZE,
                        ..default()
                    },
                    TextColor(config.color),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(label_screen_pos.x),
                        top: Val::Px(label_screen_pos.y),
                        ..default()
                    },
                    RenderLayers::from_layers(RenderLayer::Game.layers()),
                    FocusDistanceLabel,
                ));
            }
        }
    }
}

fn cleanup_focus_labels(
    mut commands: Commands,
    label_query: Query<Entity, With<FocusDistanceLabel>>,
) {
    for entity in &label_query {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::similar_names)] // x_distance, y_distance, xy_distance are intentionally similar
pub fn calculate_home_radius(
    grid_size: Vec3,
    margin: f32,
    projection: &Projection,
    camera: &Camera,
) -> Option<f32> {
    let Projection::Perspective(perspective) = projection else {
        return None;
    };

    // Get actual viewport aspect ratio
    let aspect_ratio = if let Some(viewport_size) = camera.logical_viewport_size() {
        viewport_size.x / viewport_size.y
    } else {
        perspective.aspect_ratio
    };

    let fov = perspective.fov;

    // Calculate horizontal FOV based on aspect ratio
    let horizontal_fov = 2.0 * ((fov / 2.0).tan() * aspect_ratio).atan();

    // Calculate distances required for X and Y dimensions to fit in viewport
    let x_distance = (grid_size.x / 2.0) / (horizontal_fov / 2.0).tan();
    let y_distance = (grid_size.y / 2.0) / (fov / 2.0).tan();

    // Take the max of X and Y distances
    let xy_distance = x_distance.max(y_distance);

    // For Z dimension (depth)
    let z_half_depth = grid_size.z / 2.0;

    // Add Z depth to XY distance, then apply margin to the total
    // This ensures the entire 3D boundary fits with proper margin
    Some((xy_distance + z_half_depth) * margin)
}

/// take us back to the splash screen start position
pub fn home_camera(
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    camera_config: Res<CameraConfig>,
    camera_query: Single<(&mut PanOrbitCamera, &Projection, &Camera)>,
) {
    let (mut pan_orbit, projection, camera) = camera_query.into_inner();

    let Some(target_radius) = calculate_home_radius(
        boundary.scale(),
        zoom_config.zoom_margin_multiplier(),
        projection,
        camera,
    ) else {
        return;
    };

    // Set the camera's orbit parameters
    pan_orbit.set_home_position(&camera_config, target_radius);
}

/// Observer that runs when `MoveQueue` or `ZoomToFit` is removed from an entity.
/// Restores camera smoothness values from config.
fn reset_camera_after_moves(
    _removed: On<Remove, (CameraMoveList, ZoomToFit)>,
    camera_config: Res<CameraConfig>,
    mut pan_orbit: Single<&mut PanOrbitCamera>,
) {
    pan_orbit.enable_interpolation(&camera_config);
}
