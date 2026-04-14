use bevy::prelude::*;
use bevy_kana::Position;

use super::Boundary;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::BOUNDARY_SNAP_EPSILON;

impl Boundary {
    /// Finds the intersection point of a ray (defined by an origin and
    /// direction) with the edges of a viewable area.
    ///
    /// # Parameters
    /// - `origin`: The starting point of the ray.
    /// - `direction`: The direction vector of the ray.
    /// - `dimensions`: The dimensions of the viewable area.
    ///
    /// # Returns
    /// An `Option<Vec3>` containing the intersection point if found, or `None`
    /// if no valid intersection exists.
    ///
    /// # Method
    /// - The function calculates the intersection points of the ray with the positive and negative
    ///   boundaries of the viewable area along all axes. todo: is this true? you'll have to test in
    ///   3d mode
    /// - It iterates over these axes, updating the minimum intersection distance (`t_min`) if a
    ///   valid intersection is found.
    /// - Finally, it returns the intersection point corresponding to the minimum distance, or
    ///   `None` if no valid intersection is found.
    pub fn calculate_teleport_position(position: Position, transform: &Transform) -> Position {
        let boundary_min = transform.translation - transform.scale / 2.0;
        let boundary_max = transform.translation + transform.scale / 2.0;

        let mut teleport_position = *position;

        if position.x >= boundary_max.x {
            let offset = position.x - boundary_max.x;
            teleport_position.x = boundary_min.x + offset;
        } else if position.x <= boundary_min.x {
            let offset = boundary_min.x - position.x;
            teleport_position.x = boundary_max.x - offset;
        }

        if position.y >= boundary_max.y {
            let offset = position.y - boundary_max.y;
            teleport_position.y = boundary_min.y + offset;
        } else if position.y <= boundary_min.y {
            let offset = boundary_min.y - position.y;
            teleport_position.y = boundary_max.y - offset;
        }

        if position.z >= boundary_max.z {
            let offset = position.z - boundary_max.z;
            teleport_position.z = boundary_min.z + offset;
        } else if position.z <= boundary_min.z {
            let offset = boundary_min.z - position.z;
            teleport_position.z = boundary_max.z - offset;
        }

        Position(teleport_position)
    }

