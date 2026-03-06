use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_mesh_outline::MeshOutline;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::SetFitTarget;

use super::constants::SELECTION_OUTLINE_COLOR;
use super::constants::SELECTION_OUTLINE_INTENSITY;
use super::constants::SELECTION_OUTLINE_WIDTH;
use super::zoom::ZoomTarget;
use crate::actor::Nateroid;
use crate::actor::Spaceship;
use crate::input::InspectOutlineSwitch;
use crate::playfield::BoundaryVolume;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

event!(OutlineInspectorEvent);

/// Marker component added to the selected actor entity
#[derive(Component)]
pub struct Selected;

/// Inspector-tunable configuration for selection outlines
#[derive(Resource, Reflect, InspectorOptions, Debug, Clone)]
#[reflect(Resource, InspectorOptions)]
pub struct SelectionOutlineSettings {
    #[inspector(min = 0.0, max = 30.0, display = NumberDisplay::Slider)]
    pub width:     f32,
    #[inspector(min = 0.0, max = 30.0, display = NumberDisplay::Slider)]
    pub intensity: f32,
    pub color:     Color,
}

impl Default for SelectionOutlineSettings {
    fn default() -> Self {
        Self {
            width:     SELECTION_OUTLINE_WIDTH,
            intensity: SELECTION_OUTLINE_INTENSITY,
            color:     SELECTION_OUTLINE_COLOR,
        }
    }
}

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionOutlineSettings>()
            .add_plugins(
                ResourceInspectorPlugin::<SelectionOutlineSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectOutline)),
            )
            .add_observer(on_nateroid_added)
            .add_observer(on_spaceship_added)
            .add_observer(on_selected_added)
            .add_observer(on_selected_removed)
            .add_systems(Update, clear_selection_on_background_click)
            .add_systems(Update, sync_outline_settings);
        bind_action_switch!(
            app,
            InspectOutlineSwitch,
            OutlineInspectorEvent,
            Switch::InspectOutline
        );
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
fn on_actor_clicked(click: On<Pointer<Click>>, mut commands: Commands) {
    commands.run_system_cached_with(select_actor_command, click.entity);
}

/// Reusable on-demand command that selects an actor and updates zoom target.
fn select_actor_command(
    In(actor): In<Entity>,
    mut commands: Commands,
    mut zoom_target: ResMut<ZoomTarget>,
    previously_selected: Query<Entity, With<Selected>>,
    camera: Single<Entity, With<PanOrbitCamera>>,
) {
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
    commands.trigger(SetFitTarget::new(*camera, actor));

    debug!("Selected actor {actor:?}");
}

/// When `Selected` is added, walk all descendants and add `MeshOutline` to mesh entities.
fn on_selected_added(added: On<Add, Selected>, mut commands: Commands) {
    commands.run_system_cached_with(add_selection_outline_command, added.entity);
}

/// Reusable on-demand command that applies selection outlines to actor meshes.
fn add_selection_outline_command(
    In(entity): In<Entity>,
    mut commands: Commands,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, With<Mesh3d>>,
    settings: Res<SelectionOutlineSettings>,
) {
    let outline = MeshOutline::new(settings.width)
        .with_color(settings.color)
        .with_intensity(settings.intensity);

    for descendant in children_query.iter_descendants(entity) {
        if mesh_query.get(descendant).is_ok() {
            commands.entity(descendant).insert(outline.clone());
        }
    }
}

/// When `Selected` is removed (despawn or manual deselect), remove outlines and
/// revert zoom target to boundary.
/// Skips revert if `ZoomTarget` already points to a new entity (switch-selection case).
fn on_selected_removed(removed: On<Remove, Selected>, mut commands: Commands) {
    commands.run_system_cached_with(remove_selection_outline_command, removed.entity);
}

/// Reusable on-demand command that clears selection outlines and reverts fit target.
fn remove_selection_outline_command(
    In(entity): In<Entity>,
    mut commands: Commands,
    mut zoom_target: ResMut<ZoomTarget>,
    camera: Single<Entity, With<PanOrbitCamera>>,
    boundary_query: Query<Entity, With<BoundaryVolume>>,
    children_query: Query<&Children>,
    mesh_query: Query<Entity, With<Mesh3d>>,
) {
    // Remove outlines from all descendant meshes
    for descendant in children_query.iter_descendants(entity) {
        if mesh_query.get(descendant).is_ok() {
            commands.entity(descendant).remove::<MeshOutline>();
        }
    }

    // If `on_actor_clicked` already set a new target, don't revert to boundary
    if zoom_target.0.is_some() {
        return;
    }

    zoom_target.0 = None;

    // Point fit-target visualization back at boundary
    if let Ok(boundary) = boundary_query.single() {
        commands.trigger(SetFitTarget::new(*camera, boundary));
    }

    debug!("Selection cleared, reverted to boundary");
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Syncs `SelectionOutlineSettings` changes to all active `MeshOutline` components in real time
fn sync_outline_settings(
    settings: Res<SelectionOutlineSettings>,
    selected_query: Query<Entity, With<Selected>>,
    children_query: Query<&Children>,
    mut outline_query: Query<&mut MeshOutline>,
) {
    if !settings.is_changed() {
        return;
    }

    for entity in selected_query.iter() {
        for descendant in children_query.iter_descendants(entity) {
            if let Ok(mut outline) = outline_query.get_mut(descendant) {
                outline.width = settings.width;
                outline.color = settings.color;
                outline.intensity = settings.intensity;
            }
        }
    }
}

/// Clears selection when clicking on the window background (no mesh hit)
fn clear_selection_on_background_click(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut egui_contexts: EguiContexts,
    window_picking: Query<&PickingInteraction, With<Window>>,
    actor_picking: Query<&PickingInteraction, With<Selected>>,
    selected_query: Query<Entity, With<Selected>>,
    mut zoom_target: ResMut<ZoomTarget>,
    mut commands: Commands,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    // Don't clear selection when clicking on egui inspector windows
    if egui_contexts
        .ctx_mut()
        .is_ok_and(|ctx| ctx.wants_pointer_input())
    {
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
