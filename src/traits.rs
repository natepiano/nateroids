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

/// Extension trait for `usize` to provide safe f32 conversion for game-scale values
pub trait UsizeExt {
    /// Converts `usize` to `f32` for game-scale values (safe for values < 16 million)
    fn to_f32(self) -> f32;
}

impl UsizeExt for usize {
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn to_f32(self) -> f32 { self as f32 }
}
