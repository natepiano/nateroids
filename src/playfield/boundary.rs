use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::orientation::CameraOrientation;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::portals::Portal;
use crate::playfield::portals::PortalGizmo;
use crate::state::PlayingGame;

// Epsilon values for boundary position snapping and portal overextension detection
const BOUNDARY_SNAP_EPSILON: f32 = 0.01;
const BOUNDARY_OVEREXTENSION_EPSILON: f32 = BOUNDARY_SNAP_EPSILON * 2.0;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .init_gizmo_group::<BoundaryGridGizmo>()
            .init_gizmo_group::<OuterBoundaryGizmo>()
            .add_plugins(
                ResourceInspectorPlugin::<Boundary>::default()
                    .run_if(toggle_active(false, GameAction::BoundaryInspector)),
            )
            .add_systems(Update, update_gizmos_config)
            .add_systems(Update, draw_boundary.run_if(in_state(PlayingGame)));
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct BoundaryGridGizmo {}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct OuterBoundaryGizmo {}

fn update_gizmos_config(mut config_store: ResMut<GizmoConfigStore>, boundary: Res<Boundary>) {
    let (config, _) = config_store.config_mut::<BoundaryGridGizmo>();
    config.line.width = boundary.line_width;
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());

    let (outer_config, _) = config_store.config_mut::<OuterBoundaryGizmo>();
    outer_config.line.width = boundary.outer_line_width;
    outer_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

// circle_direction_change_factor:
// if we're within a certain radians of the wall we continue to draw on it but
// after that we consider that we're looking to be at a new wall boundary point
// adjust this if it makes sense to
//
// circle_smoothing_factor:
// keep it small so that if you change directions the circle doesn't fly
// away fast - looks terrible
//
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct Boundary {
    pub cell_count:       UVec3,
    pub grid_color:       Color,
    pub outer_color:      Color,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub line_width:       f32,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub outer_line_width: f32,
    #[inspector(min = 50., max = 300., display = NumberDisplay::Slider)]
    pub scalar:           f32,
    pub transform:        Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let cell_count = UVec3::new(3, 1, 1);
        let scalar = 110.;

        Self {
            cell_count,
            grid_color: Color::from(tailwind::BLUE_500).with_alpha(0.25),
            outer_color: Color::from(tailwind::BLUE_500).with_alpha(1.0),
            line_width: 1.5,
            outer_line_width: 6.,
            scalar,
            transform: Transform::from_scale(scalar * cell_count.as_vec3()),
        }
    }
}

impl Boundary {
    fn get_overextended_intersection_points(
        &self,
        portal: &Portal,
        overextended_faces: Vec<BoundaryFace>,
    ) -> Vec<(BoundaryFace, Vec<Vec3>)> {
        let mut intersections = Vec::new();
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;

        for face in overextended_faces {
            let face_points = face.get_face_points(&min, &max);
            let face_intersections = intersect_circle_with_rectangle(portal, &face_points);

            if !face_intersections.is_empty() {
                intersections.push((face, face_intersections));
            }
        }

        intersections
    }

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
    pub fn calculate_teleport_position(&self, position: Vec3) -> Vec3 {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        let mut teleport_position = position;

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

        teleport_position
    }

    /// Snaps a position to slightly inside the boundary face based on the normal.
    /// Offsets by epsilon to prevent false-positive overextension detection that would trigger
    /// corner wrapping arcs. Clamps perpendicular axes to handle corner/edge teleportation cases.
    pub fn snap_position_to_boundary_face(&self, position: Vec3, normal: Dir3) -> Vec3 {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        // Without this offset, portals on exact boundary would be flagged as overextended
        let epsilon = BOUNDARY_SNAP_EPSILON;

        let mut snapped_position = position;

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

        snapped_position
    }

