use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::math::Isometry3d;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use rand::Rng;

use super::Aabb;
use super::Deaderoid;
use super::constants::DEATH_EFFECT_DURATION_SECS;
use super::constants::DEATH_EFFECT_EXPANDING_RING_START_SCALE;
use super::constants::DEATH_EFFECT_LINE_COUNT;
use super::constants::DEATH_EFFECT_LINE_LENGTH_BASE;
use super::constants::DEATH_EFFECT_LINE_LENGTH_VARIANCE;
use super::constants::DEATH_EFFECT_RADIUS_MARGIN;
use super::constants::FLAME_COLOR_FLICKER_SPEED;
use super::constants::FLAME_GIZMO_LINE_WIDTH;
use super::constants::FLAME_LENGTH_FLICKER_PHASE_MULT;
use super::constants::FLAME_LENGTH_FLICKER_SPEED;
use super::constants::FLAME_PHASE_SPREAD;
use super::constants::FLAME_VIBRATION_AMPLITUDE;
use super::constants::FLAME_VIBRATION_SPEED;
use super::constants::THRUSTER_COLOR_FLICKER_INTENSITY;
use super::constants::THRUSTER_COLOR_ZONE_SIZE;
use super::constants::THRUSTER_CONE_HALF_ANGLE;
use super::constants::THRUSTER_LINE_COUNT;
use super::constants::THRUSTER_LINE_LENGTH_BASE;
use super::constants::THRUSTER_LINE_LENGTH_VARIANCE;
use super::constants::THRUSTER_LINE_OFFSET;
use super::constants::THRUSTER_VIBRATION_VERTICAL_PHASE_MULT;
use super::constants::THRUSTER_VIBRATION_VERTICAL_SPEED_MULT;
use super::spaceship::Spaceship;
use super::spaceship_control::SpaceshipControl;
use crate::camera::RenderLayer;
use crate::state::GameState;
use crate::state::PauseState;

pub struct FlameGizmoPlugin;

