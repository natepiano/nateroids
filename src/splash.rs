use bevy::camera::visibility::RenderLayers;
use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::CameraConfig;
use crate::camera::CameraMove;
use crate::camera::CameraMoveList;
use crate::camera::PanOrbitCameraExt;
use crate::camera::RenderLayer;
use crate::camera::ZoomConfig;
use crate::camera::calculate_home_radius;
use crate::playfield::Boundary;
use crate::state::GameState;

pub struct SplashPlugin;

const SPLASH_TEXT_TIME: f32 = 2.;

#[derive(Component)]
pub struct SplashText;

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
                reset_timer_and_camera,
                spawn_splash_text,
                start_splash_camera_animation,
            ),
        )
        .add_systems(Update, run_splash.run_if(in_state(GameState::Splash)));
    }
}

fn reset_timer_and_camera(
    mut splash_timer: ResMut<SplashTextTimer>,
    camera_config: ResMut<CameraConfig>,
    mut panorbit: Single<&mut PanOrbitCamera>,
    mut boundary: ResMut<Boundary>,
) {
    debug!("Resetting timer and camera");
    splash_timer.timer.reset();

    panorbit.disable_interpolation();

    // Set both target and actual values to ensure clean start (matching initial spawn)
    panorbit.target_radius = camera_config.splash_start_radius;
    panorbit.target_focus = camera_config.splash_start_focus;
    panorbit.target_pitch = camera_config.splash_start_pitch;
    panorbit.target_yaw = camera_config.splash_start_yaw;
    panorbit.force_update = true;

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
        RenderLayers::from_layers(RenderLayer::Game.layers()),
    ));
}

fn run_splash(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut splash_text_timer: ResMut<SplashTextTimer>,
    time: Res<Time>,
    mut q_text: Query<(Entity, &mut TextFont), With<SplashText>>,
    camera_query: Query<(), (With<PanOrbitCamera>, With<CameraMoveList>)>,
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
    // This prevents exiting too early on first frame before MoveQueue is visible to query
    let timer_finished = splash_text_timer.timer.is_finished();
    let camera_animation_done = camera_query.is_empty(); // No MoveQueue = animation done

    if timer_finished && camera_animation_done {
        next_state.set(GameState::InGame {
            paused:     false,
            inspecting: false,
        });
    }
}

fn create_spin_sequence(home_radius: f32, durations: &[f32]) -> Vec<CameraMove> {
    let positions = [
        Vec3::new(0.0, 0.0, home_radius),
        Vec3::new(home_radius, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -home_radius),
        Vec3::new(-home_radius, 0.0, 0.0),
    ];

    positions
        .iter()
        .zip(durations.iter().cycle())
        .map(|(pos, &duration)| CameraMove {
            target_translation: *pos,
            target_focus:       Vec3::ZERO,
            duration_ms:        duration,
            easing:             EaseFunction::Linear,
        })
        .collect()
}

fn create_splash_camera_moves(splash_start_radius: f32, home_radius: f32) -> Vec<CameraMove> {
    let mut moves = vec![
        // sit still at the `splash start radius`
        // let's the text animate toward camera and moves the pitch/yaw
        // because our starting position for the camera pitch/yaw is off-kilter
        CameraMove {
            target_translation: Vec3::new(0.0, 0.0, splash_start_radius),
            target_focus:       Vec3::ZERO,
            duration_ms:        2500.0,
            easing:             EaseFunction::BounceOut,
        },
        // start spin 1
        CameraMove {
            target_translation: Vec3::new(0.0, 0.0, home_radius),
            target_focus:       Vec3::ZERO,
            duration_ms:        1500.0,
            easing:             EaseFunction::QuadraticIn,
        },
        CameraMove {
            target_translation: Vec3::new(home_radius, 0.0, 0.0),
            target_focus:       Vec3::ZERO,
            duration_ms:        500.0,
            easing:             EaseFunction::Linear,
        },
        CameraMove {
            target_translation: Vec3::new(0.0, 0.0, -home_radius),
            target_focus:       Vec3::ZERO,
            duration_ms:        400.0,
            easing:             EaseFunction::Linear,
        },
        CameraMove {
            target_translation: Vec3::new(-home_radius, 0.0, 0.0),
            target_focus:       Vec3::ZERO,
            duration_ms:        300.0,
            easing:             EaseFunction::Linear,
        },
        // start spin 2
        CameraMove {
            target_translation: Vec3::new(0.0, 0.0, home_radius),
            target_focus:       Vec3::ZERO,
            duration_ms:        200.0,
            easing:             EaseFunction::Linear,
        },
        CameraMove {
            target_translation: Vec3::new(home_radius, 0.0, 0.0),
            target_focus:       Vec3::ZERO,
            duration_ms:        100.0,
            easing:             EaseFunction::Linear,
        },
        CameraMove {
            target_translation: Vec3::new(0.0, 0.0, -home_radius),
            target_focus:       Vec3::ZERO,
            duration_ms:        50.0,
            easing:             EaseFunction::Linear,
        },
        CameraMove {
            target_translation: Vec3::new(-home_radius, 0.0, 0.0),
            target_focus:       Vec3::ZERO,
            duration_ms:        25.0,
            easing:             EaseFunction::Linear,
        },
    ];

    // Add fast spins 3, 4, 5 (all with 25ms duration)
    (0..5).for_each(|_| moves.extend(create_spin_sequence(home_radius, &[25.0])));

    // Add spin 6 with increasing durations (slowdown effect)
    moves.extend(create_spin_sequence(
        home_radius,
        &[50.0, 100.0, 150.0, 200.0],
    ));

    // Land at home with smooth easing
    moves.push(CameraMove {
        target_translation: Vec3::new(0.0, 0.0, home_radius),
        target_focus:       Vec3::ZERO,
        duration_ms:        1200.0,
        easing:             EaseFunction::QuadraticOut,
    });

    moves
}

fn start_splash_camera_animation(
    mut commands: Commands,
    camera_entity: Single<Entity, With<PanOrbitCamera>>,
    camera_config: Res<CameraConfig>,
    boundary: Res<Boundary>,
    zoom_config: Res<ZoomConfig>,
    camera_query: Query<(&Projection, &Camera), With<PanOrbitCamera>>,
) {
    let Ok((projection, camera)) = camera_query.single() else {
        return;
    };

    // Calculate "home" position - same as home_camera command
    let Some(home_radius) = calculate_home_radius(
        boundary.scale(),
        zoom_config.zoom_margin_multiplier(),
        projection,
        camera,
    ) else {
        return;
    };

    // Create the camera animation sequence - zoom from far (splash_start_radius) to home position
    let moves = create_splash_camera_moves(camera_config.splash_start_radius, home_radius);

    commands
        .entity(*camera_entity)
        .insert(CameraMoveList::new(moves.into()));
}