    pub fn draw_portal(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        orientation: &CameraOrientation,
    ) {
        let overextended_faces = self.get_overextended_faces_for(portal);

        let over_extended_intersection_points =
            self.get_overextended_intersection_points(portal, overextended_faces);

        if over_extended_intersection_points.is_empty() {
            let rotation =
                Quat::from_rotation_arc(orientation.config.axis_profundus, portal.normal.as_vec3());
            let isometry = Isometry3d::new(portal.position, rotation);
            gizmos
                .circle(isometry, portal.radius, color)
                .resolution(resolution);

            return;
        }

        // Draw primary arc only once using first valid intersection points
        let mut primary_arc_drawn = false;

        for (face, points) in over_extended_intersection_points {
            if points.len() >= 2 {
                let rotated_position =
                    self.rotate_portal_center_to_target_face(portal.position, portal.normal, face);

                // Draw the wrapped arc on the adjacent face
                gizmos
                    .short_arc_3d_between(rotated_position, points[0], points[1], color)
                    .resolution(resolution);

                // Draw primary arc only once (use first valid intersection points)
                if !primary_arc_drawn {
                    self.draw_primary_arc(gizmos, portal, color, resolution, points[0], points[1]);
                    primary_arc_drawn = true;
                }
            }
        }
    }

    // when we rotate this to the target face we get a new center
    // for the arc that is drawn outside the boundary
    // wrapped to a point that provide a center that gives
    // the illusion of having the circle wrap around the edge
    fn rotate_portal_center_to_target_face(
        &self,
        position: Vec3,
        normal: Dir3,
        target_face: BoundaryFace,
    ) -> Vec3 {
        let current_normal = normal.as_vec3();
        let target_normal = target_face.get_normal();

        // The rotation axis is the cross product of the current and target normals
        let rotation_axis = current_normal.cross(target_normal).normalize();

        // Find the closest point on the rotation axis to the current position
        let rotation_point =
            self.find_closest_point_on_edge(position, current_normal, target_normal);

        // Create a rotation quaternion (90 degrees around the rotation axis)
        let rotation = Quat::from_axis_angle(rotation_axis, std::f32::consts::FRAC_PI_2);

        // Apply the rotation to the position relative to the rotation point
        let relative_pos = position - rotation_point;
        let rotated_pos = rotation * relative_pos;

        let mut result = rotation_point + rotated_pos;

        // Rotation math at corners can produce off-plane positions - force result onto target
        // face's plane
        let half_extents = self.transform.scale / 2.0;
        let center = self.transform.translation;

        match target_face {
            BoundaryFace::Right => result.x = center.x + half_extents.x,
            BoundaryFace::Left => result.x = center.x - half_extents.x,
            BoundaryFace::Top => result.y = center.y + half_extents.y,
            BoundaryFace::Bottom => result.y = center.y - half_extents.y,
            BoundaryFace::Front => result.z = center.z + half_extents.z,
            BoundaryFace::Back => result.z = center.z - half_extents.z,
        }

        result
    }

    fn find_closest_point_on_edge(&self, position: Vec3, normal1: Vec3, normal2: Vec3) -> Vec3 {
        let half = self.transform.scale / 2.0;
        let center = self.transform.translation;
        let min = center - half;
        let max = center + half;

        // For axis-aligned cuboid, the edge between two faces runs along one axis
        // with the other two coordinates fixed at the boundary planes.
        // For each axis: if either normal points along it, fix at that boundary;
        // otherwise the edge runs along that axis, so use position's coordinate.

        let x = if normal1.x != 0.0 {
            if normal1.x > 0.0 { max.x } else { min.x }
        } else if normal2.x != 0.0 {
            if normal2.x > 0.0 { max.x } else { min.x }
        } else {
            position.x // Edge runs along X axis
        };

        let y = if normal1.y != 0.0 {
            if normal1.y > 0.0 { max.y } else { min.y }
        } else if normal2.y != 0.0 {
            if normal2.y > 0.0 { max.y } else { min.y }
        } else {
            position.y // Edge runs along Y axis
        };

        let z = if normal1.z != 0.0 {
            if normal1.z > 0.0 { max.z } else { min.z }
        } else if normal2.z != 0.0 {
            if normal2.z > 0.0 { max.z } else { min.z }
        } else {
            position.z // Edge runs along Z axis
        };

        Vec3::new(x, y, z)
    }

