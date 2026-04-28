use bevy::prelude::*;
use bevy::render::view::Hdr;

/// Required components shared across all cameras.
///
/// `Hdr` and `Msaa` must be consistent across all cameras — mismatched values
/// silently break rendering.
/// See <https://github.com/bevyengine/bevy/issues/15467>.
#[derive(Component, Default)]
#[require(Hdr)]
pub(super) struct RequiredCameraComponents;