    /// Snaps a position to slightly inside the boundary face based on the normal.
    /// Offsets by epsilon to prevent false-positive overextension detection that would trigger
    /// corner wrapping arcs. Clamps perpendicular axes to handle corner/edge teleportation cases.
    pub fn snap_position_to_boundary_face(
        position: Position,
        normal: Dir3,
        transform: &Transform,
    ) -> Position {
        let boundary_min = transform.translation - transform.scale / 2.0;
        let boundary_max = transform.translation + transform.scale / 2.0;

        // Without this offset, portals on exact boundary would be flagged as overextended
        let epsilon = BOUNDARY_SNAP_EPSILON;

        let mut snapped_position = *position;

        // Set primary axis slightly inside boundary face and clamp perpendicular axes
        match normal {
            Dir3::X => {
                snapped_position.x = boundary_max.x - epsilon;
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            Dir3::NEG_X => {
                snapped_position.x = boundary_min.x + epsilon;
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            Dir3::Y => {
                snapped_position.y = boundary_max.y - epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            Dir3::NEG_Y => {
                snapped_position.y = boundary_min.y + epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            Dir3::Z => {
                snapped_position.z = boundary_max.z - epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
            },
            Dir3::NEG_Z => {
                snapped_position.z = boundary_min.z + epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
            },
            _ => {},
        }

        Position(snapped_position)
    }

    /// Returns the normal of the closest boundary face to a position.
    /// Uses distance-based matching because teleported positions have offsets (e.g., -54.97 instead
    /// of -55.0) that break simple epsilon matching.
    pub fn get_normal_for_position(position: Position, transform: &Transform) -> Dir3 {
        let half_size = transform.scale / 2.0;
        let boundary_min = transform.translation - half_size;
        let boundary_max = transform.translation + half_size;

        // Calculate distance to all 6 faces and return normal of closest
        let faces = [
            ((position.x - boundary_min.x).abs(), BoundaryFace::Left),
            ((position.x - boundary_max.x).abs(), BoundaryFace::Right),
            ((position.y - boundary_min.y).abs(), BoundaryFace::Bottom),
            ((position.y - boundary_max.y).abs(), BoundaryFace::Top),
            ((position.z - boundary_min.z).abs(), BoundaryFace::Back),
            ((position.z - boundary_max.z).abs(), BoundaryFace::Front),
        ];
        faces
            .iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map_or(Dir3::Y, |(_, face)| face.to_dir3())
    }

    pub fn find_edge_point(
        origin: Position,
        direction: Vec3,
        transform: &Transform,
    ) -> Option<Position> {
        let boundary_min = transform.translation - transform.scale / 2.0;
        let boundary_max = transform.translation + transform.scale / 2.0;

        let mut t_min: Option<f32> = None;

        for (start, dir, pos_bound, neg_bound) in [
            (origin.x, direction.x, boundary_max.x, boundary_min.x),
            (origin.y, direction.y, boundary_max.y, boundary_min.y),
            (origin.z, direction.z, boundary_max.z, boundary_min.z),
        ] {
            if dir != 0.0 {
                let mut update_t_min = |boundary: f32| {
                    let t = (boundary - start) / dir;
                    let point = origin + direction * t;
                    if t > 0.0
                        && t_min.is_none_or(|current| t < current)
                        && is_in_bounds(point, start, origin, boundary_min, boundary_max)
                    {
                        t_min = Some(t);
                    }
                };

                update_t_min(pos_bound);
                update_t_min(neg_bound);
            }
        }

        t_min.map(|t| origin + direction * t)
    }

    pub fn longest_diagonal(&self) -> f32 {
        let boundary_scale = self.scale();
        let x = boundary_scale.x;
        let y = boundary_scale.y;
        let z = boundary_scale.z;
        // FMA optimization (faster + more precise): (x² + y² + z²).sqrt()
        z.mul_add(z, y.mul_add(y, x.mul_add(x, 0.0))).sqrt()
    }

    pub fn max_missile_distance(&self) -> f32 {
        let boundary_scale = self.scale();
        boundary_scale.x.max(boundary_scale.y).max(boundary_scale.z)
    }

    pub fn scale(&self) -> Vec3 { self.exterior_scalar * self.cell_count.as_vec3() }

    /// Returns the 8 corner points of the boundary as a fixed-size array
    pub fn corners(&self) -> [Vec3; 8] {
        let grid_size = self.scale();
        let half_size = grid_size / 2.0;
        [
            Vec3::new(-half_size.x, -half_size.y, -half_size.z),
            Vec3::new(half_size.x, -half_size.y, -half_size.z),
            Vec3::new(-half_size.x, half_size.y, -half_size.z),
            Vec3::new(half_size.x, half_size.y, -half_size.z),
            Vec3::new(-half_size.x, -half_size.y, half_size.z),
            Vec3::new(half_size.x, -half_size.y, half_size.z),
            Vec3::new(-half_size.x, half_size.y, half_size.z),
            Vec3::new(half_size.x, half_size.y, half_size.z),
        ]
    }
}

fn is_in_bounds(
    point: Position,
    start: f32,
    origin: Position,
    boundary_min: Vec3,
    boundary_max: Vec3,
) -> bool {
    if (start - origin.x).abs() < BOUNDARY_SNAP_EPSILON {
        point.y >= boundary_min.y
            && point.y <= boundary_max.y
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else if (start - origin.y).abs() < BOUNDARY_SNAP_EPSILON {
        point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else {
        point.x >= boundary_min.x
            && point.x <= boundary_max.x
            && point.y >= boundary_min.y
            && point.y <= boundary_max.y
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "test assertions compare exact known float values"
)]
mod tests {
    use super::*;

    /// Helper function to create a test transform centered at origin with given size
    fn create_test_transform(size: Vec3) -> Transform {
        Transform {
            translation: Vec3::ZERO,
            scale: size,
            ..default()
        }
    }

    #[test]
    fn test_no_teleport_when_inside_boundary() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Test positions well inside boundary
        let inside_positions = vec![
            Position::new(0.0, 0.0, 0.0),
            Position::new(10.0, 0.0, 0.0),
            Position::new(0.0, 20.0, 0.0),
            Position::new(0.0, 0.0, 30.0),
            Position::new(-10.0, -20.0, -30.0),
        ];

        for pos in inside_positions {
            let result = Boundary::calculate_teleport_position(pos, &transform);
            assert_eq!(
                result, pos,
                "Position {pos:?} inside boundary should not be teleported"
            );
        }
    }

    #[test]
    fn test_teleport_right_face_to_left() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        // boundary_max.x = 50.0, boundary_min.x = -50.0

        // Entity exits right face (+X) at x=55.0 (5.0 past boundary)
        let position = Position::new(55.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should wrap to left face at x=-45.0 (5.0 offset from left boundary)
        assert_eq!(result.x, -45.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_left_face_to_right() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits left face (-X) at x=-60.0 (10.0 past boundary)
        let position = Position::new(-60.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should wrap to right face at x=40.0 (10.0 offset from right boundary)
        assert_eq!(result.x, 40.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_top_face_to_bottom() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits top face (+Y) at y=53.0 (3.0 past boundary)
        let position = Position::new(0.0, 53.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should wrap to bottom face at y=-47.0 (3.0 offset from bottom boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, -47.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_bottom_face_to_top() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits bottom face (-Y) at y=-58.0 (8.0 past boundary)
        let position = Position::new(0.0, -58.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should wrap to top face at y=42.0 (8.0 offset from top boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 42.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_front_face_to_back() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits front face (+Z) at z=52.0 (2.0 past boundary)
        let position = Position::new(0.0, 0.0, 52.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should wrap to back face at z=-48.0 (2.0 offset from back boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, -48.0);
    }

    #[test]
    fn test_teleport_back_face_to_front() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits back face (-Z) at z=-57.0 (7.0 past boundary)
        let position = Position::new(0.0, 0.0, -57.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should wrap to front face at z=43.0 (7.0 offset from front boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 43.0);
    }

    #[test]
    fn test_teleport_preserves_offset_on_other_axes() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits right face with Y and Z offsets
        let position = Position::new(55.0, 20.0, -10.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // X should wrap, but Y and Z should remain unchanged
        assert_eq!(result.x, -45.0);
        assert_eq!(result.y, 20.0);
        assert_eq!(result.z, -10.0);
    }

    #[test]
    fn test_teleport_edge_wrapping() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits both right face (+X) and top face (+Y)
        let position = Position::new(53.0, 52.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Both axes should wrap independently
        assert_eq!(result.x, -47.0); // Wrapped from right to left
        assert_eq!(result.y, -48.0); // Wrapped from top to bottom
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_corner_wrapping() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits all three faces (corner case)
        let position = Position::new(55.0, 58.0, 52.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // All three axes should wrap independently
        assert_eq!(result.x, -45.0);
        assert_eq!(result.y, -42.0);
        assert_eq!(result.z, -48.0);
    }

    #[test]
    fn test_teleport_large_offset() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Entity far past boundary (offset = 150.0, larger than boundary itself)
        let position = Position::new(200.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // Should maintain the full offset from opposite boundary
        // offset = 200.0 - 50.0 = 150.0
        // result = -50.0 + 150.0 = 100.0
        assert_eq!(result.x, 100.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_with_non_centered_boundary() {
        // Boundary centered at (100, 50, -25) with size (200, 100, 50)
        let transform = Transform {
            translation: Vec3::new(100.0, 50.0, -25.0),
            scale: Vec3::new(200.0, 100.0, 50.0),
            ..default()
        };
        // boundary_min = (0, 0, -50), boundary_max = (200, 100, 0)

        // Test right face wrap
        let position = Position::new(205.0, 50.0, -25.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_eq!(result.x, 5.0); // Offset 5.0 from left boundary
        assert_eq!(result.y, 50.0);
        assert_eq!(result.z, -25.0);

        // Test top face wrap
        let position = Position::new(100.0, 103.0, -25.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_eq!(result.x, 100.0);
        assert_eq!(result.y, 3.0); // Offset 3.0 from bottom boundary
        assert_eq!(result.z, -25.0);
    }

    #[test]
    fn test_teleport_exactly_at_boundary() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));

        // Position exactly at boundary (should not wrap, as condition is >= not >)
        let position = Position::new(50.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        // At x=50.0 (boundary_max), should wrap to boundary_min
        assert_eq!(result.x, -50.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_asymmetric_boundary() {
        // Test with different dimensions on each axis
        let transform = create_test_transform(Vec3::new(200.0, 50.0, 80.0));
        // boundary_min = (-100, -25, -40), boundary_max = (100, 25, 40)

        // Test X axis (larger)
        let position = Position::new(110.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_eq!(result.x, -90.0);

        // Test Y axis (smaller)
        let position = Position::new(0.0, 30.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_eq!(result.y, -20.0);

        // Test Z axis (medium)
        let position = Position::new(0.0, 0.0, -45.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_eq!(result.z, 35.0);
    }
}
