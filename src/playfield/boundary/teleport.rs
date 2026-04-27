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

    /// Snaps a position to slightly inside the given boundary face.
    /// Offsets by epsilon to prevent false-positive overextension detection that would trigger
    /// corner wrapping arcs. Clamps perpendicular axes to handle corner/edge teleportation cases.
    pub(crate) fn snap_position_to_boundary_face(
        position: Position,
        face: BoundaryFace,
        transform: &Transform,
    ) -> Position {
        let boundary_min = transform.translation - transform.scale / 2.0;
        let boundary_max = transform.translation + transform.scale / 2.0;

        // Without this offset, portals on exact boundary would be flagged as overextended
        let epsilon = BOUNDARY_SNAP_EPSILON;

        let mut snapped_position = *position;

        // Set primary axis slightly inside boundary face and clamp perpendicular axes
        match face {
            BoundaryFace::Right => {
                snapped_position.x = boundary_max.x - epsilon;
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            BoundaryFace::Left => {
                snapped_position.x = boundary_min.x + epsilon;
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            BoundaryFace::Top => {
                snapped_position.y = boundary_max.y - epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            BoundaryFace::Bottom => {
                snapped_position.y = boundary_min.y + epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.z = snapped_position.z.clamp(boundary_min.z, boundary_max.z);
            },
            BoundaryFace::Front => {
                snapped_position.z = boundary_max.z - epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
            },
            BoundaryFace::Back => {
                snapped_position.z = boundary_min.z + epsilon;
                snapped_position.x = snapped_position.x.clamp(boundary_min.x, boundary_max.x);
                snapped_position.y = snapped_position.y.clamp(boundary_min.y, boundary_max.y);
            },
        }

        Position(snapped_position)
    }

    /// Returns the closest boundary face to a position.
    /// Uses distance-based matching because teleported positions have offsets (e.g., -54.97 instead
    /// of -55.0) that break simple epsilon matching.
    pub(crate) fn get_face_for_position(position: Position, transform: &Transform) -> BoundaryFace {
        let half_size = transform.scale / 2.0;
        let boundary_min = transform.translation - half_size;
        let boundary_max = transform.translation + half_size;

        // Calculate distance to all 6 faces and return the closest
        let faces = [
            ((position.x - boundary_min.x).abs(), BoundaryFace::Left),
            ((position.x - boundary_max.x).abs(), BoundaryFace::Right),
            ((position.y - boundary_min.y).abs(), BoundaryFace::Bottom),
            ((position.y - boundary_max.y).abs(), BoundaryFace::Top),
            ((position.z - boundary_min.z).abs(), BoundaryFace::Back),
            ((position.z - boundary_max.z).abs(), BoundaryFace::Front),
        ];
        faces[1..]
            .iter()
            .fold(faces[0], |acc, &cur| if cur.0 < acc.0 { cur } else { acc })
            .1
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
mod tests {
    use super::*;

    const FLOAT_EPSILON: f32 = 0.000_001;

    /// Helper function to create a test transform centered at origin with given size
    fn create_test_transform(size: Vec3) -> Transform {
        Transform {
            translation: Vec3::ZERO,
            scale: size,
            ..default()
        }
    }

    fn boundary_extents(transform: &Transform) -> (Vec3, Vec3) {
        let half_size = transform.scale / 2.0;
        (
            transform.translation - half_size,
            transform.translation + half_size,
        )
    }

    fn wrap_from_max_axis(position: f32, axis_min: f32, axis_max: f32) -> f32 {
        axis_min + (position - axis_max)
    }

    fn wrap_from_min_axis(position: f32, axis_min: f32, axis_max: f32) -> f32 {
        axis_max - (axis_min - position)
    }

    fn assert_float_eq(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= FLOAT_EPSILON,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_position_eq(actual: Position, expected: Position) {
        assert_float_eq(actual.x, expected.x);
        assert_float_eq(actual.y, expected.y);
        assert_float_eq(actual.z, expected.z);
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
            assert_position_eq(result, pos);
        }
    }

    #[test]
    fn test_teleport_right_face_to_left() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits right face (+X) at x=55.0 (5.0 past boundary)
        let position = Position::new(55.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_left_face_to_right() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits left face (-X) at x=-60.0 (10.0 past boundary)
        let position = Position::new(-60.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_min_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_top_face_to_bottom() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits top face (+Y) at y=53.0 (3.0 past boundary)
        let position = Position::new(0.0, 53.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_bottom_face_to_top() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits bottom face (-Y) at y=-58.0 (8.0 past boundary)
        let position = Position::new(0.0, -58.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_min_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_front_face_to_back() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits front face (+Z) at z=52.0 (2.0 past boundary)
        let position = Position::new(0.0, 0.0, 52.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                position.y,
                wrap_from_max_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }

    #[test]
    fn test_teleport_back_face_to_front() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits back face (-Z) at z=-57.0 (7.0 past boundary)
        let position = Position::new(0.0, 0.0, -57.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                position.x,
                position.y,
                wrap_from_min_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }

    #[test]
    fn test_teleport_preserves_offset_on_other_axes() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits right face with Y and Z offsets
        let position = Position::new(55.0, 20.0, -10.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_edge_wrapping() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits both right face (+X) and top face (+Y)
        let position = Position::new(53.0, 52.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_corner_wrapping() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity exits all three faces (corner case)
        let position = Position::new(55.0, 58.0, 52.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                wrap_from_max_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }

    #[test]
    fn test_teleport_large_offset() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Entity far past boundary (offset = 150.0, larger than boundary itself)
        let position = Position::new(200.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_with_non_centered_boundary() {
        // Boundary centered at (100, 50, -25) with size (200, 100, 50)
        let transform = Transform {
            translation: Vec3::new(100.0, 50.0, -25.0),
            scale: Vec3::new(200.0, 100.0, 50.0),
            ..default()
        };
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Test right face wrap
        let position = Position::new(205.0, 50.0, -25.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );

        // Test top face wrap
        let position = Position::new(100.0, 103.0, -25.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );
    }

    #[test]
    fn test_teleport_exactly_at_boundary() {
        let transform = create_test_transform(Vec3::new(100.0, 100.0, 100.0));
        let (boundary_min, _) = boundary_extents(&transform);

        // Position exactly at boundary (should not wrap, as condition is >= not >)
        let position = Position::new(50.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);

        assert_position_eq(
            result,
            Position::new(boundary_min.x, position.y, position.z),
        );
    }

    #[test]
    fn test_teleport_asymmetric_boundary() {
        // Test with different dimensions on each axis
        let transform = create_test_transform(Vec3::new(200.0, 50.0, 80.0));
        let (boundary_min, boundary_max) = boundary_extents(&transform);

        // Test X axis (larger)
        let position = Position::new(110.0, 0.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                wrap_from_max_axis(position.x, boundary_min.x, boundary_max.x),
                position.y,
                position.z,
            ),
        );

        // Test Y axis (smaller)
        let position = Position::new(0.0, 30.0, 0.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                position.x,
                wrap_from_max_axis(position.y, boundary_min.y, boundary_max.y),
                position.z,
            ),
        );

        // Test Z axis (medium)
        let position = Position::new(0.0, 0.0, -45.0);
        let result = Boundary::calculate_teleport_position(position, &transform);
        assert_position_eq(
            result,
            Position::new(
                position.x,
                position.y,
                wrap_from_min_axis(position.z, boundary_min.z, boundary_max.z),
            ),
        );
    }
}
