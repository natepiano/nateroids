use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

use crate::actor::Nateroid;
use crate::camera::RenderLayer;
use crate::game_input::GameAction;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian3d::PhysicsPlugins::default())
            .add_plugins(PhysicsDebugPlugin)
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .insert_resource(SubstepCount(15))
            .init_resource::<PhysicsMonitorState>()
            .add_systems(Startup, init_physics_debug_aabb)
            .add_systems(Update, toggle_physics_debug)
            .add_systems(FixedUpdate, monitor_physics_health);
    }
}

#[derive(Resource, Default)]
struct PhysicsMonitorState {
    is_stressed:       bool,
    last_stress_log:   f64,
    logged_unstressed: bool,
}

fn init_physics_debug_aabb(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = false;
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

fn toggle_physics_debug(
    user_input: Res<ActionState<GameAction>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if user_input.just_pressed(&GameAction::PhysicsAABB) {
        let (config, _) = config_store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
        println!("Physics debug: {}", config.enabled);
    }
}

fn monitor_physics_health(
    nateroids: Query<&LinearVelocity, With<Nateroid>>,
    time: Res<Time<Fixed>>,
    diagnostics: Res<DiagnosticsStore>,
    mut state: ResMut<PhysicsMonitorState>,
) {
    let nateroid_count = nateroids.iter().len();

    // Only monitor when there are enough entities to potentially cause issues
    if nateroid_count < 50 {
        return;
    }

    // Calculate average velocity magnitude
    let total_speed: f32 = nateroids.iter().map(|vel| vel.length()).sum();
    let avg_speed = if nateroid_count > 0 {
        total_speed / nateroid_count as f32
    } else {
        0.0
    };

    // Get FPS from diagnostics
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    // Detect potential physics breakdown with hysteresis to prevent oscillation
    // Use different thresholds for entering vs exiting stress state
    let physics_struggling = if state.is_stressed {
        // When already stressed, need FPS > 45.0 to exit
        fps < 45.0 || avg_speed > 200.0
    } else {
        // When not stressed, need FPS < 35.0 to enter
        fps < 35.0 || avg_speed > 200.0
    };

    let current_time = time.elapsed_secs_f64();

    if physics_struggling {
        // When stressed, log every 1 second
        let should_log = !state.is_stressed || (current_time - state.last_stress_log >= 1.0);

        if should_log {
            warn!(
                "⚠️  PHYSICS STRESS: {} nateroids | avg_speed: {:.1} | FPS: {:.1} | timestep:
    {:.3}ms",
                nateroid_count,
                avg_speed,
                fps,
                time.delta_secs() * 1000.0
            );
            state.is_stressed = true;
            state.last_stress_log = current_time;
            state.logged_unstressed = false;
        }
    } else {
        // When unstressed, log once on entry
        if !state.logged_unstressed {
            info!(
                "Physics healthy: {} nateroids | avg_speed: {:.1} | FPS: {:.1}",
                nateroid_count, avg_speed, fps
            );
            state.logged_unstressed = true;
            state.is_stressed = false;
        }
    }
}
