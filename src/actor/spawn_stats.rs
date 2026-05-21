use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_kana::ToF32;

use super::constants::NATEROID_SPAWN_HISTORY_LEN;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpawnResult {
    Success,
    Failure,
}

#[derive(Resource)]
pub(super) struct NateroidSpawnStats {
    /// Ring buffer tracking last N spawn attempts
    attempts:                     VecDeque<SpawnResult>,
    pub(super) last_warning_time: f32,
}

impl Default for NateroidSpawnStats {
    fn default() -> Self {
        Self {
            attempts:          VecDeque::with_capacity(NATEROID_SPAWN_HISTORY_LEN),
            last_warning_time: 0.0,
        }
    }
}

impl NateroidSpawnStats {
    pub(super) fn record_attempt(&mut self, result: SpawnResult) {
        self.attempts.push_back(result);
        if self.attempts.len() > NATEROID_SPAWN_HISTORY_LEN {
            self.attempts.pop_front();
        }
    }

    pub(super) fn success_rate(&self) -> f32 {
        if self.attempts.is_empty() {
            1.0 // No data - assume field is not crowded
        } else {
            let successes = self
                .attempts
                .iter()
                .filter(|&&result| result == SpawnResult::Success)
                .count();
            successes.to_f32() / self.attempts.len().to_f32()
        }
    }

    pub(super) fn attempts_count(&self) -> usize { self.attempts.len() }

    pub(super) fn successes_count(&self) -> usize {
        self.attempts
            .iter()
            .filter(|&&result| result == SpawnResult::Success)
            .count()
    }
}
