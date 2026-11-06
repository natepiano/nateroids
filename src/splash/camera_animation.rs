use std::time::Duration;

use bevy::curve::easing::EaseFunction;
use bevy::prelude::*;
use hana_lagrange::AnimationEnd;
use hana_lagrange::AnimationSource;
use hana_lagrange::CameraMove;
use hana_lagrange::CameraMoveError;
use hana_lagrange::Focus;
use hana_lagrange::FreeCamRollTarget;
use hana_lagrange::OrbitAngles;
use hana_lagrange::OrbitCam;
use hana_lagrange::PlayAnimation;
use hana_lagrange::Position;
use hana_lagrange::Radius;
use hana_lagrange::ZoomEnd;
use hana_lagrange::ZoomToFit;

use crate::camera::CameraSettings;
use crate::camera::ZOOM_MARGIN;
use crate::constants::SPLASH_FAST_SPIN_COUNT;
use crate::constants::SPLASH_FAST_SPIN_DURATION_MS;
use crate::constants::SPLASH_HOLD_DURATION_MS;
use crate::constants::SPLASH_LAND_HOME_DURATION_MS;
use crate::constants::SPLASH_SLOWDOWN_DURATIONS_MS;
use crate::constants::SPLASH_SPIN_DURATIONS_MS;
use crate::constants::SPLASH_ZOOM_DURATION_MS;
use crate::playfield::BoundaryVolume;

/// Marker component indicating the splash zoom-to-fit sequence is active.
/// Present during hold and zoom phases, removed before spins start.
#[derive(Component)]
pub(super) struct SplashZoomActive;

/// Marks the spin sequence until its completion event arrives.
#[derive(Component)]
pub(super) struct SplashSpinActive;

/// When the hold animation completes, trigger `ZoomToFit` to the boundary.
pub(super) fn on_animation_end(
    ended: On<AnimationEnd>,
    mut commands: Commands,
    zooming: Query<(), With<SplashZoomActive>>,
) {
    if ended.source != AnimationSource::PlayAnimation {
        return;
    }
    if zooming.contains(ended.camera) {
        commands.run_system_cached(splash_zoom_to_boundary_command);
    } else {
        commands.entity(ended.camera).remove::<SplashSpinActive>();
    }
}

/// Reusable on-demand command that starts splash zoom-to-fit to boundary.
fn splash_zoom_to_boundary_command(
    mut commands: Commands,
    camera_query: Query<Entity, (With<OrbitCam>, With<SplashZoomActive>)>,
    boundary_volume: Query<Entity, With<BoundaryVolume>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume.single() else {
        warn!("No BoundaryVolume entity found for splash zoom-to-fit");
        return;
    };

    commands.trigger(
        ZoomToFit::new(camera_entity, boundary_entity)
            .margin(ZOOM_MARGIN)
            .duration(Duration::from_millis(SPLASH_ZOOM_DURATION_MS))
            .easing(EaseFunction::Linear),
    );
}

/// When zoom-to-fit completes during splash, read the radius and launch spins.
pub(super) fn on_zoom_end(_trigger: On<ZoomEnd>, mut commands: Commands) {
    commands.run_system_cached(splash_start_spin_animation_command);
}

/// Reusable on-demand command that transitions splash zoom into spin animation.
fn splash_start_spin_animation_command(
    mut commands: Commands,
    camera_query: Query<(Entity, &OrbitCam), With<SplashZoomActive>>,
) -> Result {
    let Ok((camera_entity, orbit_cam)) = camera_query.single() else {
        return Ok(());
    };

    let orbit_radius = orbit_cam.zoom.target().0;
    let camera_moves = create_spin_moves(orbit_radius)?;

    commands
        .entity(camera_entity)
        .remove::<SplashZoomActive>()
        .insert(SplashSpinActive);

    commands.trigger(PlayAnimation::new(camera_entity, camera_moves));
    Ok(())
}

fn create_spin_sequence(
    radius: f32,
    durations: &[u64],
) -> Result<Vec<CameraMove>, CameraMoveError> {
    let positions = [
        Vec3::new(0.0, 0.0, radius),
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -radius),
        Vec3::new(-radius, 0.0, 0.0),
    ];

    positions
        .iter()
        .zip(durations.iter().cycle())
        .map(|(position, &duration)| {
            CameraMove::try_to_look_at(
                Position(*position),
                Focus(Vec3::ZERO),
                FreeCamRollTarget::InheritPrevious,
                Duration::from_millis(duration),
                EaseFunction::Linear,
            )
        })
        .collect()
}

/// Creates the spin animation sequence using the orbit radius from zoom-to-fit.
fn create_spin_moves(radius: f32) -> Result<Vec<CameraMove>, CameraMoveError> {
    // Orbit positions for one quarter-turn cycle
    let quarter_positions = [
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -radius),
        Vec3::new(-radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, radius),
    ];

    // Initial accelerating spins (spins 1-2) with decreasing durations per quarter
    let mut camera_moves: Vec<CameraMove> = quarter_positions
        .iter()
        .cycle()
        .zip(SPLASH_SPIN_DURATIONS_MS.iter())
        .map(|(&position, &ms)| {
            CameraMove::try_to_look_at(
                Position(position),
                Focus(Vec3::ZERO),
                FreeCamRollTarget::InheritPrevious,
                Duration::from_millis(ms),
                EaseFunction::Linear,
            )
        })
        .collect::<Result<_, _>>()?;

    // Add fast spins (all with fast spin duration)
    for _ in 0..SPLASH_FAST_SPIN_COUNT {
        camera_moves.extend(create_spin_sequence(
            radius,
            &[SPLASH_FAST_SPIN_DURATION_MS],
        )?);
    }

    // Add slowdown spin with increasing durations
    camera_moves.extend(create_spin_sequence(radius, SPLASH_SLOWDOWN_DURATIONS_MS)?);

    // Return to the home-facing position after the final spin.
    camera_moves.push(CameraMove::try_to_look_at(
        Position(Vec3::new(0.0, 0.0, radius)),
        Focus(Vec3::ZERO),
        FreeCamRollTarget::InheritPrevious,
        Duration::from_millis(SPLASH_LAND_HOME_DURATION_MS),
        EaseFunction::QuadraticOut,
    )?);

    Ok(camera_moves)
}

/// Snap camera to splash start position, then hold while text animates.
pub(super) fn start_splash_camera_animation(
    mut commands: Commands,
    camera_settings: Res<CameraSettings>,
    camera_query: Query<Entity, With<OrbitCam>>,
) -> Result {
    let Ok(entity) = camera_query.single() else {
        return Ok(());
    };

    // Instant snap to splash start position, then hold while text animates
    let snap_move = CameraMove::try_to_orbital_look_at(
        Focus(*camera_settings.splash_start.focus),
        OrbitAngles {
            yaw:   camera_settings.splash_start.yaw,
            pitch: camera_settings.splash_start.pitch,
        },
        Radius(camera_settings.splash_start.radius),
        FreeCamRollTarget::InheritPrevious,
        Duration::ZERO,
        EaseFunction::Linear,
    )?;
    let hold_move = CameraMove::try_to_look_at(
        Position(Vec3::new(0.0, 0.0, camera_settings.splash_start.radius)),
        Focus(Vec3::ZERO),
        FreeCamRollTarget::InheritPrevious,
        Duration::from_millis(SPLASH_HOLD_DURATION_MS),
        EaseFunction::BounceOut,
    )?;

    commands.entity(entity).insert(SplashZoomActive);
    commands.trigger(PlayAnimation::new(entity, vec![snap_move, hold_move]));
    Ok(())
}
