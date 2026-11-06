use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use hana_kana::ToF32;

use super::constants::STAR_TWINKLE_PHASE_REBASE;
use super::star_material::StarData;
use super::star_material::StarMaterial;
use super::stars::StarSettings;

pub(super) struct StarTwinklingPlugin;

impl Plugin for StarTwinklingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<StarMaterial>::default())
            .init_resource::<StarTwinkling>()
            .add_systems(Update, update_twinkling);
    }
}

/// CPU copy used only when spawning or rebasing the shared GPU clock.
#[derive(Resource, Default)]
pub(super) struct StarTwinkling {
    pub(super) stars:        Vec<StarData>,
    pub(super) star_buffer:  Handle<ShaderBuffer>,
    pub(super) clock_buffer: Handle<ShaderBuffer>,
    phase:                   f64,
}

impl StarTwinkling {
    pub(super) const fn reset(&mut self) { self.phase = 0.0; }

    /// Integrating speed preserves phase when the live speed setting changes.
    fn advance(&mut self, delta_secs: f64, speed: f32) -> Option<f64> {
        self.phase = delta_secs.mul_add(f64::from(speed), self.phase);
        if self.phase.abs() < STAR_TWINKLE_PHASE_REBASE {
            return None;
        }
        let phase = self.phase;
        self.phase = 0.0;
        Some(phase)
    }
}

fn update_twinkling(
    time: Res<Time>,
    star_settings: Res<StarSettings>,
    mut star_twinkling: ResMut<StarTwinkling>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
) {
    if star_twinkling.stars.is_empty() {
        return;
    }
    if let Some(phase) = star_twinkling.advance(time.delta_secs_f64(), star_settings.twinkle.speed)
    {
        for star in &mut star_twinkling.stars {
            star.rebase(phase);
        }
        if let Some(mut buffer) = buffers.get_mut(&star_twinkling.star_buffer) {
            buffer.clear();
            buffer.extend_from_slice(&star_twinkling.stars);
        }
    }
    if let Some(mut buffer) = buffers.get_mut(&star_twinkling.clock_buffer) {
        buffer.clear();
        buffer.extend([Vec4::new(
            star_twinkling.phase.to_f32(),
            star_settings.twinkle.amplitude,
            0.0,
            0.0,
        )]);
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use bevy::prelude::*;

    use super::StarTwinkling;
    use crate::camera::constants::STAR_TWINKLE_PHASE_REBASE;
    use crate::camera::star_material::StarData;

    const INITIAL_PHASE: f32 = 0.7;
    const AMPLITUDE_FRACTION: f32 = 0.8;
    const SPEED_FRACTION: f32 = 1.3;
    const PHASE_TOLERANCE: f64 = 0.00001;

    fn star() -> StarData {
        StarData {
            emissive: Vec4::ONE,
            twinkle:  Vec4::new(INITIAL_PHASE, AMPLITUDE_FRACTION, SPEED_FRACTION, 0.0),
        }
    }

    fn sine(star: &StarData, phase: f64) -> f64 {
        f64::from(star.twinkle.z)
            .mul_add(phase, f64::from(star.twinkle.x))
            .sin()
    }

    #[test]
    fn speed_edits_and_zero_speed_preserve_existing_phase() {
        let mut clock = StarTwinkling::default();
        let star = star();
        assert_eq!(clock.advance(2.0, 3.0), None);
        let before = sine(&star, clock.phase);
        assert_eq!(clock.advance(0.0, 10.0), None);
        assert!((sine(&star, clock.phase) - before).abs() < PHASE_TOLERANCE);
        assert_eq!(clock.advance(100.0, 0.0), None);
        assert!((sine(&star, clock.phase) - before).abs() < PHASE_TOLERANCE);
        assert_eq!(clock.advance(0.5, 10.0), None);
        let expected = f64::from(SPEED_FRACTION)
            .mul_add(11.0, f64::from(INITIAL_PHASE))
            .sin();
        assert!((sine(&star, clock.phase) - expected).abs() < PHASE_TOLERANCE);
    }

    #[test]
    fn rebase_preserves_each_stars_brightness_and_subsequent_motion() {
        let mut clock = StarTwinkling::default();
        let mut star = star();
        let original = star;
        let elapsed = STAR_TWINKLE_PHASE_REBASE + 1.0;
        assert_eq!(clock.advance(elapsed, 1.0), Some(elapsed));
        star.rebase(elapsed);
        assert!(clock.phase.abs() < PHASE_TOLERANCE);
        assert!((0.0..TAU).contains(&f64::from(star.twinkle.x)));
        assert!((sine(&star, clock.phase) - sine(&original, elapsed)).abs() < PHASE_TOLERANCE);
        assert_eq!(clock.advance(0.25, 3.0), None);
        assert!(
            (sine(&star, clock.phase) - sine(&original, elapsed + 0.75)).abs() < PHASE_TOLERANCE
        );
        assert_eq!(star.emissive, original.emissive);
        assert!((star.twinkle.y - original.twinkle.y).abs() < f32::EPSILON);
        assert!((star.twinkle.z - original.twinkle.z).abs() < f32::EPSILON);
    }
}
