use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy::render::view::Hdr;

/// Required components shared across all cameras.
/// Adding this component automatically inserts `Hdr`.
#[derive(Component, Default)]
#[require(Hdr, DepthPrepass)]
pub struct RequiredCameraComponents;
