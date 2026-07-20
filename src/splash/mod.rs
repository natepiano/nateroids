mod camera_animation;
mod skip;
mod ui;

use bevy::prelude::*;
use skip::SplashSkipState;
use skip::SplashTextTimer;
pub(crate) use ui::SplashSkipHint;
pub(crate) use ui::SplashText;

use crate::constants::SPLASH_TEXT_TIME;
use crate::state::GameState;

pub(crate) struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashTextTimer(Timer::from_seconds(
            SPLASH_TEXT_TIME,
            TimerMode::Once,
        )))
        .init_resource::<SplashSkipState>()
        .add_systems(
            OnEnter(GameState::Splash),
            (
                skip::reset_timer_and_boundary,
                ui::splash_ui.spawn(),
                camera_animation::start_splash_camera_animation,
            ),
        )
        .add_systems(Update, skip::run_splash.run_if(in_state(GameState::Splash)))
        .add_observer(camera_animation::on_animation_end)
        .add_observer(camera_animation::on_zoom_end);
    }
}
