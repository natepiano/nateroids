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
    pub fn get_normal(&self) -> Vec3 {
        match self {
            BoundaryFace::Left => Vec3::NEG_X,
            BoundaryFace::Right => Vec3::X,
            BoundaryFace::Top => Vec3::Y,
            BoundaryFace::Bottom => Vec3::NEG_Y,
            BoundaryFace::Front => Vec3::Z,
            BoundaryFace::Back => Vec3::NEG_Z,
        }
    }

    /// Infallible conversion to `Dir3` - all boundary faces are axis-aligned
    pub fn to_dir3(&self) -> Dir3 {
        match self {
            BoundaryFace::Right => Dir3::X,
            BoundaryFace::Left => Dir3::NEG_X,
            BoundaryFace::Top => Dir3::Y,
            BoundaryFace::Bottom => Dir3::NEG_Y,
            BoundaryFace::Front => Dir3::Z,
            BoundaryFace::Back => Dir3::NEG_Z,
        }
    }

    pub fn from_normal(normal: Dir3) -> Option<Self> {
        match normal {
            Dir3::X => Some(BoundaryFace::Right),
            Dir3::NEG_X => Some(BoundaryFace::Left),
            Dir3::Y => Some(BoundaryFace::Top),
            Dir3::NEG_Y => Some(BoundaryFace::Bottom),
            Dir3::Z => Some(BoundaryFace::Front),
            Dir3::NEG_Z => Some(BoundaryFace::Back),
            _ => None,
        }
    }

    pub fn get_face_points(&self, min: &Vec3, max: &Vec3) -> [Vec3; 4] {
        match self {
            BoundaryFace::Left => [
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(min.x, max.y, min.z),
                Vec3::new(min.x, max.y, max.z),
                Vec3::new(min.x, min.y, max.z),
            ],
            BoundaryFace::Right => [
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(max.x, min.y, max.z),
            ],
            BoundaryFace::Bottom => [
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, min.y, max.z),
                Vec3::new(min.x, min.y, max.z),
            ],
            BoundaryFace::Top => [
                Vec3::new(min.x, max.y, min.z),
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(min.x, max.y, max.z),
            ],
            BoundaryFace::Back => [
                Vec3::new(min.x, min.y, min.z),
                Vec3::new(max.x, min.y, min.z),
                Vec3::new(max.x, max.y, min.z),
                Vec3::new(min.x, max.y, min.z),
            ],
            BoundaryFace::Front => [
                Vec3::new(min.x, min.y, max.z),
                Vec3::new(max.x, min.y, max.z),
                Vec3::new(max.x, max.y, max.z),
                Vec3::new(min.x, max.y, max.z),
            ],
        }
    }
}
