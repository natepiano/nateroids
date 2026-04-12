use bevy::camera::primitives::Aabb;
use bevy::color::palettes::tailwind;
use bevy::math::Isometry3d;
use bevy::prelude::*;
use bevy_kana::ToF32;
use rand::RngExt;

use super::FlameGizmo;
use super::compute_flicker;
use super::lerp_color;
use crate::actor::Deaderoid;
use crate::actor::aabb;
use crate::actor::constants::DEATH_EFFECT_DURATION_SECS;
use crate::actor::constants::DEATH_EFFECT_EXPANDING_RING_START_SCALE;
use crate::actor::constants::DEATH_EFFECT_LINE_COUNT;
use crate::actor::constants::DEATH_EFFECT_LINE_LENGTH_BASE;
use crate::actor::constants::DEATH_EFFECT_LINE_LENGTH_VARIANCE;
use crate::actor::constants::DEATH_EFFECT_RADIUS_MARGIN;
use crate::actor::constants::FLAME_PHASE_SPREAD;
use crate::actor::constants::FLAME_VIBRATION_AMPLITUDE;
use crate::actor::constants::FLAME_VIBRATION_SPEED;
use crate::state::PauseState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
enum DeathStyle {
    ExpandingRing,
    StaticFlash,
    MultipleRings,
}

impl DeathStyle {
    const ALL: [Self; 3] = [Self::ExpandingRing, Self::StaticFlash, Self::MultipleRings];

    fn random() -> Self {
        let mut rng = rand::rng();
        Self::ALL[rng.random_range(0..Self::ALL.len())]
    }

    const fn config(self) -> RingEffectConfig {
        match self {
            Self::ExpandingRing => RingEffectConfig::EXPANDING_RING,
            Self::StaticFlash => RingEffectConfig::STATIC_FLASH,
            Self::MultipleRings => RingEffectConfig::MULTIPLE_RINGS,
        }
    }
}

/// Visual death effect that follows the entity's current position each frame.
/// Duration is independent of entity lifetime.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct DeathEffect {
    style:    DeathStyle,
    radius:   f32,
    duration: f32,
    elapsed:  f32,
}

impl DeathEffect {
    fn new(radius: f32) -> Self {
        Self {
            style:    DeathStyle::random(),
            radius:   radius + DEATH_EFFECT_RADIUS_MARGIN,
            duration: DEATH_EFFECT_DURATION_SECS,
            elapsed:  0.0,
        }
    }
}

/// Alpha fade curve for death effects.
#[derive(Clone, Copy)]
enum AlphaCurve {
    /// Linear fade from 1.0 to 0.0
    LinearFade,
    /// Quick flash in, then fade out
    FlashInFadeOut { flash_in_fraction: f32 },
}

impl AlphaCurve {
    fn compute(self, progress: f32) -> f32 {
        match self {
            Self::LinearFade => 1.0 - progress,
            Self::FlashInFadeOut { flash_in_fraction } => {
                if progress < flash_in_fraction {
                    progress / flash_in_fraction
                } else {
                    1.0 - ((progress - flash_in_fraction) / (1.0 - flash_in_fraction))
                }
            },
        }
    }
}

#[derive(Clone, Copy)]
enum RingExpansion {
    Expanding,
    Static,
}

/// Configuration for ring-based death effects.
#[derive(Clone, Copy)]
struct RingEffectConfig {
    ring_count:        usize,
    expansion:         RingExpansion,
    radius_scale:      f32,
    line_length_scale: f32,
    ring_delay_secs:   f32,
    ring_phase_offset: f32,
    alpha_curve:       AlphaCurve,
}

impl RingEffectConfig {
    const EXPANDING_RING: Self = Self {
        ring_count:        1,
        expansion:         RingExpansion::Expanding,
        radius_scale:      1.0,
        line_length_scale: 0.5,
        ring_delay_secs:   0.0,
        ring_phase_offset: 0.0,
        alpha_curve:       AlphaCurve::LinearFade,
    };

    const STATIC_FLASH: Self = Self {
        ring_count:        1,
        expansion:         RingExpansion::Static,
        radius_scale:      0.4,
        line_length_scale: 0.5,
        ring_delay_secs:   0.0,
        ring_phase_offset: 0.0,
        alpha_curve:       AlphaCurve::FlashInFadeOut {
            flash_in_fraction: 0.2,
        },
    };

    const MULTIPLE_RINGS: Self = Self {
        ring_count:        3,
        expansion:         RingExpansion::Expanding,
        radius_scale:      1.0,
        line_length_scale: 1.0 / 3.0,
        ring_delay_secs:   0.4,
        ring_phase_offset: 2.0,
        alpha_curve:       AlphaCurve::LinearFade,
    };
}

struct RingDrawParams {
    radius:               f32,
    line_length_base:     f32,
    line_length_variance: f32,
    color_a:              Color,
    color_b:              Color,
    phase_offset:         f32,
}