    // arc_3d has these assumptions:
    // rotation: defines orientation of the arc, by default we assume the arc is
    // contained in a plane parallel to the XZ plane and the default starting
    // point is (position + Vec3::X)
    //
    // so we have to rotate the arc to match up with the actual place it should be
    // drawn
    fn draw_primary_arc(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        from: Vec3,
        to: Vec3,
    ) {
        let center = portal.position;
        let radius = portal.radius;
        let normal = portal.normal.as_vec3();

        // Calculate vectors from center to intersection points
        let vec_from = (from - center).normalize();
        let vec_to = (to - center).normalize();

        // Calculate the angle and determine direction
        let mut angle = vec_from.angle_between(vec_to);
        let cross_product = vec_from.cross(vec_to);
        let is_clockwise = cross_product.dot(normal) < 0.0;

        angle = std::f32::consts::TAU - angle;

        // Calculate the rotation to align the arc with the boundary face
        let face_rotation = Quat::from_rotation_arc(Vec3::Y, normal);

        // Determine the start vector based on clockwise/counterclockwise
        let start_vec = if is_clockwise { vec_from } else { vec_to };
        let start_rotation = Quat::from_rotation_arc(face_rotation * Vec3::X, start_vec);

        // Combine rotations
        let final_rotation = start_rotation * face_rotation;

        // Draw the arc
        gizmos
            .arc_3d(
                angle,
                radius,
                Isometry3d::new(center, final_rotation),
                color,
            )
            .resolution(resolution);

        // Debug visualization
        // gizmos.line(center, from, Color::from(tailwind::GREEN_500));
        // gizmos.line(center, to, Color::from(tailwind::BLUE_500));
    }

    fn get_overextended_faces_for(&self, portal: &Portal) -> Vec<BoundaryFace> {
        let mut overextended_faces = Vec::new();
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;
        let radius = portal.radius;

        // Portals are snapped 0.01 inside boundary - without this margin, they'd be incorrectly
        // detected as overextended, triggering broken corner wrapping math
        let epsilon = BOUNDARY_OVEREXTENSION_EPSILON;

        // Check all faces - only truly overextended if beyond boundary + epsilon
        if portal.position.x - radius < min.x - epsilon {
            overextended_faces.push(BoundaryFace::Left);
        }
        if portal.position.x + radius > max.x + epsilon {
            overextended_faces.push(BoundaryFace::Right);
        }
        if portal.position.y - radius < min.y - epsilon {
            overextended_faces.push(BoundaryFace::Bottom);
        }
        if portal.position.y + radius > max.y + epsilon {
            overextended_faces.push(BoundaryFace::Top);
        }
        if portal.position.z - radius < min.z - epsilon {
            overextended_faces.push(BoundaryFace::Back);
        }
        if portal.position.z + radius > max.z + epsilon {
            overextended_faces.push(BoundaryFace::Front);
        }

        // Remove the face the portal is on from the overextended faces
        let face_to_remove = match portal.normal {
            Dir3::NEG_X => BoundaryFace::Left,
            Dir3::X => BoundaryFace::Right,
            Dir3::NEG_Y => BoundaryFace::Bottom,
            Dir3::Y => BoundaryFace::Top,
            Dir3::NEG_Z => BoundaryFace::Back,
            Dir3::Z => BoundaryFace::Front,
            _ => return overextended_faces, // Handle any other case without removing a face
        };

        overextended_faces.retain(|&face| face != face_to_remove);
        overextended_faces
    }
    /// Returns the normal of the closest boundary face to a position.
    /// Uses distance-based matching because teleported positions have offsets (e.g., -54.97 instead
    /// of -55.0) that break simple epsilon matching.
    pub fn get_normal_for_position(&self, position: Vec3) -> Dir3 {
        let half_size = self.transform.scale / 2.0;
        let boundary_min = self.transform.translation - half_size;
        let boundary_max = self.transform.translation + half_size;

        // Calculate distance to all 6 faces and return normal of closest
        let dist_to_min_x = (position.x - boundary_min.x).abs();
        let dist_to_max_x = (position.x - boundary_max.x).abs();
        let dist_to_min_y = (position.y - boundary_min.y).abs();
        let dist_to_max_y = (position.y - boundary_max.y).abs();
        let dist_to_min_z = (position.z - boundary_min.z).abs();
        let dist_to_max_z = (position.z - boundary_max.z).abs();

        let min_dist = dist_to_min_x
            .min(dist_to_max_x)
            .min(dist_to_min_y)
            .min(dist_to_max_y)
            .min(dist_to_min_z)
            .min(dist_to_max_z);

        if (dist_to_min_x - min_dist).abs() < 0.001 {
            Dir3::NEG_X
        } else if (dist_to_max_x - min_dist).abs() < 0.001 {
            Dir3::X
        } else if (dist_to_min_y - min_dist).abs() < 0.001 {
            Dir3::NEG_Y
        } else if (dist_to_max_y - min_dist).abs() < 0.001 {
            Dir3::Y
        } else if (dist_to_min_z - min_dist).abs() < 0.001 {
            Dir3::NEG_Z
        } else if (dist_to_max_z - min_dist).abs() < 0.001 {
            Dir3::Z
        } else {
            // Fallback to Y
            Dir3::Y
        }
    }

