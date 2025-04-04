use crate::global_input::GlobalAction;
use bevy::prelude::*;
use bevy_rapier3d::prelude::{
    DebugRenderContext,
    NoUserData,
    RapierDebugRenderPlugin,
    RapierPhysicsPlugin,
};
use leafwing_input_manager::action_state::ActionState;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugins(RapierDebugRenderPlugin::default())
            .add_systems(Startup, init_physics_debug_aabb)
            .add_systems(Update, toggle_physics_debug);
    }
}

fn init_physics_debug_aabb(mut rapier_debug: ResMut<DebugRenderContext>) { rapier_debug.enabled = false; }

// fn disable_physics_debug(mut rapier_debug: ResMut<DebugRenderContext>) {
// rapier_debug.enabled = false; }

fn toggle_physics_debug(
    user_input: Res<ActionState<GlobalAction>>,
    mut rapier_debug: ResMut<DebugRenderContext>,
) {
    if user_input.just_pressed(&GlobalAction::PhysicsAABB) {
        rapier_debug.enabled = !rapier_debug.enabled;
        println!("Physics debug: {}", rapier_debug.enabled);
    }
}
