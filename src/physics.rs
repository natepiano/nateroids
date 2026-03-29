use avian3d::prelude::*;
use bevy::diagnostic::Diagnostic;
use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy_kana::ToF32;

use crate::actor::Nateroid;
use crate::camera::RenderLayer;
use crate::input::PhysicsAabbSwitch;
use crate::switches::Switch;
use crate::switches::Switches;

const MIN_NATEROIDS_FOR_MONITORING: usize = 50;
const STRESS_EXIT_FPS_THRESHOLD: f64 = 45.0;
const STRESS_ENTER_FPS_THRESHOLD: f64 = 35.0;
const STRESS_VELOCITY_THRESHOLD: f32 = 200.0;

event!(PhysicsAabbEvent);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(avian3d::PhysicsPlugins::default())
            .add_plugins(PhysicsDebugPlugin)
            .insert_resource(SubstepCount(15))
            .init_resource::<PhysicsMonitorState>()
            .add_systems(Startup, init_physics_debug_aabb)
            .add_systems(
                Update,
                sync_physics_debug_gizmos.run_if(resource_changed::<Switches>),
            )
            .add_systems(FixedUpdate, monitor_physics_health);
        bind_action_switch!(
            app,
            PhysicsAabbSwitch,
            PhysicsAabbEvent,
            Switch::ShowPhysicsDebug
        );
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
    config.render_layers = RenderLayer::Game.layers();
}

fn sync_physics_debug_gizmos(switches: Res<Switches>, mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = switches.is_switch_on(Switch::ShowPhysicsDebug);
}

fn monitor_physics_health(
    nateroids: Query<&LinearVelocity, With<Nateroid>>,
    time: Res<Time<Fixed>>,
    diagnostics: Res<DiagnosticsStore>,
    mut state: ResMut<PhysicsMonitorState>,
) {
    let nateroid_count = nateroids.iter().len();

    // Only monitor when there are enough entities to potentially cause issues
    if nateroid_count < MIN_NATEROIDS_FOR_MONITORING {
        return;
    }

    // Calculate average velocity magnitude
    let total_speed: f32 = nateroids.iter().map(|vel| vel.length()).sum();
    let avg_speed = if nateroid_count > 0 {
        total_speed / nateroid_count.to_f32()
    } else {
        0.0
    };

    // Get FPS from diagnostics
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(Diagnostic::smoothed)
        .unwrap_or(0.0);

    // Detect potential physics breakdown with hysteresis to prevent oscillation
    // Use different thresholds for entering vs exiting stress state
    let physics_struggling = if state.is_stressed {
        fps < STRESS_EXIT_FPS_THRESHOLD || avg_speed > STRESS_VELOCITY_THRESHOLD
    } else {
        fps < STRESS_ENTER_FPS_THRESHOLD || avg_speed > STRESS_VELOCITY_THRESHOLD
    };

    let current_time = time.elapsed_secs_f64();

    if physics_struggling {
        // When stressed, log every 1 second
        let should_log = !state.is_stressed || (current_time - state.last_stress_log >= 1.0);

        if should_log {
            warn!(
                "⚠️  PHYSICS STRESS: {nateroid_count} nateroids | avg_speed: {avg_speed:.1} | FPS: {fps:.1} | timestep: {:.3}ms",
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
                "Physics healthy: {nateroid_count} nateroids | avg_speed: {avg_speed:.1} | FPS: {fps:.1}"
            );
            state.logged_unstressed = true;
            state.is_stressed = false;
        }
    }
}
