use bevy::prelude::*;
use bevy_lagrange::CameraMoveList;
use bevy_lagrange::OrbitCam;
use hana_lading::AllSetsLoaded;

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

event!(
    /// Announcement from `run_splash` that the splash is over, either by a
    /// keypress skip or by its timer and camera animation completing.
    SplashFinished { skipped: bool }
);

/// Registers `enter_game` once every startup asset set has loaded. Until then
/// `SplashFinished` triggers are dropped — that is what holds the game in the
/// splash while loading runs.
pub(super) fn arm_game_entry(_loaded: On<AllSetsLoaded>, mut commands: Commands) {
    debug!("All startup asset sets loaded; game entry armed");
    commands.add_observer(enter_game);
}

/// Ends the splash: `GridFlash` plus the `GameState::InGame` transition. A
/// keypress skip additionally clears splash-only UI and stops the in-flight
/// splash camera sequence, reusing the camera home path so skip lands at the
/// same home framing.
fn enter_game(
    finished: On<SplashFinished>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    splash_ui_query: Query<Entity, Or<(With<SplashText>, With<SplashSkipHint>)>>,
    camera_entity: Single<Entity, With<OrbitCam>>,
) {
    if finished.skipped {
        for entity in &splash_ui_query {
            commands.entity(entity).despawn();
        }
        commands
            .entity(*camera_entity)
            .remove::<(CameraMoveList, SplashZoomActive)>();
        commands.trigger(CameraHomeEvent);
    }

    commands.trigger(GridFlash);
    next_state.set(GameState::InGame);
}

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
    mut splash_text_timer: ResMut<SplashTextTimer>,
    mut skip_state: ResMut<SplashSkipState>,
    key_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut text_query: Query<(Entity, &mut TextFont), With<SplashText>>,
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
        commands.trigger(SplashFinished { skipped: true });
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

    // Re-triggers every frame the splash is complete: until assets load,
    // `enter_game` does not exist and the trigger is dropped. Once it fires,
    // the state leaves `Splash` and this system's `in_state` run condition
    // stops it.
    if splash_text_timer.0.is_finished() && camera_query.is_empty() {
        commands.trigger(SplashFinished { skipped: false });
    }
}
