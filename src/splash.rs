use std::time::Duration;

use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_lagrange::AnimationEnd;
use bevy_lagrange::CameraMove;
use bevy_lagrange::CameraMoveList;
use bevy_lagrange::OrbitCam;
use bevy_lagrange::PlayAnimation;
use bevy_lagrange::ZoomEnd;
use bevy_lagrange::ZoomToFit;

use crate::camera::CameraHomeEvent;
use crate::camera::CameraSettings;
use crate::camera::RenderLayer;
use crate::camera::ZOOM_MARGIN;
use crate::constants::SPLASH_FAST_SPIN_COUNT;
use crate::constants::SPLASH_FAST_SPIN_DURATION_MS;
use crate::constants::SPLASH_LAND_HOME_DURATION_MS;
use crate::constants::SPLASH_SLOWDOWN_DURATIONS_MS;
use crate::constants::SPLASH_SPIN_DURATIONS_MS;
use crate::constants::SPLASH_TEXT_GROWTH_RATE;
use crate::constants::SPLASH_TEXT_TIME;
use crate::constants::SPLASH_ZOOM_DURATION_MS;
use crate::playfield::Boundary;
use crate::playfield::BoundaryVolume;
use crate::playfield::GridFlash;
use crate::state::GameState;

pub struct SplashPlugin;

#[derive(Component)]
pub struct SplashText;

/// Bottom hint shown during splash to indicate that users can skip.
#[derive(Component)]
pub struct SplashSkipHint;

/// Marker component indicating the splash zoom-to-fit sequence is active.
/// Present during hold and zoom phases, removed before spins start.
#[derive(Component)]
struct SplashZoomActive;

#[derive(Resource, Debug)]
struct SplashTextTimer {
    pub timer: Timer,
}

#[derive(Default, Debug, PartialEq, Eq)]
enum SkipReadiness {
    #[default]
    NotReady,
    Armed,
}

#[derive(Resource, Debug, Default)]
struct SplashSkipState {
    readiness: SkipReadiness,
}

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashTextTimer {
            timer: Timer::from_seconds(SPLASH_TEXT_TIME, TimerMode::Once),
        })
        .init_resource::<SplashSkipState>()
        .add_systems(
            OnEnter(GameState::Splash),
            (
                reset_timer_and_boundary,
                spawn_splash_text,
                spawn_splash_skip_hint,
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
    mut skip_state: ResMut<SplashSkipState>,
    mut boundary: ResMut<Boundary>,
) {
    debug!("Resetting timer and boundary");
    splash_timer.timer.reset();
    skip_state.readiness = SkipReadiness::NotReady;

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

fn spawn_splash_skip_hint(mut commands: Commands) {
    commands.spawn((
        SplashSkipHint,
        Text::new("Press any key to skip"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(24.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            ..default()
        },
        RenderLayer::UI.layers(),
    ));
}

fn run_splash(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut splash_text_timer: ResMut<SplashTextTimer>,
    mut skip_state: ResMut<SplashSkipState>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut text_query: Query<(Entity, &mut TextFont), With<SplashText>>,
    splash_ui_query: Query<Entity, Or<(With<SplashText>, With<SplashSkipHint>)>>,
    camera_entity: Single<Entity, With<OrbitCam>>,
    camera_query: Query<
        (),
        (
            With<OrbitCam>,
            Or<(With<CameraMoveList>, With<SplashZoomActive>)>,
        ),
    >,
) {
    if skip_state.readiness == SkipReadiness::NotReady {
        // Avoid instant skip from keys held during the transition into Splash
        // (e.g. Cmd+Shift+R restart shortcut).
        if key_input.get_pressed().next().is_none() {
            skip_state.readiness = SkipReadiness::Armed;
        }
    } else if key_input.get_just_pressed().next().is_some() {
        // Immediate splash skip: clear splash-only UI and stop in-flight splash camera sequence.
        for entity in &splash_ui_query {
            commands.entity(entity).despawn();
        }

        commands
            .entity(*camera_entity)
            .remove::<(CameraMoveList, SplashZoomActive)>();
        // Reuse the camera home command path so skip lands at the same home framing.
        commands.trigger(CameraHomeEvent);
        commands.trigger(GridFlash);
        next_state.set(GameState::InGame);
        return;
    }

    splash_text_timer.timer.tick(time.delta());

    // Animate text for 2 seconds, then despawn it (observer will spawn objects)
    if let Ok((text_entity, mut text)) = text_query.single_mut() {
        if splash_text_timer.timer.just_finished() {
            // Text timer done - remove the text (triggers On<Remove, SplashText> observer)
            commands.entity(text_entity).despawn();
        } else {
            // Still animating
            text.font_size += SPLASH_TEXT_GROWTH_RATE;
        }
    }

    // Exit splash only when BOTH timer is finished AND camera animation is complete
    let timer_finished = splash_text_timer.timer.is_finished();
    let camera_animation_done = camera_query.is_empty();

    if timer_finished && camera_animation_done {
        commands.trigger(GridFlash);
        next_state.set(GameState::InGame);
    }
}

/// When the hold animation completes, trigger `ZoomToFit` to the boundary.
fn on_animation_end(_trigger: On<AnimationEnd>, mut commands: Commands) {
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
fn on_zoom_end(_trigger: On<ZoomEnd>, mut commands: Commands) {
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

    let moves = create_spin_moves(orbit_radius);
    commands.trigger(PlayAnimation::new(camera_entity, moves));
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
        .map(|(pos, &duration)| CameraMove::ToPosition {
            translation: *pos,
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
    let mut moves: Vec<CameraMove> = quarter_positions
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
        moves.extend(create_spin_sequence(
            radius,
            &[SPLASH_FAST_SPIN_DURATION_MS],
        ));
    });

    // Add slowdown spin with increasing durations
    moves.extend(create_spin_sequence(radius, SPLASH_SLOWDOWN_DURATIONS_MS));

    // Land at home with smooth easing
    moves.push(CameraMove::ToPosition {
        translation: Vec3::new(0.0, 0.0, radius),
        focus:       Vec3::ZERO,
        duration:    Duration::from_millis(SPLASH_LAND_HOME_DURATION_MS),
        easing:      EaseFunction::QuadraticOut,
    });

    moves
}

/// Snap camera to splash start position, then hold while text animates.
fn start_splash_camera_animation(
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
        focus:    *camera_settings.splash_start_focus,
        yaw:      camera_settings.splash_start_yaw,
        pitch:    camera_settings.splash_start_pitch,
        radius:   camera_settings.splash_start_radius,
        duration: Duration::ZERO,
        easing:   EaseFunction::Linear,
    };
    let hold_move = CameraMove::ToPosition {
        translation: Vec3::new(0.0, 0.0, camera_settings.splash_start_radius),
        focus:       Vec3::ZERO,
        duration:    Duration::from_millis(2500),
        easing:      EaseFunction::BounceOut,
    };

    commands.trigger(PlayAnimation::new(entity, vec![snap_move, hold_move]));
}
