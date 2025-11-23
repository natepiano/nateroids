//! Camera movement queue plugin and component
//! Allows for simple animation of camera movements.

//! There's a false positive on this lint for the `MoveState` enum
#![allow(clippy::used_underscore_binding)]

use std::collections::VecDeque;

use bevy::math::curve::Curve;
use bevy::math::curve::easing::EaseFunction;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use super::PanOrbitCameraExt;

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
    pub easing:             EaseFunction, // Easing function for this move
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
pub struct CameraMoveList {
    pub moves: VecDeque<CameraMove>,
    state:     MoveState,
}

impl CameraMoveList {
    pub const fn new(moves: VecDeque<CameraMove>) -> Self {
        Self {
            moves,
            state: MoveState::Ready,
        }
    }

    /// Calculates total remaining time in milliseconds for all queued moves
    pub fn remaining_time_ms(&self) -> f32 {
        // Get remaining time for current move
        let current_remaining = match &self.state {
            MoveState::InProgress { elapsed_ms, .. } => {
                if let Some(current_move) = self.moves.front() {
                    (current_move.duration_ms - elapsed_ms).max(0.0)
                } else {
                    0.0
                }
            },
            MoveState::Ready => self.moves.front().map_or(0.0, |m| m.duration_ms),
        };

        // Add duration of all remaining moves (skip first since already counted)
        let remaining_queue: f32 = self.moves.iter().skip(1).map(|m| m.duration_ms).sum();

        current_remaining + remaining_queue
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
    mut camera_query: Single<(Entity, &mut PanOrbitCamera, &mut CameraMoveList)>,
) {
    let (entity, ref mut pan_orbit, ref mut queue) = *camera_query;

    // Get the current move from the front of the queue (clone to avoid borrow issues)
    let Some(current_move) = queue.moves.front().cloned() else {
        // Queue is empty - remove component (observer will restore smoothness)
        commands.entity(entity).remove::<CameraMoveList>();
        return;
    };

    // Check if this is the last move (for easing) - check prior to mutable borrow in the match
    let is_last_move = queue.moves.len() == 1;

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

            let is_final_frame = t >= 1.0;

            // Calculate canonical orbital parameters from target position
            let offset = current_move.target_translation - current_move.target_focus;
            let canonical_radius = offset.length();
            let canonical_yaw = offset.x.atan2(offset.z);
            let horizontal_dist = offset.x.hypot(offset.z);
            let canonical_pitch = (-offset.y).atan2(horizontal_dist);

            // Clamp t to exactly 1.0 if over (important for smooth completion)
            let t_clamped = t.min(1.0);

            // Apply easing function from the move
            let t_interp = current_move.easing.sample_unchecked(t_clamped);

            // Determine angle diffs: unwrap during animation, canonical on final frame
            let (yaw_diff, pitch_diff) = if is_last_move && is_final_frame {
                // Final frame of last move: use canonical angles (no unwrapping)
                (canonical_yaw - *start_yaw, canonical_pitch - *start_pitch)
            } else {
                // During animation: unwrap angles for smooth continuous rotation
                let mut yaw_diff = canonical_yaw - *start_yaw;
                // Normalize yaw difference to [-PI, PI]
                yaw_diff = std::f32::consts::TAU.mul_add(
                    -((yaw_diff + std::f32::consts::PI) / std::f32::consts::TAU).floor(),
                    yaw_diff,
                );

                // Unwrap pitch angle to avoid discontinuous jumps
                let mut pitch_target = canonical_pitch;
                let pitch_diff = pitch_target - *start_pitch;
                if pitch_diff > std::f32::consts::PI {
                    pitch_target -= std::f32::consts::TAU;
                } else if pitch_diff < -std::f32::consts::PI {
                    pitch_target += std::f32::consts::TAU;
                }
                let pitch_diff = pitch_target - *start_pitch;

                (yaw_diff, pitch_diff)
            };

            // Interpolate to target (single code path for all cases)
            pan_orbit.target_focus = start_focus.lerp(current_move.target_focus, t_interp);
            pan_orbit.target_radius =
                (canonical_radius - *start_radius).mul_add(t_interp, *start_radius);
            pan_orbit.target_yaw = yaw_diff.mul_add(t_interp, *start_yaw);
            pan_orbit.target_pitch = pitch_diff.mul_add(t_interp, *start_pitch);
            pan_orbit.force_update = true;

            // Check if move complete and advance to next
            if is_final_frame {
                queue.moves.pop_front();
                queue.state = MoveState::Ready;
            }
        },
    }
}
