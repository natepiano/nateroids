use std::time::Duration;

use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_lagrange::AnimationEnd;
use bevy_lagrange::CameraMove;
use bevy_lagrange::OrbitCam;
use bevy_lagrange::PlayAnimation;
use bevy_lagrange::ZoomEnd;
use bevy_lagrange::ZoomToFit;

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

/// When the hold animation completes, trigger `ZoomToFit` to the boundary.
pub(super) fn on_animation_end(_trigger: On<AnimationEnd>, mut commands: Commands) {
    commands.run_system_cached(splash_zoom_to_boundary_command);
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
) {
    let Ok((camera_entity, orbit_cam)) = camera_query.single() else {
        return;
    };

    let orbit_radius = orbit_cam.target_radius;

    commands.entity(camera_entity).remove::<SplashZoomActive>();

    let camera_moves = create_spin_moves(orbit_radius);
    commands.trigger(PlayAnimation::new(camera_entity, camera_moves));
}

fn create_spin_sequence(radius: f32, durations: &[u64]) -> Vec<CameraMove> {
    let positions = [
        Vec3::new(0.0, 0.0, radius),
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -radius),
        Vec3::new(-radius, 0.0, 0.0),
    ];

    positions
        .iter()
        .zip(durations.iter().cycle())
        .map(|(position, &duration)| CameraMove::ToPosition {
            translation: *position,
            focus:       Vec3::ZERO,
            duration:    Duration::from_millis(duration),
            easing:      EaseFunction::Linear,
        })
        .collect()
}

/// Creates the spin animation sequence using the orbit radius from zoom-to-fit.
fn create_spin_moves(radius: f32) -> Vec<CameraMove> {
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
        .map(|(&translation, &ms)| CameraMove::ToPosition {
            translation,
            focus: Vec3::ZERO,
            duration: Duration::from_millis(ms),
            easing: EaseFunction::Linear,
        })
        .collect();

    // Add fast spins (all with fast spin duration)
    (0..SPLASH_FAST_SPIN_COUNT).for_each(|_| {
        camera_moves.extend(create_spin_sequence(
            radius,
            &[SPLASH_FAST_SPIN_DURATION_MS],
        ));
    });

    // Add slowdown spin with increasing durations
    camera_moves.extend(create_spin_sequence(radius, SPLASH_SLOWDOWN_DURATIONS_MS));

    // Land at home with smooth easing
    camera_moves.push(CameraMove::ToPosition {
        translation: Vec3::new(0.0, 0.0, radius),
        focus:       Vec3::ZERO,
        duration:    Duration::from_millis(SPLASH_LAND_HOME_DURATION_MS),
        easing:      EaseFunction::QuadraticOut,
    });

    camera_moves
}

/// Snap camera to splash start position, then hold while text animates.
pub(super) fn start_splash_camera_animation(
    mut commands: Commands,
    camera_settings: Res<CameraSettings>,
    camera_query: Query<Entity, With<OrbitCam>>,
) {
    let Ok(entity) = camera_query.single() else {
        return;
    };

    commands.entity(entity).insert(SplashZoomActive);

    // Instant snap to splash start position, then hold while text animates
    let snap_move = CameraMove::ToOrbit {
        focus:    *camera_settings.splash_start.focus,
        yaw:      camera_settings.splash_start.yaw,
        pitch:    camera_settings.splash_start.pitch,
        radius:   camera_settings.splash_start.radius,
        duration: Duration::ZERO,
        easing:   EaseFunction::Linear,
    };
    let hold_move = CameraMove::ToPosition {
        translation: Vec3::new(0.0, 0.0, camera_settings.splash_start.radius),
        focus:       Vec3::ZERO,
        duration:    Duration::from_millis(SPLASH_HOLD_DURATION_MS),
        easing:      EaseFunction::BounceOut,
    };

    commands.trigger(PlayAnimation::new(entity, vec![snap_move, hold_move]));
}
