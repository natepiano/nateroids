use bevy::camera::visibility::RenderLayers;
use bevy::math::curve::easing::EaseFunction;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::Hdr;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_panorbit_camera::TrackpadBehavior;
use bevy_panorbit_camera_ext::AnimateToFit;
use bevy_panorbit_camera_ext::FitTargetGizmo;
use bevy_panorbit_camera_ext::FitTargetVisualizationPlugin;
use bevy_panorbit_camera_ext::SetFitTarget;
use leafwing_input_manager::prelude::ActionState;

use super::constants::CAMERA_ZOOM_LOWER_LIMIT;
use super::constants::CAMERA_ZOOM_SENSITIVITY;
use super::constants::EDGE_MARKER_FONT_SIZE;
use super::constants::EDGE_MARKER_SPHERE_RADIUS;
use super::constants::HOME_ANIMATION_DURATION_MS;
use super::constants::ZOOM_MARGIN;
use super::lights::LightConfig;
use super::selection::SelectionPlugin;
use super::zoom::start_zoom_to_fit;
use crate::asset_loader::SceneAssets;
use crate::camera::CameraOrder;
use crate::camera::RenderLayer;
use crate::camera::config::CameraConfig;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::game_input::toggle_active;
use crate::playfield::BoundaryVolume;

pub struct CamerasPlugin;

impl Plugin for CamerasPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PanOrbitCameraPlugin)
            .add_plugins(MeshPickingPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(FitTargetVisualizationPlugin)
            .init_gizmo_group::<FocusGizmo>()
            .init_resource::<FocusConfig>()
            .init_resource::<FocusGizmoState>()
            .add_plugins(
                ResourceInspectorPlugin::<FocusConfig>::default()
                    .run_if(toggle_active(false, GameAction::FocusConfigInspector)),
            )
            .add_systems(
                Startup,
                (
                    spawn_ui_camera,
                    spawn_star_camera,
                    spawn_panorbit_camera,
                    set_fit_target_debug,
                )
                    .chain(),
            )
            .add_systems(Update, home_camera.run_if(just_pressed(GameAction::Home)))
            .add_systems(
                Update,
                start_zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)),
            )
            .add_systems(Update, toggle_fit_target_debug)
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
        Hdr,
    ));
}

// star camera uses bloom so it needs to be in its own layer as we don't
// want that effect on the colliders
// Hdr is brought in by a require on `Bloom` but adding it explicitly because
// (apparently) we need to have Hdr on all cameras so we don't run into spurious errors
// hopefully this is a long term fix for issues where i code something unrelated and the Stars layer
// disappears
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
        Hdr,
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
            Hdr,
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

/// Animate the camera back to the home position (yaw=0, pitch=0, fitted to boundary).
pub fn home_camera(
    mut commands: Commands,
    boundary_volume_query: Query<Entity, With<BoundaryVolume>>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume_query.single() else {
        warn!("No BoundaryVolume entity found");
        return;
    };

    commands.trigger(AnimateToFit::new(
        camera_entity,
        boundary_entity,
        0.0,
        0.0,
        ZOOM_MARGIN,
        HOME_ANIMATION_DURATION_MS,
        EaseFunction::QuadraticOut,
    ));
}

/// Toggle fit target visualization (mimics physics debug pattern)
fn toggle_fit_target_debug(
    user_input: Res<ActionState<GameAction>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if user_input.just_pressed(&GameAction::BoundaryBox) {
        let (config, _) = config_store.config_mut::<FitTargetGizmo>();
        config.enabled = !config.enabled;
        info!("Fit target visualization: {}", config.enabled);
    }
}

/// Sets the boundary as the debug target for fit visualization at startup
fn set_fit_target_debug(
    mut commands: Commands,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
    boundary_volume_query: Query<Entity, With<BoundaryVolume>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume_query.single() else {
        warn!("No BoundaryVolume entity found for fit target debug");
        return;
    };

    // Set boundary as the fit target for debug visualization
    commands.trigger(SetFitTarget::new(camera_entity, boundary_entity));
    info!("Set boundary as fit target for debug visualization");
}