impl Plugin for FlameGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<FlameGizmo>()
            .add_systems(Startup, configure_flame_gizmo)
            .add_observer(on_deaderoid_added)
            .add_systems(
                Update,
                update_thruster_effect.run_if(in_state(PauseState::Playing)),
            )
            .add_systems(
                Update,
                (draw_thruster_flames, draw_death_effects).run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct FlameGizmo {}

fn configure_flame_gizmo(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<FlameGizmo>();
    config.line.width = FLAME_GIZMO_LINE_WIDTH;
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

/// observer that adds a death effect to a `Deaderoid`
fn on_deaderoid_added(deaderoid: On<Add, Deaderoid>, mut commands: Commands, query: Query<&Aabb>) {
    if let Ok(aabb) = query.get(deaderoid.entity) {
        let death_effect = DeathEffect::new(aabb.max_dimension());
        commands.entity(deaderoid.entity).insert(death_effect);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum DeathStyle {
    ExpandingRing,
    StaticFlash,
    MultipleRings,
}

impl DeathStyle {
    const ALL: [Self; 3] = [Self::ExpandingRing, Self::StaticFlash, Self::MultipleRings];

    pub fn random() -> Self {
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
pub struct DeathEffect {
    pub style:    DeathStyle,
    pub radius:   f32,
    pub duration: f32,
    pub elapsed:  f32,
}

impl DeathEffect {
    pub fn new(radius: f32) -> Self {
        Self {
            style:    DeathStyle::random(),
            radius:   radius + DEATH_EFFECT_RADIUS_MARGIN,
            duration: DEATH_EFFECT_DURATION_SECS,
            elapsed:  0.0,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ThrusterEffect;

struct FlickerValues {
    length: f32,
    color:  f32,
}

fn compute_flicker(elapsed: f32, line_index: f32, phase_offset: f32) -> FlickerValues {
    let phase = line_index.mul_add(FLAME_PHASE_SPREAD, phase_offset);
    FlickerValues {
        length: elapsed
            .mul_add(
                FLAME_LENGTH_FLICKER_SPEED,
                phase * FLAME_LENGTH_FLICKER_PHASE_MULT,
            )
            .sin()
            .mul_add(0.5, 0.5),
        color:  elapsed
            .mul_add(FLAME_COLOR_FLICKER_SPEED, phase)
            .sin()
            .mul_add(0.5, 0.5),
    }
}

/// Classifies a position within a flame into temperature zones.
/// Used for thruster color gradients where center = hottest.
enum FlameZone {
    /// Outer edge of flame (coolest) - value is normalized 0-1 within zone
    Outer(f32),
    /// Middle transition zone (uses time-based flicker, not position)
    Middle,
    /// Center of flame (hottest) - value is normalized 0-1 within zone
    Center(f32),
}

impl FlameZone {
    /// Classifies a `center_factor` (0.0 = edge, 1.0 = center) into zones.
    fn from_center_factor(value: f32, zone_size: f32) -> Self {
        let middle_threshold = zone_size * 2.0;
        if value < zone_size {
            Self::Outer(value / zone_size)
        } else if value < middle_threshold {
            Self::Middle
        } else {
            Self::Center((value - middle_threshold) / (1.0 - middle_threshold))
        }
    }

    /// Computes the flame color based on zone.
    /// - Outer: red → orange (coolest)
    /// - Middle: orange with flicker toward yellow/red
    /// - Center: orange → yellow (hottest)
    fn color(
        &self,
        elapsed: f32,
        phase: f32,
        color_red: Color,
        color_orange: Color,
        color_yellow: Color,
    ) -> Color {
        match self {
            Self::Outer(t) => lerp_color(color_red, color_orange, *t),
            Self::Middle => {
                // Flicker between cooler (red) and hotter (yellow)
                let flicker = elapsed.mul_add(FLAME_COLOR_FLICKER_SPEED, phase).sin();
                if flicker > 0.0 {
                    lerp_color(
                        color_orange,
                        color_yellow,
                        flicker * THRUSTER_COLOR_FLICKER_INTENSITY,
                    )
                } else {
                    lerp_color(
                        color_orange,
                        color_red,
                        -flicker * THRUSTER_COLOR_FLICKER_INTENSITY,
                    )
                }
            },
            Self::Center(t) => lerp_color(color_orange, color_yellow, *t),
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

/// Configuration for ring-based death effects.
#[derive(Clone, Copy)]
struct RingEffectConfig {
    ring_count:        usize,
    expands:           bool,
    radius_scale:      f32,
    line_length_scale: f32,
    ring_delay_secs:   f32,
    ring_phase_offset: f32,
    alpha_curve:       AlphaCurve,
}

impl RingEffectConfig {
    const EXPANDING_RING: Self = Self {
        ring_count:        1,
        expands:           true,
        radius_scale:      1.0,
        line_length_scale: 0.5,
        ring_delay_secs:   0.0,
        ring_phase_offset: 0.0,
        alpha_curve:       AlphaCurve::LinearFade,
    };

    const STATIC_FLASH: Self = Self {
        ring_count:        1,
        expands:           false,
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
        expands:           true,
        radius_scale:      1.0,
        line_length_scale: 1.0 / 3.0,
        ring_delay_secs:   0.4,
        ring_phase_offset: 2.0,
        alpha_curve:       AlphaCurve::LinearFade,
    };
}

fn update_thruster_effect(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &ActionState<SpaceshipControl>,
            Option<&ThrusterEffect>,
        ),
        With<Spaceship>,
    >,
) {
    let Ok((entity, controls, thruster_effect)) = query.single() else {
        return;
    };

    let is_accelerating = controls.pressed(&SpaceshipControl::Accelerate);

    match (is_accelerating, thruster_effect) {
        (true, None) => {
            commands.entity(entity).insert(ThrusterEffect);
        },
        (false, Some(_)) => {
            commands.entity(entity).remove::<ThrusterEffect>();
        },
        _ => {},
    }
}

fn draw_thruster_flames(
    mut gizmos: Gizmos<FlameGizmo>,
    time: Res<Time>,
    pause_state: Res<State<PauseState>>,
    mut frozen_elapsed: Local<f32>,
    query: Query<&Transform, With<ThrusterEffect>>,
) {
    let elapsed = if *pause_state.get() == PauseState::Paused {
        *frozen_elapsed
    } else {
        *frozen_elapsed = time.elapsed_secs();
        time.elapsed_secs()
    };

    for transform in query.iter() {
        draw_exhaust_flames(&mut gizmos, transform, elapsed);
    }
}

fn draw_exhaust_flames(gizmos: &mut Gizmos<FlameGizmo>, transform: &Transform, elapsed: f32) {
    let back_direction = -transform.forward().as_vec3();
    let right = transform.right().as_vec3();
    let up = transform.up().as_vec3();

    let base_position = transform.translation + back_direction * THRUSTER_LINE_OFFSET;

    let color_yellow = Color::from(tailwind::YELLOW_400);
    let color_orange = Color::from(tailwind::ORANGE_500);
    let color_red = Color::from(tailwind::RED_600);

    #[allow(clippy::cast_precision_loss)]
    let line_count_f32 = THRUSTER_LINE_COUNT as f32;

    for i in 0..THRUSTER_LINE_COUNT {
        #[allow(clippy::cast_precision_loss)]
        let line_index = i as f32;
        let angle_offset = (line_index - (line_count_f32 - 1.0) / 2.0) / (line_count_f32 / 2.0)
            * THRUSTER_CONE_HALF_ANGLE;

        let phase = line_index * FLAME_PHASE_SPREAD;
        let vibration_lateral =
            elapsed.mul_add(FLAME_VIBRATION_SPEED, phase).sin() * FLAME_VIBRATION_AMPLITUDE;
        let vibration_vertical = (elapsed * FLAME_VIBRATION_SPEED)
            .mul_add(
                THRUSTER_VIBRATION_VERTICAL_SPEED_MULT,
                phase * THRUSTER_VIBRATION_VERTICAL_PHASE_MULT,
            )
            .cos()
            * FLAME_VIBRATION_AMPLITUDE;

        let flicker = compute_flicker(elapsed, line_index, 0.0);
        let line_length = flicker
            .length
            .mul_add(THRUSTER_LINE_LENGTH_VARIANCE, THRUSTER_LINE_LENGTH_BASE);

        let spread_direction = (back_direction + right * angle_offset.sin()).normalize();

        let start = base_position + right * vibration_lateral + up * vibration_vertical;
        let end = start + spread_direction * line_length;

        let center_factor = 1.0 - (angle_offset.abs() / THRUSTER_CONE_HALF_ANGLE);
        let zone = FlameZone::from_center_factor(center_factor, THRUSTER_COLOR_ZONE_SIZE);
        let color = zone.color(elapsed, phase, color_red, color_orange, color_yellow);

        gizmos.line(start, end, color);
    }
}

fn draw_death_effects(
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
    radius: f32,
    line_length_base: f32,
    line_length_variance: f32,
    color_a: Color,
    color_b: Color,
    elapsed: f32,
    phase_offset: f32,
) {
    let position = Vec3::from(isometry.translation);
    let rotation = isometry.rotation;

    #[allow(clippy::cast_precision_loss)]
    let line_count_f32 = DEATH_EFFECT_LINE_COUNT as f32;

    for i in 0..DEATH_EFFECT_LINE_COUNT {
        #[allow(clippy::cast_precision_loss)]
        let line_index = i as f32;

        let angle = std::f32::consts::TAU * line_index / line_count_f32;

        // Ring in XZ plane (Y normal) to align with deaderoid's spin axis
        let radial_local = Vec3::new(angle.cos(), 0.0, angle.sin());
        let tangent_local = Vec3::new(-angle.sin(), 0.0, angle.cos());

        let radial = rotation * radial_local;
        let tangent = rotation * tangent_local;

        let phase = line_index.mul_add(FLAME_PHASE_SPREAD, phase_offset);
        let vibration =
            elapsed.mul_add(FLAME_VIBRATION_SPEED, phase).sin() * FLAME_VIBRATION_AMPLITUDE;

        let flicker = compute_flicker(elapsed, line_index, phase_offset);
        let line_length = flicker
            .length
            .mul_add(line_length_variance, line_length_base);

        let start = position + radial * radius + tangent * vibration;
        let end = start + radial * line_length;

        let color = lerp_color(color_a, color_b, flicker.color);

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
        #[allow(clippy::cast_precision_loss)]
        let ring_idx_f32 = ring_idx as f32;
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

        let radius = if config.expands {
            let ease_out = (1.0 - progress).mul_add(-(1.0 - progress), 1.0);
            let scale = (1.0 - DEATH_EFFECT_EXPANDING_RING_START_SCALE)
                .mul_add(ease_out, DEATH_EFFECT_EXPANDING_RING_START_SCALE);
            death_effect.radius * config.radius_scale * scale
        } else {
            death_effect.radius * config.radius_scale
        };

        let alpha = config.alpha_curve.compute(progress);
        let color_orange = Color::from(tailwind::ORANGE_500).with_alpha(alpha);
        let color_yellow = Color::from(tailwind::YELLOW_400).with_alpha(alpha);

        draw_ring_lines(
            gizmos,
            isometry,
            radius,
            line_length_base,
            line_length_variance,
            color_orange,
            color_yellow,
            elapsed,
            ring_idx_f32 * config.ring_phase_offset,
        );
    }
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::from(a.to_linear().mix(&b.to_linear(), t.clamp(0.0, 1.0)))
}
