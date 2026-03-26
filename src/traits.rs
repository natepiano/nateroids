use bevy::prelude::*;

/// Extension trait for `Transform` to provide more concise construction methods
pub trait TransformExt {
    /// Creates a `Transform` from translation, rotation, and scale in one call
    fn from_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self;
}

impl TransformExt for Transform {
    fn from_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }
}
