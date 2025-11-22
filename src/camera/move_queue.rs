//! Camera movement queue plugin and component
//! Allows for simple animation of camera movements.

//! There's a false positive on this lint for the `MoveState` enum
#![allow(clippy::used_underscore_binding)]

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::PanOrbitCameraExt;

pub struct MoveQueuePlugin;

impl Plugin for MoveQueuePlugin {
    fn build(&self, app: &mut App) { app.add_systems(Update, move_camera_system); }
}

/// Individual camera movement with target position and duration
#[derive(Clone, Reflect)]
pub struct CameraMove {
    pub target_translation: Vec3, // Where to position the camera in world space
    pub target_focus:       Vec3, // What point the camera should look at
    pub duration_ms:        f32,  // Duration in milliseconds to complete this move
}

/// State tracking for the current camera movement
#[derive(Clone, Reflect, Default, Debug)]
enum MoveState {
    InProgress {
        elapsed_ms:   f32,
        start_focus:  Vec3,
        start_pitch:  f32,
        start_radius: f32,
        start_yaw:    f32,
    },
    #[default]
    Ready,
}

/// Component that queues multiple camera movements to execute sequentially
///
/// Simply spawn this component on a camera entity with a list of movements.
/// The system will automatically process them one by one, removing the component
/// when the queue is empty.
///
/// Camera smoothing is automatically disabled while moves are in progress and
/// restored when the queue completes.
#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct MoveQueue {
    pub moves: VecDeque<CameraMove>,
    state:     MoveState,
}

impl MoveQueue {
    pub const fn new(moves: VecDeque<CameraMove>) -> Self {
        Self {
            moves,
            state: MoveState::Ready,
        }
    }
}

/// System that processes camera movement queues with duration-based linear interpolation
///
/// When the `PanOrbitCamera` has a `MoveQueue`, interpolates linearly toward the target over
/// the specified duration. When a move completes, automatically moves to the next.
/// Removes the `MoveQueue` component when all moves are complete.
pub fn move_camera_system(
    mut commands: Commands,
    time: Res<Time>,
    mut camera_query: Single<(Entity, &mut PanOrbitCamera, &mut MoveQueue)>,
) {
    let (entity, ref mut pan_orbit, ref mut queue) = *camera_query;

    // Get the current move from the front of the queue (clone to avoid borrow issues)
    let Some(current_move) = queue.moves.front().cloned() else {
        // Queue is empty - remove component (observer will restore smoothness)
        commands.entity(entity).remove::<MoveQueue>();
        return;
    };

    match &mut queue.state {
        MoveState::Ready => {
            // Disable smoothing for precise control
            pan_orbit.disable_interpolation();

            // Transition to InProgress with captured starting orbital parameters
            queue.state = MoveState::InProgress {
                elapsed_ms:   0.0,
                start_focus:  pan_orbit.target_focus,
                start_radius: pan_orbit.target_radius,
                start_yaw:    pan_orbit.target_yaw,
                start_pitch:  pan_orbit.target_pitch,
            };
        },
        MoveState::InProgress {
            elapsed_ms,
            start_focus,
            start_radius,
            start_yaw,
            start_pitch,
        } => {
            // Update elapsed time
            *elapsed_ms += time.delta_secs() * 1000.0;

            // Calculate interpolation factor (0.0 to 1.0)
            let t = (*elapsed_ms / current_move.duration_ms).min(1.0);

            // Calculate target orbital parameters from target translation and focus
            let offset = current_move.target_translation - current_move.target_focus;
            let target_radius = offset.length();
            let target_yaw = offset.x.atan2(offset.z);
            let horizontal_dist = offset.x.hypot(offset.z);
            let mut target_pitch = (-offset.y).atan2(horizontal_dist);

            // Unwrap yaw angle to ensure continuous rotation
            let mut yaw_diff = target_yaw - *start_yaw;
            // Normalize angle difference to [-PI, PI]
            yaw_diff = std::f32::consts::TAU.mul_add(
                -((yaw_diff + std::f32::consts::PI) / std::f32::consts::TAU).floor(),
                yaw_diff,
            );

            // Unwrap pitch angle to avoid discontinuous jumps
            let pitch_diff = target_pitch - *start_pitch;
            if pitch_diff > std::f32::consts::PI {
                target_pitch -= std::f32::consts::TAU;
            } else if pitch_diff < -std::f32::consts::PI {
                target_pitch += std::f32::consts::TAU;
            }
            let pitch_diff = target_pitch - *start_pitch;

            // Linear interpolation from start to target
            pan_orbit.target_focus = start_focus.lerp(current_move.target_focus, t);
            pan_orbit.target_radius = (target_radius - *start_radius).mul_add(t, *start_radius);
            pan_orbit.target_yaw = yaw_diff.mul_add(t, *start_yaw);
            pan_orbit.target_pitch = pitch_diff.mul_add(t, *start_pitch);
            pan_orbit.force_update = true;

            // Check if move complete
            if t >= 1.0 {
                queue.moves.pop_front();
                queue.state = MoveState::Ready;
            }
        },
    }
}
