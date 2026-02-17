use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::basic;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::FitTargetGizmo;
use bevy_panorbit_camera_ext::SetFitTarget;

use crate::actor::Nateroid;
use crate::actor::Spaceship;
use crate::actor::aabb_size;
use crate::camera::RenderLayer;
use crate::playfield::BoundaryVolume;

/// Custom gizmo group so selection gizmo only renders on the game camera
#[derive(Default, Reflect, GizmoConfigGroup)]
struct SelectionGizmo {}

/// Marker component added to the selected actor entity
#[derive(Component)]
pub struct Selected;

/// Resource tracking the currently selected entity for zoom-to-fit.
/// When `None`, Z zooms to boundary.
#[derive(Resource, Default)]
pub struct ZoomTarget(pub Option<Entity>);

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomTarget>()
            .init_gizmo_group::<SelectionGizmo>()
            .add_observer(on_nateroid_added)
            .add_observer(on_spaceship_added)
            .add_observer(on_selected_removed)
            .add_systems(Startup, configure_selection_gizmo)
            .add_systems(Update, clear_selection_on_background_click)
            .add_systems(Update, draw_selected_aabb_gizmo);
    }
}

// ---------------------------------------------------------------------------
// Observers
// ---------------------------------------------------------------------------

/// Attach a click observer when a `Nateroid` is spawned
fn on_nateroid_added(added: On<Add, Nateroid>, mut commands: Commands) {
    commands.entity(added.entity).observe(on_actor_clicked);
}

/// Attach a click observer when a `Spaceship` is spawned
fn on_spaceship_added(added: On<Add, Spaceship>, mut commands: Commands) {
    commands.entity(added.entity).observe(on_actor_clicked);
}

/// Per-entity observer: fires when a `Pointer<Click>` bubbles up from a
/// descendant `Mesh3d` to this actor entity.
fn on_actor_clicked(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    mut zoom_target: ResMut<ZoomTarget>,
    previously_selected: Query<Entity, With<Selected>>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let actor = click.entity;

    // Deselect previous
    for prev in previously_selected.iter() {
        if prev != actor {
            commands.entity(prev).remove::<Selected>();
        }
    }

    // Select this actor
    commands.entity(actor).insert(Selected);
    zoom_target.0 = Some(actor);

    // Update the fit-target visualization to track this entity
    if let Ok(camera) = camera_query.single() {
        commands.trigger(SetFitTarget::new(camera, actor));
    }

    debug!("Selected actor {actor:?}");
}

/// When `Selected` is removed (despawn or manual deselect), revert to boundary.
/// Skips revert if `ZoomTarget` already points to a new entity (switch-selection case).
fn on_selected_removed(
    _removed: On<Remove, Selected>,
    mut commands: Commands,
    mut zoom_target: ResMut<ZoomTarget>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
    boundary_query: Query<Entity, With<BoundaryVolume>>,
) {
    // If `on_actor_clicked` already set a new target, don't revert to boundary
    if zoom_target.0.is_some() {
        return;
    }

    zoom_target.0 = None;

    // Point fit-target visualization back at boundary
    if let Ok(camera) = camera_query.single()
        && let Ok(boundary) = boundary_query.single()
    {
        commands.trigger(SetFitTarget::new(camera, boundary));
    }

    debug!("Selection cleared, reverted to boundary");
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Clears selection when clicking on the window background (no mesh hit)
fn clear_selection_on_background_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_picking: Query<&PickingInteraction, With<Window>>,
    actor_picking: Query<&PickingInteraction, With<Selected>>,
    selected_query: Query<Entity, With<Selected>>,
    mut zoom_target: ResMut<ZoomTarget>,
    mut commands: Commands,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    let window_clicked = window_picking
        .iter()
        .any(|i| *i == PickingInteraction::Pressed);

    let actor_clicked = actor_picking
        .iter()
        .any(|i| *i == PickingInteraction::Pressed);

    if window_clicked && !actor_clicked {
        // Clear zoom target before removing `Selected` so `on_selected_removed`
        // sees `None` and properly reverts to boundary
        zoom_target.0 = None;
        for entity in selected_query.iter() {
            commands.entity(entity).remove::<Selected>();
        }
        debug!("Background click — clearing selection");
    }
}

/// Sets render layers on the `SelectionGizmo` group so it only renders on the game camera
fn configure_selection_gizmo(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<SelectionGizmo>();
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

/// Draws a purple wireframe cube around the selected entity's `Aabb`.
/// Suppressed when `FitTargetGizmo` debug visualization is active.
fn draw_selected_aabb_gizmo(
    mut gizmos: Gizmos<SelectionGizmo>,
    selected: Query<(&Transform, &Aabb), With<Selected>>,
    config_store: Res<GizmoConfigStore>,
) {
    let (fit_config, _) = config_store.config::<FitTargetGizmo>();
    if fit_config.enabled {
        return;
    }

    for (transform, aabb) in selected.iter() {
        let center = transform.transform_point(Vec3::from(aabb.center));
        gizmos.cube(
            Transform::from_translation(center)
                .with_rotation(transform.rotation)
                .with_scale(aabb_size(aabb) * transform.scale),
            Color::from(basic::PURPLE),
        );
    }
}
