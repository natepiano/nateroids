use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_panorbit_camera_ext::AnimationEnd;
use bevy_panorbit_camera_ext::CameraMove;
use bevy_panorbit_camera_ext::CameraMoveList;
use bevy_panorbit_camera_ext::PlayAnimation;
use bevy_panorbit_camera_ext::ZoomEnd;
use bevy_panorbit_camera_ext::ZoomToFit;

use crate::camera::CameraConfig;
use crate::camera::RenderLayer;
use crate::camera::ZOOM_MARGIN;
use crate::playfield::Boundary;
use crate::playfield::BoundaryVolume;
use crate::state::GameState;

pub struct SplashPlugin;

const SPLASH_TEXT_TIME: f32 = 2.;
const SPLASH_ZOOM_DURATION_MS: f32 = 1000.0;

#[derive(Component)]
pub struct SplashText;

/// Marker component indicating the splash zoom-to-fit sequence is active.
/// Present during hold and zoom phases, removed before spins start.
#[derive(Component)]
struct SplashZoomActive;

#[derive(Resource, Debug)]
struct SplashTextTimer {
    pub timer: Timer,
}

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashTextTimer {
            timer: Timer::from_seconds(SPLASH_TEXT_TIME, TimerMode::Once),
        })
        .add_systems(
            OnEnter(GameState::Splash),
            (
                reset_timer_and_boundary,
                spawn_splash_text,
                start_splash_camera_animation,
            ),
        )
        .add_systems(Update, run_splash.run_if(in_state(GameState::Splash)))
        .add_observer(on_animation_end)
        .add_observer(on_zoom_end);
    }
}

fn reset_timer_and_boundary(
    mut splash_timer: ResMut<SplashTextTimer>,
    mut boundary: ResMut<Boundary>,
) {
    debug!("Resetting timer and boundary");
    splash_timer.timer.reset();

    // Reset boundary alpha to 0 (transparent) for fade-in animation
    boundary.grid_color = boundary.grid_color.with_alpha(0.0);
    boundary.outer_color = boundary.outer_color.with_alpha(0.0);
}

