use bevy::prelude::*;
use bevy_lagrange::CameraMoveList;
use bevy_lagrange::OrbitCam;

use super::camera_animation::SplashZoomActive;
use super::ui::SplashSkipHint;
use super::ui::SplashText;
use crate::camera::CameraHomeEvent;
use crate::constants::SPLASH_TEXT_GROWTH_RATE;
use crate::playfield::BOUNDARY_COLOR;
use crate::playfield::BOUNDARY_START_ALPHA;
use crate::playfield::Boundary;
use crate::playfield::GridFlash;
use crate::state::GameState;

#[derive(Resource, Debug)]
pub(super) struct SplashTextTimer(pub(super) Timer);

#[derive(Default, Debug, PartialEq, Eq)]
enum SkipReadiness {
    #[default]
    NotReady,
    Armed,
}

#[derive(Resource, Debug, Default)]
pub(super) struct SplashSkipState {
    skip_readiness: SkipReadiness,
}

pub(super) fn reset_timer_and_boundary(
    mut splash_text_timer: ResMut<SplashTextTimer>,
    mut skip_state: ResMut<SplashSkipState>,
    mut boundary: ResMut<Boundary>,
) {
    debug!("Resetting timer and boundary");
    splash_text_timer.0.reset();
    skip_state.skip_readiness = SkipReadiness::NotReady;

    // `BOUNDARY_START_ALPHA` hides `Boundary` gizmos until `BoundaryFadeIn` runs.
    boundary.grid_color = BOUNDARY_COLOR.with_alpha(BOUNDARY_START_ALPHA);
    boundary.outer_color = BOUNDARY_COLOR.with_alpha(BOUNDARY_START_ALPHA);
}

pub(super) fn run_splash(
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
    if skip_state.skip_readiness == SkipReadiness::NotReady {
        // Avoid instant skip from keys held during the transition into Splash
        // (e.g. Cmd+Shift+R restart shortcut).
        if key_input.get_pressed().next().is_none() {
            skip_state.skip_readiness = SkipReadiness::Armed;
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

    splash_text_timer.0.tick(time.delta());

    // `SplashTextTimer` controls `SplashText` growth and removal; removing the
    // component starts its observer-driven transition.
    if let Ok((text_entity, mut text)) = text_query.single_mut() {
        if splash_text_timer.0.just_finished() {
            commands.entity(text_entity).despawn();
        } else if let FontSize::Px(px) = &mut text.font_size {
            *px += SPLASH_TEXT_GROWTH_RATE;
        }
    }

    // `GameState::InGame` waits for both `SplashTextTimer` and the splash camera
    // components to finish.
    let timer_finished = splash_text_timer.0.is_finished();
    let camera_animation_done = camera_query.is_empty();

    if timer_finished && camera_animation_done {
        commands.trigger(GridFlash);
        next_state.set(GameState::InGame);
    }
}
