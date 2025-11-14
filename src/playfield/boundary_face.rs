use bevy::math::Dir3;
use bevy::math::Vec3;
use bevy::prelude::Reflect;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Reflect)]
pub enum BoundaryFace {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

impl BoundaryFace {
    pub const fn get_normal(self) -> Vec3 {
        match self {
            Self::Left => Vec3::NEG_X,
            Self::Right => Vec3::X,
            Self::Top => Vec3::Y,
            Self::Bottom => Vec3::NEG_Y,
            Self::Front => Vec3::Z,
            Self::Back => Vec3::NEG_Z,
        }
    }

    /// Infallible conversion to `Dir3` - all boundary faces are axis-aligned
    pub const fn to_dir3(self) -> Dir3 {
        match self {
            Self::Right => Dir3::X,
            Self::Left => Dir3::NEG_X,
            Self::Top => Dir3::Y,
            Self::Bottom => Dir3::NEG_Y,
            Self::Front => Dir3::Z,
            Self::Back => Dir3::NEG_Z,
        }
    }

    pub fn from_normal(normal: Dir3) -> Option<Self> {
        match normal {
            Dir3::X => Some(Self::Right),
            Dir3::NEG_X => Some(Self::Left),
            Dir3::Y => Some(Self::Top),
            Dir3::NEG_Y => Some(Self::Bottom),
            Dir3::Z => Some(Self::Front),
            Dir3::NEG_Z => Some(Self::Back),
            _ => None,
        }
    }

    pub const fn get_face_points(self, min: &Vec3, max: &Vec3) -> [Vec3; 4] {
        match self {
            Self::Left => [
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(min.x, max.y, min.z),
                Vec3::new(min.x, max.y, max.z),
                Vec3::new(min.x, min.y, max.z),
            ],
            Self::Right => [
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(max.x, min.y, max.z),
            ],
            Self::Bottom => [
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, min.y, max.z),
                Vec3::new(min.x, min.y, max.z),
            ],
            Self::Top => [
                Vec3::new(min.x, max.y, min.z),
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(min.x, max.y, max.z),
            ],
            Self::Back => [
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(min.x, max.y, min.z),
            ],
            Self::Front => [
                Vec3::new(min.x, min.y, max.z),
                Vec3::new(max.x, min.y, max.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(min.x, max.y, max.z),
            ],
        }
    }
}
