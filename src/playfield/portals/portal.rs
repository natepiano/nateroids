use bevy::math::Dir3;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_kana::Position;

use super::constants::DEFAULT_PORTAL_FACE_COUNT;
use crate::playfield::boundary_face::BoundaryFace;

#[derive(Resource, Clone, Debug)]
pub(crate) struct Portal {
    pub(crate) actor_direction:            Vec3,
    pub(crate) actor_distance_to_wall:     f32,
    pub(crate) boundary_distance_approach: f32,
    pub(crate) boundary_distance_shrink:   f32,
    pub(crate) boundary_face:              BoundaryFace,
    pub(crate) face_count:                 usize,
    pub(crate) fade_out_started:           Option<f32>,
    pub(crate) position:                   Position,
    pub(crate) radius:                     f32,
}

impl Portal {
    /// Returns the normal direction for this portal's face.
    pub const fn normal(&self) -> Dir3 { self.boundary_face.to_dir3() }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            actor_direction:            Vec3::ZERO,
            actor_distance_to_wall:     0.,
            boundary_distance_approach: 0.,
            boundary_distance_shrink:   0.,
            boundary_face:              BoundaryFace::Right,
            face_count:                 DEFAULT_PORTAL_FACE_COUNT,
            fade_out_started:           None,
            position:                   Position::new(0.0, 0.0, 0.0),
            radius:                     0.,
        }
    }
}
