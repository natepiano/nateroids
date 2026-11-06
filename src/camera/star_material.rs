use std::f32::consts::TAU as TAU_F32;
use std::f64::consts::TAU;

use bevy::pbr::ExtendedMaterial;
use bevy::pbr::MaterialExtension;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use bytemuck::Pod;
use bytemuck::Zeroable;
use hana_kana::ToF32;
use rand::RngExt;

use super::constants::STAR_SHADER_PATH;
use super::constants::STAR_TWINKLE_AMPLITUDE_FRACTION_MAX;
use super::constants::STAR_TWINKLE_AMPLITUDE_FRACTION_MIN;
use super::constants::STAR_TWINKLE_SPEED_FRACTION_MAX;
use super::constants::STAR_TWINKLE_SPEED_FRACTION_MIN;

pub(super) type StarMaterial = ExtendedMaterial<StandardMaterial, StarMaterialExtension>;

/// One material for the starfield; `MeshTag` selects each star's buffer entry.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct StarMaterialExtension {
    #[storage(100, read_only)]
    pub(super) stars: Handle<ShaderBuffer>,
    #[storage(101, read_only)]
    pub(super) clock: Handle<ShaderBuffer>,
}

impl MaterialExtension for StarMaterialExtension {
    fn vertex_shader() -> ShaderRef { STAR_SHADER_PATH.into() }

    fn fragment_shader() -> ShaderRef { STAR_SHADER_PATH.into() }
}

/// Two aligned vectors shared by Rust and `star.wesl`.
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub(super) struct StarData {
    pub(super) emissive: Vec4,
    /// Initial phase, amplitude fraction, speed fraction, and alignment padding.
    pub(super) twinkle:  Vec4,
}

impl StarData {
    pub(super) fn random(emissive: Vec4, rng: &mut impl RngExt) -> Self {
        Self {
            emissive,
            twinkle: Vec4::new(
                rng.random_range(0.0..TAU_F32),
                rng.random_range(
                    STAR_TWINKLE_AMPLITUDE_FRACTION_MIN..STAR_TWINKLE_AMPLITUDE_FRACTION_MAX,
                ),
                rng.random_range(STAR_TWINKLE_SPEED_FRACTION_MIN..STAR_TWINKLE_SPEED_FRACTION_MAX),
                0.0,
            ),
        }
    }

    /// Preserve the sine phase when the shared clock resets to keep GPU floats precise.
    pub(super) fn rebase(&mut self, phase: f64) {
        self.twinkle.x = f64::from(self.twinkle.z)
            .mul_add(phase, f64::from(self.twinkle.x))
            .rem_euclid(TAU)
            .to_f32();
    }
}