    pub fn find_edge_point(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

        let mut t_min = f32::MAX;

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
                        && t < t_min
                        && is_in_bounds(point, start, origin, boundary_min, boundary_max)
                    {
                        t_min = t;
                    }
                };

                update_t_min(pos_bound);
                update_t_min(neg_bound);
            }
        }

        if t_min != f32::MAX {
            let edge_point = origin + direction * t_min;
            return Some(edge_point);
        }
        None
    }

    pub fn longest_diagonal(&self) -> f32 {
        let boundary_scale = self.scale();
        (boundary_scale.x.powi(2) + boundary_scale.y.powi(2) + boundary_scale.z.powi(2)).sqrt()
    }

    pub fn max_missile_distance(&self) -> f32 {
        let boundary_scale = self.scale();
        boundary_scale.x.max(boundary_scale.y).max(boundary_scale.z)
    }

    pub fn scale(&self) -> Vec3 { self.scalar * self.cell_count.as_vec3() }
}

fn is_in_bounds(
    point: Vec3,
    start: f32,
    origin: Vec3,
    boundary_min: Vec3,
    boundary_max: Vec3,
) -> bool {
    if start == origin.x {
        point.y >= boundary_min.y
            && point.y <= boundary_max.y
            && point.z >= boundary_min.z
            && point.z <= boundary_max.z
    } else if start == origin.y {
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

fn draw_boundary(
    mut boundary: ResMut<Boundary>,
    mut grid_gizmo: Gizmos<BoundaryGridGizmo>,
    mut outer_boundary_gizmo: Gizmos<OuterBoundaryGizmo>,
    camera_query: Query<(&Camera, &Projection, &GlobalTransform), With<PanOrbitCamera>>,
) {
    // updating the boundary resource transform from its configuration so it can be
    // dynamically changed with the inspector while the game is running
    // the boundary transform is used both for position but also
    // so the fixed camera can be positioned based on the boundary scale
    boundary.transform.scale = boundary.scale();

    grid_gizmo
        .grid_3d(
            Isometry3d::new(boundary.transform.translation, Quat::IDENTITY),
            boundary.cell_count,
            Vec3::splat(boundary.scalar),
            boundary.grid_color,
        )
        .outer_edges();

    // Calculate world-space offset based on camera projection
    let Ok((camera, projection, camera_transform)) = camera_query.single() else {
        panic!("No camera found");
    };
    let Projection::Perspective(perspective) = projection else {
        panic!("Expected perspective camera");
    };

    let viewport_size = camera
        .logical_viewport_size()
        .unwrap_or(Vec2::new(1920.0, 1080.0));
    let camera_distance = camera_transform
        .translation()
        .distance(boundary.transform.translation);
    let world_height_at_boundary = 2.0 * camera_distance * (perspective.fov / 2.0).tan();
    let world_units_per_pixel = world_height_at_boundary / viewport_size.y;

    // Gizmo lines are centered on edges
    // Empirically tuned multiplier to account for gizmo rendering
    let total_line_width = boundary.line_width + boundary.outer_line_width;
    let outer_scale =
        boundary.transform.scale + Vec3::splat(total_line_width * world_units_per_pixel * 0.1);

    outer_boundary_gizmo.primitive_3d(
        &Cuboid::from_size(outer_scale),
        Isometry3d::new(boundary.transform.translation, Quat::IDENTITY),
        boundary.outer_color,
    );
}

pub fn intersect_circle_with_rectangle(portal: &Portal, rectangle_points: &[Vec3; 4]) -> Vec<Vec3> {
    let mut intersections = Vec::new();

    for i in 0..4 {
        let start = rectangle_points[i];
        let end = rectangle_points[(i + 1) % 4];

        let edge_intersections = intersect_circle_with_line_segment(portal, start, end);
        intersections.extend(edge_intersections);
    }

    intersections
}

fn intersect_circle_with_line_segment(portal: &Portal, start: Vec3, end: Vec3) -> Vec<Vec3> {
    let edge = end - start;
    let center_to_start = start - portal.position;

    let a = edge.dot(edge);
    let b = 2.0 * center_to_start.dot(edge);
    let c = center_to_start.dot(center_to_start) - portal.radius * portal.radius;

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return vec![];
    }

    let mut intersections = Vec::new();
    let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

    if (0.0..=1.0).contains(&t1) {
        intersections.push(start + t1 * edge);
    }
    if (0.0..=1.0).contains(&t2) && (t1 - t2).abs() > 1e-6 {
        intersections.push(start + t2 * edge);
    }

    intersections
}

/// Filters intersection points to only include those within the face's boundary limits.
/// At corners, this prevents arcs from extending into adjacent faces.
///
/// Returns filtered vector containing only points within valid region. May be empty
/// if all points were outside boundaries (e.g., small portal near corner).
fn constrain_intersection_points(
    points: Vec<Vec3>,
    current_face: BoundaryFace,
    overextended_faces: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> Vec<Vec3> {
    points
        .into_iter()
        .filter(|point| {
            point_within_boundary_for_face(*point, current_face, overextended_faces, min, max)
        })
        .collect()
}

fn point_within_boundary_for_face(
    point: Vec3,
    face: BoundaryFace,
    overextended_faces: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> bool {
    // For each overextended face that's on a different axis, ensure point doesn't
    // extend beyond that boundary
    for overextended in overextended_faces {
        if faces_share_axis(face, *overextended) {
            continue; // Same axis, no constraint needed (optimization)
        }

        // Check if point exceeds the boundary this overextended face represents
        // These are exact comparisons - no epsilon needed for geometric filtering
        match overextended {
            BoundaryFace::Left => {
                if point.x < min.x {
                    return false;
                }
            },
            BoundaryFace::Right => {
                if point.x > max.x {
                    return false;
                }
            },
            BoundaryFace::Bottom => {
                if point.y < min.y {
                    return false;
                }
            },
            BoundaryFace::Top => {
                if point.y > max.y {
                    return false;
                }
            },
            BoundaryFace::Back => {
                if point.z < min.z {
                    return false;
                }
            },
            BoundaryFace::Front => {
                if point.z > max.z {
                    return false;
                }
            },
        }
    }

    true
}

/// Returns true if two faces are perpendicular to the same axis.
/// Used to optimize constraint checks by skipping geometrically impossible conditions.
///
/// Faces share an axis when they're perpendicular to the same coordinate axis:
/// - Left/Right: both perpendicular to X-axis (points have fixed X, varying Y/Z)
/// - Top/Bottom: both perpendicular to Y-axis (points have fixed Y, varying X/Z)
/// - Front/Back: both perpendicular to Z-axis (points have fixed Z, varying X/Y)
///
/// Example: When drawing on Left face (x = -55) with Right overextended (x = 55),
/// the constraint `point.x > 55` is impossible (point.x is fixed at -55).
/// Skipping this check is a performance optimization.
fn faces_share_axis(face1: BoundaryFace, face2: BoundaryFace) -> bool {
    use BoundaryFace::*;
    matches!(
        (face1, face2),
        // Same face (optimization for redundant self-checks)
        (Left, Left) | (Right, Right) |
        (Top, Top) | (Bottom, Bottom) |
        (Front, Front) | (Back, Back) |
        // Opposite faces on same axis
        (Left, Right) | (Right, Left) |
        (Top, Bottom) | (Bottom, Top) |
        (Front, Back) | (Back, Front)
    )
}

#[cfg(test)]
impl Boundary {
    /// Test helper: Get portal rendering data without drawing
    ///
    /// This is a self-contained implementation that doesn't depend on
    /// `get_overextended_intersection_points()` (which will be removed
    /// in the portal-upgraded.md refactor). It directly calculates
    /// intersections and applies constraints using the Phase 1 helpers.
    pub fn calculate_portal_render_data(&self, portal: &Portal) -> PortalRenderData {
        let overextended_faces = self.get_overextended_faces_for(portal);

        if overextended_faces.is_empty() {
            return PortalRenderData::SingleCircle {
                position: portal.position,
                normal:   portal.normal,
                radius:   portal.radius,
            };
        }

        // Calculate boundary extents for constraint checking
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;

        let primary_face = BoundaryFace::from_normal(portal.normal).unwrap();
        let mut arc_data = Vec::new();

        // Collect ALL faces that need arcs (primary + overextended)
        let mut all_faces = vec![primary_face];
        all_faces.extend(overextended_faces.iter());

        // Calculate intersections for each face
        for &face in &all_faces {
            let face_points = face.get_face_points(&min, &max);
            let mut face_intersections = intersect_circle_with_rectangle(portal, &face_points);

            // Apply constraints: filter out points that extend beyond face boundaries
            face_intersections = constrain_intersection_points(
                face_intersections,
                face,
                &overextended_faces,
                &min,
                &max,
            );

            if !face_intersections.is_empty() {
                arc_data.push((face, face_intersections));
            }
        }

        PortalRenderData::SplitArcs {
            primary_face,
            arc_data,
        }
    }
}

/// Test helper return type that captures complete portal rendering data.
///
/// **Design rationale**: This type intentionally includes ALL data that the real
/// `draw_portal()` function uses, even though current tests only validate subsets:
///
/// 1. **Debugging value**: When tests fail, developers can inspect the complete rendering state
///    (position, radius, normals, intersection points) to understand what went wrong, not just
///    which assertion failed.
///
/// 2. **Future test expansion**: Additional tests may validate position accuracy (e.g., verifying
///    `snap_position_to_boundary_face()` worked correctly) or radius constraints (e.g., portals
///    don't exceed max size).
///
/// 3. **Mirrors production code**: Type structure matches what `draw_portal()` actually uses,
///    making it a true "rendering data snapshot" rather than a minimal test assertion type.
///
/// **Current test usage**: Tests validate rendering strategy (single circle vs
/// split arcs) and face selection, using pattern matching with `..` to ignore
/// geometric parameters that are already known (input values).
#[cfg(test)]
#[derive(Debug, PartialEq)]
pub enum PortalRenderData {
    SingleCircle {
        position: Vec3,
        normal:   Dir3,
        radius:   f32,
    },
    SplitArcs {
        primary_face: BoundaryFace,
        arc_data:     Vec<(BoundaryFace, Vec<Vec3>)>,
    },
}

#[cfg(test)]
mod portal_render_tests {
    use super::*;
    use crate::playfield::portals::Portal;

    // Note: The following types are available via `use super::*;`:
    // - Boundary, BoundaryFace (from boundary.rs)
    // - Vec3, Dir3, UVec3, Transform, default() (from bevy prelude)
    // - constrain_intersection_points() and other helper functions (from Phase 1)

    fn create_test_boundary() -> Boundary {
        Boundary {
            cell_count: UVec3::new(1, 1, 1),
            scalar: 110.,
            transform: Transform::from_scale(Vec3::splat(110.)),
            ..default()
        }
        // Boundary extends from -55 to 55 on all axes
    }

    fn create_portal(position: Vec3, radius: f32, normal: Dir3) -> Portal {
        let mut portal = Portal::default();
        portal.position = position;
        portal.radius = radius;
        portal.normal = normal;
        portal
    }

    // ===== CATEGORY 1: TOO FAR (1 test) =====

    #[test]
    fn test_portal_too_far_from_boundary_no_overextension() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::ZERO, 10.0, Dir3::X);

        let render_data = boundary.calculate_portal_render_data(&portal);

        match render_data {
            PortalRenderData::SingleCircle { .. } => {
                // Expected - portal doesn't reach boundaries
            },
            _ => panic!("Expected single circle for portal far from boundaries"),
        }
    }

    // ===== CATEGORY 2: SINGLE FACE (6 tests) =====

    #[test]
    fn test_portal_approaching_single_face_right_wall() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(54.99, 0.0, 0.0), 5.0, Dir3::X);

        let render_data = boundary.calculate_portal_render_data(&portal);

        match render_data {
            PortalRenderData::SingleCircle { normal, .. } => {
                assert_eq!(normal, Dir3::X);
            },
            _ => panic!("Expected single circle on right face"),
        }
    }

    #[test]
    fn test_portal_approaching_single_face_left_wall() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-54.99, 0.0, 0.0), 5.0, Dir3::NEG_X);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SingleCircle { normal, .. } => {
                assert_eq!(normal, Dir3::NEG_X);
            },
            _ => panic!("Expected single circle on left face"),
        }
    }

    #[test]
    fn test_portal_approaching_single_face_top_wall() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, 54.99, 0.0), 5.0, Dir3::Y);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SingleCircle { normal, .. } => {
                assert_eq!(normal, Dir3::Y);
            },
            _ => panic!("Expected single circle on top face"),
        }
    }

    #[test]
    fn test_portal_approaching_single_face_bottom_wall() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, -54.99, 0.0), 5.0, Dir3::NEG_Y);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SingleCircle { normal, .. } => {
                assert_eq!(normal, Dir3::NEG_Y);
            },
            _ => panic!("Expected single circle on bottom face"),
        }
    }

    #[test]
    fn test_portal_approaching_single_face_front_wall() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, 0.0, 54.99), 5.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SingleCircle { normal, .. } => {
                assert_eq!(normal, Dir3::Z);
            },
            _ => panic!("Expected single circle on front face"),
        }
    }

    #[test]
    fn test_portal_approaching_single_face_back_wall() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, 0.0, -54.99), 5.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SingleCircle { normal, .. } => {
                assert_eq!(normal, Dir3::NEG_Z);
            },
            _ => panic!("Expected single circle on back face"),
        }
    }

    // ===== CATEGORY 3: EDGE CASES (12 tests) =====

    // X-axis Edges (4 tests)

    #[test]
    fn test_portal_at_top_back_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, 54.99, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2, "Expected 2 faces at edge");

                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Top));

                for (face, points) in &arc_data {
                    assert!(points.len() >= 2, "Face {face:?} needs >= 2 points");
                }
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_top_front_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, 54.99, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_bottom_back_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, -54.99, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_bottom_front_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(0.0, -54.99, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    // Y-axis Edges (4 tests)

    #[test]
    fn test_portal_at_left_back_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-54.99, 0.0, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Left));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_left_front_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-54.99, 0.0, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Left));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_right_back_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(54.99, 0.0, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Right));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_right_front_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(54.99, 0.0, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Right));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    // Z-axis Edges (4 tests)

    #[test]
    fn test_portal_at_left_top_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-54.99, 54.99, 0.0), 15.0, Dir3::NEG_X);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Left));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_left_bottom_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-54.99, -54.99, 0.0), 15.0, Dir3::NEG_X);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Left));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_right_top_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(54.99, 54.99, 0.0), 15.0, Dir3::X);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Right));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    #[test]
    fn test_portal_at_right_bottom_edge() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(54.99, -54.99, 0.0), 15.0, Dir3::X);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 2);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Right));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at edge"),
        }
    }

    // ===== CATEGORY 4: CORNER CASES (8 tests) - EXPECTED TO FAIL =====

    #[test]
    fn test_portal_at_left_bottom_back_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-50.0, -50.0, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs {
                primary_face,
                arc_data,
            } => {
                assert_eq!(primary_face, BoundaryFace::Back);
                assert_eq!(arc_data.len(), 3, "Expected 3 faces at corner");

                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Left));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_left_bottom_front_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-50.0, -50.0, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Left));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_left_top_back_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-50.0, 50.0, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Left));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_left_top_front_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(-50.0, 50.0, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Left));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_right_bottom_back_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(50.0, -50.0, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Right));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_right_bottom_front_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(50.0, -50.0, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Right));
                assert!(faces.contains(&BoundaryFace::Bottom));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_right_top_back_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(50.0, 50.0, -54.99), 15.0, Dir3::NEG_Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Back));
                assert!(faces.contains(&BoundaryFace::Right));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }

    #[test]
    fn test_portal_at_right_top_front_corner() {
        let boundary = create_test_boundary();
        let portal = create_portal(Vec3::new(50.0, 50.0, 54.99), 15.0, Dir3::Z);

        match boundary.calculate_portal_render_data(&portal) {
            PortalRenderData::SplitArcs { arc_data, .. } => {
                assert_eq!(arc_data.len(), 3);
                let faces: Vec<_> = arc_data.iter().map(|(f, _)| *f).collect();
                assert!(faces.contains(&BoundaryFace::Front));
                assert!(faces.contains(&BoundaryFace::Right));
                assert!(faces.contains(&BoundaryFace::Top));
            },
            _ => panic!("Expected split arcs at corner"),
        }
    }
}