/// observer that adds a death effect to a `Deaderoid`
pub(super) fn on_deaderoid_added(
    deaderoid: On<Add, Deaderoid>,
    mut commands: Commands,
    query: Query<&Aabb>,
) {
    if let Ok(aabb) = query.get(deaderoid.entity) {
        let death_effect = DeathEffect::new(aabb::max_dimension(aabb));
        commands.entity(deaderoid.entity).insert(death_effect);
    }
}

pub(super) fn draw_death_effects(
    mut commands: Commands,
    mut gizmos: Gizmos<FlameGizmo>,
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    mut frozen_elapsed: Local<f32>,
    mut death_effect_query: Query<(Entity, &mut DeathEffect, &Transform)>,
) {
    let is_paused = *pause_state.get() == PauseState::Paused;
    let elapsed = if is_paused {
        *frozen_elapsed
    } else {
        *frozen_elapsed = time.elapsed_secs();
        time.elapsed_secs()
    };

    for (entity, mut death_effect, transform) in &mut death_effect_query {
        // Only advance effect timer when not paused
        if !is_paused {
            death_effect.elapsed += time.delta_secs();

            if death_effect.elapsed >= death_effect.duration {
                commands.entity(entity).remove::<DeathEffect>();
                continue;
            }
        }

        // Use the deaderoid's rotation so the ring follows its spin
        let isometry = Isometry3d::new(transform.translation, transform.rotation);
        let config = death_effect.style.config();

        draw_death_effect_ring(&mut gizmos, &death_effect, &config, isometry, elapsed);
    }
}

/// Draws a ring of flickering lines radiating outward.
fn draw_ring_lines(
    gizmos: &mut Gizmos<FlameGizmo>,
    isometry: Isometry3d,
    params: &RingDrawParams,
    elapsed: f32,
) {
    let position = Vec3::from(isometry.translation);
    let rotation = isometry.rotation;

    let line_count_f32 = DEATH_EFFECT_LINE_COUNT.to_f32();

    for i in 0..DEATH_EFFECT_LINE_COUNT {
        let line_index = i.to_f32();

        let angle = std::f32::consts::TAU * line_index / line_count_f32;

        // Ring in XZ plane (Y normal) to align with deaderoid's spin axis
        let radial_local = Vec3::new(angle.cos(), 0.0, angle.sin());
        let tangent_local = Vec3::new(-angle.sin(), 0.0, angle.cos());

        let radial = rotation * radial_local;
        let tangent = rotation * tangent_local;

        let phase = line_index.mul_add(FLAME_PHASE_SPREAD, params.phase_offset);
        let vibration =
            elapsed.mul_add(FLAME_VIBRATION_SPEED, phase).sin() * FLAME_VIBRATION_AMPLITUDE;

        let flicker = compute_flicker(elapsed, line_index, params.phase_offset);
        let line_length = flicker
            .length
            .mul_add(params.line_length_variance, params.line_length_base);

        let start = position + radial * params.radius + tangent * vibration;
        let end = start + radial * line_length;

        let color = lerp_color(params.color_a, params.color_b, flicker.color);

        gizmos.line(start, end, color);
    }
}

/// Unified death effect drawing using `RingEffectConfig`.
fn draw_death_effect_ring(
    gizmos: &mut Gizmos<FlameGizmo>,
    death_effect: &DeathEffect,
    config: &RingEffectConfig,
    isometry: Isometry3d,
    elapsed: f32,
) {
    let line_length_base = DEATH_EFFECT_LINE_LENGTH_BASE * config.line_length_scale;
    let line_length_variance = DEATH_EFFECT_LINE_LENGTH_VARIANCE * config.line_length_scale;

    for ring_idx in 0..config.ring_count {
        let ring_idx_f32 = ring_idx.to_f32();
        let ring_start_time = ring_idx_f32 * config.ring_delay_secs;

        if death_effect.elapsed < ring_start_time {
            continue;
        }

        let ring_elapsed = death_effect.elapsed - ring_start_time;
        let ring_duration = death_effect.duration - ring_start_time;

        if ring_elapsed > ring_duration {
            continue;
        }

        let progress = ring_elapsed / ring_duration;

        let radius = if matches!(config.expansion, RingExpansion::Expanding) {
            let ease_out = (1.0 - progress).mul_add(-(1.0 - progress), 1.0);
            let scale = (1.0 - DEATH_EFFECT_EXPANDING_RING_START_SCALE)
                .mul_add(ease_out, DEATH_EFFECT_EXPANDING_RING_START_SCALE);
            death_effect.radius * config.radius_scale * scale
        } else {
            death_effect.radius * config.radius_scale
        };

        let alpha = config.alpha_curve.compute(progress);

        draw_ring_lines(
            gizmos,
            isometry,
            &RingDrawParams {
                radius,
                line_length_base,
                line_length_variance,
                color_a: Color::from(tailwind::ORANGE_500).with_alpha(alpha),
                color_b: Color::from(tailwind::YELLOW_400).with_alpha(alpha),
                phase_offset: ring_idx_f32 * config.ring_phase_offset,
            },
            elapsed,
        );
    }
}
