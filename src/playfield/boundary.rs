use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::camera::RenderLayer;
use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;
use crate::orientation::CameraOrientation;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::portals::Portal;
use crate::playfield::portals::PortalGizmo;
use crate::state::PlayingGame;

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .init_gizmo_group::<BoundaryGridGizmo>()
            .init_gizmo_group::<OuterBoundaryGizmo>()
            .add_plugins(
                ResourceInspectorPlugin::<Boundary>::default()
                    .run_if(toggle_active(false, GlobalAction::BoundaryInspector)),
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
        let cell_count = UVec3::new(2, 1, 1);
        let scalar = 110.;

        Self {
            cell_count,
            grid_color: Color::from(tailwind::BLUE_300).with_alpha(0.25),
            outer_color: Color::from(tailwind::BLUE_500),
            line_width: 3.,
            outer_line_width: 3.,
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
            teleport_position.x = boundary_min.x;
        } else if position.x <= boundary_min.x {
            teleport_position.x = boundary_max.x;
        }

        if position.y >= boundary_max.y {
            teleport_position.y = boundary_min.y;
        } else if position.y <= boundary_min.y {
            teleport_position.y = boundary_max.y;
        }

        if position.z >= boundary_max.z {
            teleport_position.z = boundary_min.z;
        } else if position.z <= boundary_min.z {
            teleport_position.z = boundary_max.z;
        }

        teleport_position
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

        // todo #handle3d - with all likelihood this doesn't exactly make sense
        // when there's a corner so you may need a match to output both sets of points
        // for the extensions and only output the draw_portal_arc once..
        for (face, points) in over_extended_intersection_points {
            if points.len() >= 2 {
                let rotated_position =
                    self.rotate_portal_center_to_target_face(portal.position, portal.normal, face);

                // keep this around if you need to debug 3d later on
                // gizmos.sphere(portal.position, Quat::IDENTITY,1.,
                // Color::from(tailwind::PURPLE_500)).resolution(resolution);
                // gizmos.sphere(rotated_position, Quat::IDENTITY,1.,
                // Color::from(tailwind::PURPLE_500)).resolution(resolution);
                //
                // let rotation_point = self.find_closest_point_on_edge(portal.position,
                // portal.normal.as_vec3(), face.get_normal());
                // let rotation_axis =
                // portal.normal.as_vec3().cross(face.get_normal()).normalize();
                // gizmos.line(rotation_point, rotation_point + rotation_axis * 30.0,
                // Color::from(tailwind::YELLOW_500));
                // gizmos.line(rotation_point, portal.position, Color::from(tailwind::RED_500));
                // gizmos.line(rotation_point, rotated_position,
                // Color::from(tailwind::GREEN_500));

                gizmos
                    .short_arc_3d_between(
                        rotated_position,
                        points[0],
                        points[1],
                        color, // Color::from(tailwind::GREEN_800),
                    )
                    .resolution(resolution);
                self.draw_primary_arc(gizmos, portal, color, resolution, points[0], points[1]);
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

        // Return the rotated position in world space
        rotation_point + rotated_pos
    }

    fn find_closest_point_on_edge(&self, position: Vec3, normal1: Vec3, normal2: Vec3) -> Vec3 {
        // Calculate the direction of the intersection line (edge)
        let edge_direction = normal1.cross(normal2).normalize();

        // Compute the half extents and center of the cuboid
        let half_extents = self.transform.scale / 2.0;
        let center = self.transform.translation;

        // Determine the correct anchor point on the edge formed by the two normals
        let mut anchor_point = center;

        // Adjust the anchor point based on which planes are intersecting
        if normal1.x != 0.0 {
            anchor_point.x += normal1.x * half_extents.x;
        }
        if normal1.y != 0.0 {
            anchor_point.y += normal1.y * half_extents.y;
        }
        if normal1.z != 0.0 {
            anchor_point.z += normal1.z * half_extents.z;
        }

        if normal2.x != 0.0 {
            anchor_point.x += normal2.x * half_extents.x;
        }
        if normal2.y != 0.0 {
            anchor_point.y += normal2.y * half_extents.y;
        }
        if normal2.z != 0.0 {
            anchor_point.z += normal2.z * half_extents.z;
        }

        // Calculate vector from anchor point to position
        let to_position = position - anchor_point;

        // Project this onto the edge direction
        let projection_length = to_position.dot(edge_direction);

        anchor_point + projection_length * edge_direction
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

        // Check all faces regardless of the portal's normal
        if portal.position.x - radius < min.x {
            overextended_faces.push(BoundaryFace::Left);
        }
        if portal.position.x + radius > max.x {
            overextended_faces.push(BoundaryFace::Right);
        }
        if portal.position.y - radius < min.y {
            overextended_faces.push(BoundaryFace::Bottom);
        }
        if portal.position.y + radius > max.y {
            overextended_faces.push(BoundaryFace::Top);
        }
        if portal.position.z - radius < min.z {
            overextended_faces.push(BoundaryFace::Back);
        }
        if portal.position.z + radius > max.z {
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
    pub fn get_normal_for_position(&self, position: Vec3) -> Dir3 {
        let half_size = self.transform.scale / 2.0;
        let boundary_min = self.transform.translation - half_size;
        let boundary_max = self.transform.translation + half_size;

        let epsilon = 0.001; // Small value to account for floating-point imprecision

        if (position.x - boundary_min.x).abs() < epsilon {
            Dir3::NEG_X
        } else if (position.x - boundary_max.x).abs() < epsilon {
            Dir3::X
        } else if (position.y - boundary_min.y).abs() < epsilon {
            Dir3::NEG_Y
        } else if (position.y - boundary_max.y).abs() < epsilon {
            Dir3::Y
        } else if (position.z - boundary_min.z).abs() < epsilon {
            Dir3::NEG_Z
        } else if (position.z - boundary_max.z).abs() < epsilon {
            Dir3::Z
        } else {
            // Default to Y if not on a boundary face
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

    // Draw outer boundary cuboid with thicker line
    outer_boundary_gizmo.primitive_3d(
        &Cuboid::from_size(boundary.transform.scale),
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
