//! Detects whether this GPU can run Bevy's GPU mesh preprocessing, and keeps
//! the renderer consistent when it cannot.

use bevy::prelude::*;
use bevy::render;
use bevy::render::RenderApp;
use bevy::render::RenderStartup;
use bevy::render::batching::gpu_preprocessing::GpuPreprocessingMode;
use bevy::render::batching::gpu_preprocessing::GpuPreprocessingSupport;
use bevy::tasks;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::PowerPreference;
use wgpu::RequestAdapterOptions;

use crate::constants::MESH_PREPROCESSING_STORAGE_BUFFERS_PER_SHADER_STAGE;

/// Tells the render phases that GPU preprocessing is off, matching the
/// `PbrPlugin::use_gpu_instance_buffer_builder` value [`MeshPreprocessing::detect`]
/// chose.
pub(crate) struct MeshPreprocessingPlugin {
    pub(crate) mesh_preprocessing: MeshPreprocessing,
}

impl Plugin for MeshPreprocessingPlugin {
    fn build(&self, app: &mut App) {
        if self.mesh_preprocessing == MeshPreprocessing::Gpu {
            return;
        }

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.add_systems(
            RenderStartup,
            disable_gpu_preprocessing.after(render::init_gpu_resource::<GpuPreprocessingSupport>),
        );
    }
}

/// Where `MeshUniform`s are built, which `PbrPlugin::use_gpu_instance_buffer_builder`
/// selects.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeshPreprocessing {
    /// Compute shaders write `MeshUniform`s and cull on the GPU.
    Gpu,
    /// Systems write `MeshUniform`s on the CPU.
    Cpu,
}

impl MeshPreprocessing {
    /// Asks `wgpu` for the adapter `RenderPlugin` will select and reads its
    /// `max_storage_buffers_per_shader_stage`.
    ///
    /// `prepare_preprocess_pipelines` builds the GPU frustum culling bind group
    /// layout on every device that supports compute, without checking that
    /// limit first. Below
    /// [`MESH_PREPROCESSING_STORAGE_BUFFERS_PER_SHADER_STAGE`] the layout fails
    /// validation and `RenderErrorHandler` writes `AppExit::error()`, so the
    /// window closes during the first frame. The Honeykrisp Vulkan driver on
    /// Apple silicon reports six.
    ///
    /// A failed adapter request leaves the choice to Bevy — `RenderPlugin` hits
    /// the same failure and reports it with more context than this probe can.
    pub(crate) fn detect() -> Self {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle_from_env());
        let request_adapter_options = RequestAdapterOptions {
            power_preference:       PowerPreference::from_env()
                .unwrap_or(PowerPreference::HighPerformance),
            force_fallback_adapter: false,
            compatible_surface:     None,
        };

        let Ok(adapter) = tasks::block_on(instance.request_adapter(&request_adapter_options))
        else {
            return Self::Gpu;
        };

        if adapter.limits().max_storage_buffers_per_shader_stage
            >= MESH_PREPROCESSING_STORAGE_BUFFERS_PER_SHADER_STAGE
        {
            Self::Gpu
        } else {
            Self::Cpu
        }
    }
}

/// `GpuPreprocessingSupport` is derived from device limits alone, so it still
/// reports `PreprocessingOnly` on a device whose storage buffer limit forced
/// [`MeshPreprocessing::Cpu`]. `BinnedRenderPhase` reads it to pick its batch
/// sets, and `Direct` batch sets are the ones CPU batching rejects with
/// "Dynamic uniform batch sets should be used when GPU preprocessing is off" —
/// dropping every mesh from the phase.
fn disable_gpu_preprocessing(mut gpu_preprocessing_support: ResMut<GpuPreprocessingSupport>) {
    gpu_preprocessing_support.max_supported_mode = GpuPreprocessingMode::None;
}
