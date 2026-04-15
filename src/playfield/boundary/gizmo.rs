use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_lagrange::CameraMoveList;

use super::Boundary;
use crate::camera::RenderLayer;
use crate::playfield::constants::BOUNDARY_GRID_ALPHA;
use crate::playfield::constants::BOUNDARY_OUTER_ALPHA;
use crate::playfield::constants::FADE_LOG_FRAME_EPSILON;
use crate::playfield::constants::FADE_LOG_INTERVAL_SECS;
use crate::playfield::constants::GRID_FLASH_DURATION;
use crate::playfield::types::BoundaryGizmo;
use crate::playfield::types::GridFlash;
use crate::playfield::types::GridFlashAnimation;
use crate::playfield::types::GridGizmo;
use crate::splash::SplashText;

/// Marker component for the boundary volume entity.
/// Holds a hidden unit-cube mesh so zoom-to-fit can extract vertices.
/// Syncs with `Boundary` resource configuration via `Transform` scale.
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct BoundaryVolume;

/// Component that triggers a fade-in animation for the `Boundary` gizmo
/// Lerps the `Boundary` resource's color alphas from 0.0 to target values over time
#[derive(Component)]
pub(super) struct BoundaryFadeIn(Timer);

pub(super) fn apply_boundary_settings(
    mut config_store: ResMut<GizmoConfigStore>,
    boundary: Res<Boundary>,
) {
    let (config, _) = config_store.config_mut::<GridGizmo>();
    config.line.width = boundary.grid_line_width;
    config.render_layers = RenderLayer::Game.layers();

    let (outer_config, _) = config_store.config_mut::<BoundaryGizmo>();
    outer_config.line.width = boundary.exterior_line_width;
    outer_config.render_layers = RenderLayer::Game.layers();
}

/// Spawns the `BoundaryVolume` entity with a hidden unit-cube mesh.
/// The `Transform` scale sizes it to the boundary; the mesh provides vertices for zoom-to-fit.
pub(super) fn spawn_boundary_volume(
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let scale = boundary.exterior_scalar * boundary.cell_count.as_vec3();

    commands.spawn((
        BoundaryVolume,
        Transform::from_scale(scale),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        Visibility::Hidden,
    ));

    debug!("Spawned BoundaryVolume entity");
}

/// Synchronizes the `BoundaryVolume` entity's `Transform` with the `Boundary` resource.
pub(super) fn sync_boundary_volume(
    boundary: Res<Boundary>,
    mut volume_query: Query<&mut Transform, With<BoundaryVolume>>,
) {
    let Ok(mut transform) = volume_query.single_mut() else {
        return;
    };

    // Update Transform scale to match boundary size (mesh is a unit cube scaled by Transform)
    transform.scale = boundary.exterior_scalar * boundary.cell_count.as_vec3();
}

/// draw the grid and then slightly outside the grid, draw the boundary around the whole grid
/// transform
pub(super) fn draw_boundary(
    boundary: Res<Boundary>,
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    mut grid_gizmo: Gizmos<GridGizmo>,
    mut outer_boundary_gizmo: Gizmos<BoundaryGizmo>,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    grid_gizmo
        .grid_3d(
            Isometry3d::new(boundary_transform.translation, Quat::IDENTITY),
            boundary.cell_count,
            Vec3::splat(boundary.exterior_scalar),
            boundary.grid_color,
        )
        .outer_edges();

    // Draw outer boundary box
    outer_boundary_gizmo.primitive_3d(
        &Cuboid::from_size(boundary_transform.scale),
        Isometry3d::new(boundary_transform.translation, Quat::IDENTITY),
        boundary.outer_color,
    );
}

/// Observer that triggers when `SplashText` is removed
/// Starts the boundary fade-in animation by spawning an entity with `BoundaryFadeIn`
pub(super) fn start_boundary_fade(
    _trigger: On<Remove, SplashText>,
    mut commands: Commands,
    camera_query: Query<&CameraMoveList>,
) {
    // Get remaining time from camera animation
    let remaining_time_ms = camera_query
        .iter()
        .next()
        .map_or(0.0, CameraMoveList::remaining_time_ms);

    // Convert milliseconds to seconds for Timer
    let duration_secs = remaining_time_ms / 1000.0;

    // Spawn entity with fade timer
    commands.spawn(BoundaryFadeIn(Timer::from_seconds(
        duration_secs,
        TimerMode::Once,
    )));
}

/// System that fades in the boundary gizmo by lerping alpha values
pub(super) fn fade_boundary_in(
    mut commands: Commands,
    time: Res<Time>,
    mut boundary: ResMut<Boundary>,
    mut fade_query: Query<(Entity, &mut BoundaryFadeIn)>,
) {
    for (entity, mut fade) in &mut fade_query {
        fade.0.tick(time.delta());

        // Calculate interpolation factor (0.0 to 1.0)
        let t = fade.0.fraction();

        // Lerp alpha from 0.0 to target values
        let grid_alpha = BOUNDARY_GRID_ALPHA * t;
        let outer_alpha = BOUNDARY_OUTER_ALPHA * t;

        // Update boundary colors
        boundary.grid_color = Color::from(tailwind::BLUE_500).with_alpha(grid_alpha);
        boundary.outer_color = Color::from(tailwind::BLUE_500).with_alpha(outer_alpha);

        // Log progress occasionally
        if fade.0.elapsed_secs() % FADE_LOG_INTERVAL_SECS < FADE_LOG_FRAME_EPSILON {
            debug!(
                "Boundary fade progress: {:.1}% (grid alpha={grid_alpha:.3}, outer alpha={outer_alpha:.3})",
                t * 100.0,
            );
        }

        // Remove component when fade is complete
        if fade.0.is_finished() {
            debug!("Boundary fade complete!");
            commands.entity(entity).despawn();
        }
    }
}

/// Observer that starts or resets the grid flash animation
pub(super) fn on_grid_flash(_trigger: On<GridFlash>, mut commands: Commands) {
    commands.insert_resource(GridFlashAnimation {
        timer: Timer::from_seconds(GRID_FLASH_DURATION, TimerMode::Once),
    });
}

/// Detects when `Boundary.cell_count` changes and triggers a `GridFlash`
pub(super) fn detect_cell_count_change(
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut previous_cells: Local<Option<UVec3>>,
) {
    if !boundary.is_changed() {
        return;
    }

    let current = boundary.cell_count;

    match *previous_cells {
        Some(prev) if prev == current => {},
        _ => {
            // Skip the very first change (resource initialization)
            if previous_cells.is_some() {
                commands.trigger(GridFlash);
            }
            *previous_cells = Some(current);
        },
    }
}

/// Drives the grid flash alpha using a triangle curve: 0 -> 1 -> 0 over the duration
pub(super) fn animate_grid_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash: ResMut<GridFlashAnimation>,
    mut boundary: ResMut<Boundary>,
) {
    flash.timer.tick(time.delta());

    let t = flash.timer.fraction();
    let alpha = 1.0 - 2.0f32.mul_add(t, -1.0).abs();

    boundary.grid_color = Color::from(tailwind::BLUE_500).with_alpha(alpha);

    if flash.timer.is_finished() {
        boundary.grid_color = Color::from(tailwind::BLUE_500).with_alpha(BOUNDARY_GRID_ALPHA);
        commands.remove_resource::<GridFlashAnimation>();
    }
}
