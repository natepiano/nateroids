use std::f32::consts::TAU;

use bevy::prelude::*;
use rand::RngExt;
use rand::prelude::ThreadRng;

use super::constants::STAR_TWINKLE_AMPLITUDE_FRACTION_MAX;
use super::constants::STAR_TWINKLE_AMPLITUDE_FRACTION_MIN;
use super::constants::STAR_TWINKLE_SPEED_FRACTION_MAX;
use super::constants::STAR_TWINKLE_SPEED_FRACTION_MIN;
use super::stars::Star;
use super::stars::StarSettings;

pub(super) struct StarTwinklingPlugin;

impl Plugin for StarTwinklingPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Update, update_twinkling); }
}

/// Per-star twinkle state, baked once at spawn so the field twinkles with
/// varied timing, amplitude, and rate rather than in lockstep.
/// `update_twinkling` scales each star by the live `StarTwinkleSettings`
/// `amplitude`/`speed` through these fractions, so an inspector edit rescales
/// every star uniformly while each keeps its individual character.
#[derive(Component)]
pub(super) struct Twinkle {
    /// Running sine argument, seeded to a random offset so stars start out of
    /// sync; advanced each frame by `speed * speed_fraction`.
    phase:              f32,
    /// This star's share of `StarTwinkleSettings::amplitude` — how much it
    /// brightens and dims relative to its neighbors.
    amplitude_fraction: f32,
    /// This star's share of `StarTwinkleSettings::speed` — how fast it cycles
    /// relative to its neighbors.
    speed_fraction:     f32,
}

impl Twinkle {
    pub(super) fn random(rng: &mut ThreadRng) -> Self {
        Self {
            phase:              rng.random_range(0.0..TAU),
            amplitude_fraction: rng.random_range(
                STAR_TWINKLE_AMPLITUDE_FRACTION_MIN..STAR_TWINKLE_AMPLITUDE_FRACTION_MAX,
            ),
            speed_fraction:     rng
                .random_range(STAR_TWINKLE_SPEED_FRACTION_MIN..STAR_TWINKLE_SPEED_FRACTION_MAX),
        }
    }
}

fn update_twinkling(
    time: Res<Time>,
    star_settings: Res<StarSettings>,
    mut stars: Query<(&Star, &MeshMaterial3d<StandardMaterial>, &mut Twinkle)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let twinkle_settings = &star_settings.twinkle;
    let delta = time.delta_secs();

    for (star, material_handle, mut twinkle) in &mut stars {
        // Wrap `phase` with `rem_euclid` so `f32` precision holds over long runs.
        twinkle.phase = (twinkle.speed_fraction)
            .mul_add(twinkle_settings.speed * delta, twinkle.phase)
            .rem_euclid(TAU);

        let Some(mut material) = materials.get_mut(material_handle) else {
            continue;
        };

        // `amplitude` is the live master knob scaled by this star's fraction;
        // `factor` oscillates around 1.0 and the clamp stops a full-amplitude
        // trough from pushing emissive below black.
        let amplitude = twinkle_settings.amplitude * twinkle.amplitude_fraction;
        let factor = amplitude.mul_add(twinkle.phase.sin(), 1.0).max(0.0);
        let emissive = star.emissive * factor;
        material.emissive = LinearRgba::new(emissive.x, emissive.y, emissive.z, emissive.w);
    }
}
