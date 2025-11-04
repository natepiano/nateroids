use crate::{
    camera::RenderLayer,
    global_input::GlobalAction,
};
use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers,
    prelude::*,
};
use leafwing_input_manager::action_state::ActionState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .add_plugins(PhysicsDebugPlugin)
            .add_systems(Startup, init_physics_debug_aabb)
            .add_systems(Update, toggle_physics_debug);
    }
}

fn init_physics_debug_aabb(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<PhysicsGizmos>();
    config.enabled = false;
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

fn toggle_physics_debug(
    user_input: Res<ActionState<GlobalAction>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if user_input.just_pressed(&GlobalAction::PhysicsAABB) {
        let (config, _) = config_store.config_mut::<PhysicsGizmos>();
        config.enabled = !config.enabled;
        println!("Physics debug: {}", config.enabled);
    }
}
