use crate::{
    camera::RenderLayer,
    state::GameState,
};
use bevy::{
    prelude::*,
    render::view::RenderLayers,
};

pub(crate) struct SplashPlugin;

const SPLASH_TIME: f32 = 2.;

#[derive(Component)]
pub(crate) struct SplashText;

#[derive(Resource, Debug)]
struct SplashTimer {
    pub timer: Timer,
}

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashTimer {
            timer: Timer::from_seconds(SPLASH_TIME, TimerMode::Once),
        })
        .add_systems(OnEnter(GameState::Splash), splash_screen)
        .add_systems(Update, run_splash.run_if(in_state(GameState::Splash)));
    }
}

fn splash_screen(mut commands: Commands) {
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
        RenderLayers::from_layers(RenderLayer::Game.layers()),
    ));
}

fn run_splash(
    mut next_state: ResMut<NextState<GameState>>,
    mut spawn_timer: ResMut<SplashTimer>,
    time: Res<Time>,
    mut q_text: Query<&mut TextFont, With<SplashText>>,
) {
    spawn_timer.timer.tick(time.delta());
    if let Ok(mut text) = q_text.single_mut() {
        text.font_size += 1.2;
    }
    if spawn_timer.timer.just_finished() {
        next_state.set(GameState::InGame {
            paused:     false,
            inspecting: false,
        });
    }
}
