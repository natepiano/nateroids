use bevy::math::Dir3;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_kana::Position;

use crate::playfield::boundary_face::BoundaryFace;

#[derive(Resource, Clone, Debug)]
pub struct Portal {
    pub actor_direction:            Vec3,
    pub actor_distance_to_wall:     f32,
    pub boundary_distance_approach: f32,
    pub boundary_distance_shrink:   f32,
    pub face:                       BoundaryFace,
    pub face_count:                 usize,
    pub fade_out_started:           Option<f32>,
    pub position:                   Position,
    pub radius:                     f32,
}

impl Portal {
    /// Returns the normal direction for this portal's face.
    pub const fn normal(&self) -> Dir3 { self.face.to_dir3() }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            actor_direction:            Vec3::ZERO,
            actor_distance_to_wall:     0.,
            boundary_distance_approach: 0.,
            boundary_distance_shrink:   0.,
            face:                       BoundaryFace::Right,
            face_count:                 1,
            fade_out_started:           None,
            position:                   Position::new(0.0, 0.0, 0.0),
            radius:                     0.,
        }
    }
}