fn spawn_splash_text(mut commands: Commands) {
    commands.spawn((
        SplashText,
        Text::new("nateroids"),
        TextFont {
            font_size: 1.0,
            ..default()
        },
        Node {
            align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        RenderLayer::UI.layers(),
    ));
}

fn run_splash(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut splash_text_timer: ResMut<SplashTextTimer>,
    time: Res<Time>,
    mut q_text: Query<(Entity, &mut TextFont), With<SplashText>>,
    camera_query: Query<
        (),
        (
            With<PanOrbitCamera>,
            Or<(With<CameraMoveList>, With<SplashZoomActive>)>,
        ),
    >,
) {
    splash_text_timer.timer.tick(time.delta());

    // Animate text for 2 seconds, then despawn it (observer will spawn objects)
    if let Ok((text_entity, mut text)) = q_text.single_mut() {
        if splash_text_timer.timer.just_finished() {
            // Text timer done - remove the text (triggers On<Remove, SplashText> observer)
            commands.entity(text_entity).despawn();
        } else {
            // Still animating
            text.font_size += 1.2;
        }
    }

    // Exit splash only when BOTH timer is finished AND camera animation is complete
    let timer_finished = splash_text_timer.timer.is_finished();
    let camera_animation_done = camera_query.is_empty();

    if timer_finished && camera_animation_done {
        next_state.set(GameState::InGame);
    }
}

/// When the hold animation completes, trigger `ZoomToFit` to the boundary.
fn on_animation_end(
    _trigger: On<AnimationEnd>,
    mut commands: Commands,
    camera_query: Query<Entity, (With<PanOrbitCamera>, With<SplashZoomActive>)>,
    boundary_volume: Query<Entity, With<BoundaryVolume>>,
) {
    let Ok(camera_entity) = camera_query.single() else {
        return;
    };

    let Ok(boundary_entity) = boundary_volume.single() else {
        warn!("No BoundaryVolume entity found for splash zoom-to-fit");
        return;
    };

    commands.trigger(ZoomToFit::new(
        camera_entity,
        boundary_entity,
        ZOOM_MARGIN,
        SPLASH_ZOOM_DURATION_MS,
        EaseFunction::Linear,
    ));
}

/// When zoom-to-fit completes during splash, read the radius and launch spins.
fn on_zoom_end(
    _trigger: On<ZoomEnd>,
    mut commands: Commands,
    camera_query: Query<(Entity, &PanOrbitCamera), With<SplashZoomActive>>,
) {
    let Ok((camera_entity, panorbit)) = camera_query.single() else {
        return;
    };

    let orbit_radius = panorbit.target_radius;

    commands.entity(camera_entity).remove::<SplashZoomActive>();

    let moves = create_spin_moves(orbit_radius);
    commands.trigger(PlayAnimation::new(camera_entity, moves.into()));
}

fn create_spin_sequence(radius: f32, durations: &[f32]) -> Vec<CameraMove> {
    let positions = [
        Vec3::new(0.0, 0.0, radius),
        Vec3::new(radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -radius),
        Vec3::new(-radius, 0.0, 0.0),
    ];

    positions
        .iter()
        .zip(durations.iter().cycle())
        .map(|(pos, &duration)| CameraMove::ToPosition {
            translation: *pos,
            focus: Vec3::ZERO,
            duration_ms: duration,
            easing: EaseFunction::Linear,
        })
        .collect()
}

/// Creates the spin animation sequence using the orbit radius from zoom-to-fit.
fn create_spin_moves(radius: f32) -> Vec<CameraMove> {
    let mut moves = vec![
        // start spin 1 (already at radius from zoom-to-fit, just orbit)
        CameraMove::ToPosition {
            translation: Vec3::new(radius, 0.0, 0.0),
            focus: Vec3::ZERO,
            duration_ms: 500.0,
            easing: EaseFunction::Linear,
        },
        CameraMove::ToPosition {
            translation: Vec3::new(0.0, 0.0, -radius),
            focus: Vec3::ZERO,
            duration_ms: 400.0,
            easing: EaseFunction::Linear,
        },
        CameraMove::ToPosition {
            translation: Vec3::new(-radius, 0.0, 0.0),
            focus: Vec3::ZERO,
            duration_ms: 300.0,
            easing: EaseFunction::Linear,
        },
        // start spin 2
        CameraMove::ToPosition {
            translation: Vec3::new(0.0, 0.0, radius),
            focus: Vec3::ZERO,
            duration_ms: 200.0,
            easing: EaseFunction::Linear,
        },
        CameraMove::ToPosition {
            translation: Vec3::new(radius, 0.0, 0.0),
            focus: Vec3::ZERO,
            duration_ms: 100.0,
            easing: EaseFunction::Linear,
        },
        CameraMove::ToPosition {
            translation: Vec3::new(0.0, 0.0, -radius),
            focus: Vec3::ZERO,
            duration_ms: 50.0,
            easing: EaseFunction::Linear,
        },
        CameraMove::ToPosition {
            translation: Vec3::new(-radius, 0.0, 0.0),
            focus: Vec3::ZERO,
            duration_ms: 25.0,
            easing: EaseFunction::Linear,
        },
    ];

    // Add fast spins 3, 4, 5 (all with 25ms duration)
    (0..5).for_each(|_| moves.extend(create_spin_sequence(radius, &[25.0])));

    // Add spin 6 with increasing durations (slowdown effect)
    moves.extend(create_spin_sequence(radius, &[50.0, 100.0, 150.0, 200.0]));

    // Land at home with smooth easing
    moves.push(CameraMove::ToPosition {
        translation: Vec3::new(0.0, 0.0, radius),
        focus: Vec3::ZERO,
        duration_ms: 1200.0,
        easing: EaseFunction::QuadraticOut,
    });

    moves
}

/// Snap camera to splash start position, then hold while text animates.
fn start_splash_camera_animation(
    mut commands: Commands,
    camera_config: Res<CameraConfig>,
    camera_query: Query<Entity, With<PanOrbitCamera>>,
) {
    let Ok(entity) = camera_query.single() else {
        return;
    };

    commands.entity(entity).insert(SplashZoomActive);

    // Instant snap to splash start position, then hold while text animates
    let snap_move = CameraMove::ToOrbit {
        focus: camera_config.splash_start_focus,
        yaw: camera_config.splash_start_yaw,
        pitch: camera_config.splash_start_pitch,
        radius: camera_config.splash_start_radius,
        duration_ms: 0.0,
        easing: EaseFunction::Linear,
    };
    let hold_move = CameraMove::ToPosition {
        translation: Vec3::new(0.0, 0.0, camera_config.splash_start_radius),
        focus: Vec3::ZERO,
        duration_ms: 2500.0,
        easing: EaseFunction::BounceOut,
    };

    commands.trigger(PlayAnimation::new(
        entity,
        vec![snap_move, hold_move].into(),
    ));
}
