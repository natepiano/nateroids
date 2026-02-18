use bevy::prelude::*;
use bevy::render::view::Hdr;

/// Required components shared across all cameras.
///
/// `Msaa::Off` is required by `bevy_mesh_outline`. Like `Hdr`, `Msaa` must be
/// consistent across all cameras — mismatched values silently break rendering.
/// See <https://github.com/bevyengine/bevy/issues/15467>.
#[derive(Component, Default)]
#[require( Msaa = msaa_setting(),   Hdr )]
pub struct RequiredCameraComponents;

const fn msaa_setting() -> Msaa { Msaa::Off }
