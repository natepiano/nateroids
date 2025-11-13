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

const MIN_POINTS_FOR_ARC: usize = 2;

// Deaderoid portal colors
const DEADEROID_APPROACHING_COLOR: Color = Color::srgb(1.0, 0.0, 0.0); // Red
const CORNER_COLOR_LEFT_RIGHT_YZ: Color = Color::srgb(1.0, 0.0, 0.0); // Red
const CORNER_COLOR_TOP_BOTTOM_XZ: Color = Color::srgb(0.0, 1.0, 0.0); // Green
const CORNER_COLOR_FRONT_BACK_XY: Color = Color::srgb(1.0, 1.0, 0.0); // Yellow

pub struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .init_gizmo_group::<GridGizmo>()
            .init_gizmo_group::<BoundaryGizmo>()
            .add_plugins(
                ResourceInspectorPlugin::<Boundary>::default()
                    .run_if(toggle_active(false, GameAction::BoundaryInspector)),
            )
            .add_systems(Update, update_gizmos_config)
            .add_systems(Update, draw_boundary.run_if(in_state(PlayingGame)));
    }
}

/// Describes the geometric configuration of a portal relative to boundary faces
#[derive(Debug, Clone, PartialEq, Eq)]
enum PortalGeometry {
    /// Portal completely within a single boundary face
    SingleFace,
    /// Portal extends across multiple faces (edge or corner)
    MultiFace(MultiFaceGeometry),
}

/// Describes portals that span multiple boundary faces
#[derive(Debug, Clone, PartialEq, Eq)]
enum MultiFaceGeometry {
    /// Portal extends across an edge between two faces
    Edge {
        primary:      BoundaryFace,
        overextended: BoundaryFace,
    },
    /// Portal extends into a corner (3+ faces)
    Corner {
        primary:      BoundaryFace,
        overextended: Vec<BoundaryFace>,
    },
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GridGizmo {}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct BoundaryGizmo {}

fn update_gizmos_config(mut config_store: ResMut<GizmoConfigStore>, boundary: Res<Boundary>) {
    let (config, _) = config_store.config_mut::<GridGizmo>();
    config.line.width = boundary.grid_line_width;
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());

    let (outer_config, _) = config_store.config_mut::<BoundaryGizmo>();
    outer_config.line.width = boundary.boundary_line_width;
    outer_config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

/// defines
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct Boundary {
    pub cell_count:          UVec3,
    pub grid_color:          Color,
    pub outer_color:         Color,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub grid_line_width:     f32,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub boundary_line_width: f32,
    #[inspector(min = 50., max = 300., display = NumberDisplay::Slider)]
    pub boundary_scalar:     f32,
    pub transform:           Transform,
}

impl Default for Boundary {
    fn default() -> Self {
        let cell_count = UVec3::new(3, 2, 1);
        let boundary_scalar = 110.;

        Self {
            cell_count,
            grid_color: Color::from(tailwind::BLUE_500).with_alpha(0.25),
            outer_color: Color::from(tailwind::BLUE_500).with_alpha(1.0),
            grid_line_width: 1.5,
            boundary_line_width: 6.,
            boundary_scalar,
            transform: Transform::from_scale(boundary_scalar * cell_count.as_vec3()),
        }
    }
}

impl Boundary {
    /// Analyzes portal geometry relative to boundary faces
    fn classify_portal_geometry(&self, portal: &Portal) -> PortalGeometry {
        let overextended_faces = self.get_overextended_faces_for(portal);
        let primary = portal.face;

        if overextended_faces.is_empty() {
            PortalGeometry::SingleFace
        } else if overextended_faces.len() == 1 {
            PortalGeometry::MultiFace(MultiFaceGeometry::Edge {
                primary,
                overextended: overextended_faces[0],
            })
        } else {
            PortalGeometry::MultiFace(MultiFaceGeometry::Corner {
                primary,
                overextended: overextended_faces,
            })
        }
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

    /// Calculates how many faces a portal spans at a given position
    pub fn calculate_portal_face_count(&self, portal: &Portal) -> usize {
        let geometry = self.classify_portal_geometry(portal);

        match geometry {
            PortalGeometry::SingleFace => 1,
            PortalGeometry::MultiFace(multiface) => {
                self.count_faces_with_valid_arcs(portal, &multiface)
            },
        }
    }

    /// Counts how many faces have valid arc intersections for a multi-face portal
    fn count_faces_with_valid_arcs(&self, portal: &Portal, multiface: &MultiFaceGeometry) -> usize {
        // Calculate boundary extents for constraint checking
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;

        // Collect all faces from the geometry
        let all_faces_in_corner = match multiface {
            MultiFaceGeometry::Edge {
                primary,
                overextended,
            } => vec![*primary, *overextended],
            MultiFaceGeometry::Corner {
                primary,
                overextended,
            } => {
                let mut faces = vec![*primary];
                faces.extend(overextended);
                faces
            },
        };

        let mut face_count = 0;

        // Calculate constrained intersections for each face
        for &face in &all_faces_in_corner {
            let face_points = face.get_face_points(&min, &max);
            let raw_intersections = intersect_portal_with_rectangle(portal, &face_points);

            // Apply constraints: filter out points that extend beyond face boundaries
            let constrained_points = constrain_intersection_points(
                raw_intersections,
                face,
                &all_faces_in_corner,
                &min,
                &max,
            );

            if constrained_points.len() >= MIN_POINTS_FOR_ARC {
                face_count += 1;
            }
        }

        face_count
    }

    pub fn draw_portal(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        orientation: &CameraOrientation,
        is_deaderoid: bool,
    ) {
        let geometry = self.classify_portal_geometry(portal);
        self.render_portal_by_geometry(
            gizmos,
            portal,
            color,
            resolution,
            orientation,
            is_deaderoid,
            &geometry,
        );
    }

    fn render_portal_by_geometry(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        orientation: &CameraOrientation,
        is_deaderoid: bool,
        geometry: &PortalGeometry,
    ) {
        match geometry {
            PortalGeometry::SingleFace => {
                // Draw full circle
                let rotation = Quat::from_rotation_arc(
                    orientation.config.axis_profundus,
                    portal.normal().as_vec3(),
                );
                let isometry = Isometry3d::new(portal.position, rotation);
                gizmos
                    .circle(isometry, portal.radius, color)
                    .resolution(resolution);
            },
            PortalGeometry::MultiFace(multiface) => {
                self.draw_multiface_portal(
                    gizmos,
                    portal,
                    color,
                    resolution,
                    is_deaderoid,
                    multiface,
                );
            },
        }
    }

    fn draw_multiface_portal(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        is_deaderoid: bool,
        geometry: &MultiFaceGeometry,
    ) {
        // Extract primary face and overextended faces from geometry
        let (primary_face, overextended_faces) = match geometry {
            MultiFaceGeometry::Edge {
                primary,
                overextended,
            } => (*primary, vec![*overextended]),
            MultiFaceGeometry::Corner {
                primary,
                overextended,
            } => (*primary, overextended.clone()),
        };

        // Calculate boundary extents for constraint checking
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;

        // Collect ALL faces that need arcs (primary + overextended)
        let mut all_faces_in_corner = vec![primary_face];
        all_faces_in_corner.extend(overextended_faces.iter());

        let mut face_arcs = Vec::new();

        // Calculate constrained intersections for each face
        for &face in &all_faces_in_corner {
            let face_points = face.get_face_points(&min, &max);
            let raw_intersections = intersect_portal_with_rectangle(portal, &face_points);

            // Apply constraints: filter out points that extend beyond face boundaries
            // Pass ALL faces so each face can check against all others
            let constrained_points = constrain_intersection_points(
                raw_intersections,
                face,
                &all_faces_in_corner,
                &min,
                &max,
            );

            if constrained_points.len() >= MIN_POINTS_FOR_ARC {
                face_arcs.push((face, constrained_points));
            }
        }

        // Draw all arcs
        for (face, points) in face_arcs {
            // Apply face color-coding only for deaderoid portals
            let face_color = if is_deaderoid {
                match geometry {
                    MultiFaceGeometry::Corner { .. } => {
                        // Corner: use 3-color diagnostic scheme
                        match face {
                            BoundaryFace::Left | BoundaryFace::Right => CORNER_COLOR_LEFT_RIGHT_YZ,
                            BoundaryFace::Top | BoundaryFace::Bottom => CORNER_COLOR_TOP_BOTTOM_XZ,
                            BoundaryFace::Front | BoundaryFace::Back => CORNER_COLOR_FRONT_BACK_XY,
                        }
                    },
                    MultiFaceGeometry::Edge { .. } => DEADEROID_APPROACHING_COLOR,
                }
            } else {
                color // Non-deaderoid portals: always use the provided color
            };

            // Only use draw_arc_with_center_and_normal for edge primary faces, notorners
            match geometry {
                MultiFaceGeometry::Edge { .. } if face == primary_face => {
                    // Primary face at edge uses the complex arc logic with TAU - angle inversion
                    self.draw_arc_with_center_and_normal(
                        gizmos,
                        portal.position,
                        portal.radius,
                        portal.normal().as_vec3(),
                        face_color,
                        resolution,
                        points[0],
                        points[1],
                    );
                },
                MultiFaceGeometry::Edge { .. } => {
                    // Edge overextended faces
                    let center = self.rotate_portal_center_to_target_face(
                        portal.position,
                        portal.normal(),
                        face,
                    );
                    gizmos
                        .short_arc_3d_between(center, points[0], points[1], face_color)
                        .resolution(resolution);
                },
                MultiFaceGeometry::Corner { .. } => {
                    // For ALL corner faces (including primary)
                    gizmos
                        .short_arc_3d_between(portal.position, points[0], points[1], face_color)
                        .resolution(resolution);
                },
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

    // Helper function to draw an arc with explicit center, radius, and normal
    // Used for primary face arcs - inverts the angle for proper rendering
    fn draw_arc_with_center_and_normal(
        &self,
        gizmos: &mut Gizmos<PortalGizmo>,
        center: Vec3,
        radius: f32,
        normal: Vec3,
        color: Color,
        resolution: u32,
        from: Vec3,
        to: Vec3,
    ) {
        // Calculate vectors from center to intersection points
        let vec_from = (from - center).normalize();
        let vec_to = (to - center).normalize();

        // Calculate the angle and determine direction
        let mut angle = vec_from.angle_between(vec_to);
        let cross_product = vec_from.cross(vec_to);
        let is_clockwise = cross_product.dot(normal) < 0.0;

        // Invert the angle for arc_3d rendering logic
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
        overextended_faces.retain(|&face| face != portal.face);
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

    pub fn scale(&self) -> Vec3 { self.boundary_scalar * self.cell_count.as_vec3() }

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
    point: Vec3,
    start: f32,
    origin: Vec3,
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

/// draw the grid and then slightly outside the grid, draw the boundary around the whole grid
/// transform
fn draw_boundary(
    mut boundary: ResMut<Boundary>,
    mut grid_gizmo: Gizmos<GridGizmo>,
    mut outer_boundary_gizmo: Gizmos<BoundaryGizmo>,
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
            Vec3::splat(boundary.boundary_scalar),
            boundary.grid_color,
        )
        .outer_edges();

    // Calculate world-space offset based on camera projection
    let Ok((camera, projection, camera_transform)) = camera_query.single() else {
        return; // No camera yet, skip gizmo rendering this frame
    };
    let Projection::Perspective(perspective) = projection else {
        return; // Not perspective camera, skip
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
    let total_line_width = boundary.grid_line_width + boundary.boundary_line_width;
    let outer_scale =
        boundary.transform.scale + Vec3::splat(total_line_width * world_units_per_pixel * 0.1);

    outer_boundary_gizmo.primitive_3d(
        &Cuboid::from_size(outer_scale),
        Isometry3d::new(boundary.transform.translation, Quat::IDENTITY),
        boundary.outer_color,
    );
}

pub fn intersect_portal_with_rectangle(portal: &Portal, rectangle_points: &[Vec3; 4]) -> Vec<Vec3> {
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
    raw_intersections: Vec<Vec3>,
    current_face: BoundaryFace,
    all_faces_in_corner: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> Vec<Vec3> {
    raw_intersections
        .into_iter()
        .filter(|point| {
            point_within_boundary_for_face(*point, current_face, all_faces_in_corner, min, max)
        })
        .collect()
}

fn point_within_boundary_for_face(
    point: Vec3,
    current_face: BoundaryFace,
    all_faces_in_corner: &[BoundaryFace],
    min: &Vec3,
    max: &Vec3,
) -> bool {
    // Check that point doesn't extend beyond ANY of the other faces in the corner
    for &other_face in all_faces_in_corner {
        if other_face == current_face {
            continue; // Skip checking against ourselves
        }
        if faces_share_axis(current_face, other_face) {
            continue; // Same axis, no constraint needed (optimization)
        }

        // Check if point exceeds the boundary this other face represents
        // These are exact comparisons - no epsilon needed for geometric filtering
        match other_face {
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
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    /// Helper function to create a test boundary centered at origin with given size
    fn create_test_boundary(size: Vec3) -> Boundary {
        Boundary {
            transform: Transform {
                translation: Vec3::ZERO,
                scale: size,
                ..default()
            },
            ..default()
        }
    }

    #[test]
    fn test_no_teleport_when_inside_boundary() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Test positions well inside boundary
        let inside_positions = vec![
            Vec3::ZERO,
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(0.0, 20.0, 0.0),
            Vec3::new(0.0, 0.0, 30.0),
            Vec3::new(-10.0, -20.0, -30.0),
        ];

        for pos in inside_positions {
            let result = boundary.calculate_teleport_position(pos);
            assert_eq!(
                result, pos,
                "Position {pos} inside boundary should not be teleported"
            );
        }
    }

    #[test]
    fn test_teleport_right_face_to_left() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));
        // boundary_max.x = 50.0, boundary_min.x = -50.0

        // Entity exits right face (+X) at x=55.0 (5.0 past boundary)
        let position = Vec3::new(55.0, 0.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

        // Should wrap to left face at x=-45.0 (5.0 offset from left boundary)
        assert_eq!(result.x, -45.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_left_face_to_right() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits left face (-X) at x=-60.0 (10.0 past boundary)
        let position = Vec3::new(-60.0, 0.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

        // Should wrap to right face at x=40.0 (10.0 offset from right boundary)
        assert_eq!(result.x, 40.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_top_face_to_bottom() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits top face (+Y) at y=53.0 (3.0 past boundary)
        let position = Vec3::new(0.0, 53.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

        // Should wrap to bottom face at y=-47.0 (3.0 offset from bottom boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, -47.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_bottom_face_to_top() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits bottom face (-Y) at y=-58.0 (8.0 past boundary)
        let position = Vec3::new(0.0, -58.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

        // Should wrap to top face at y=42.0 (8.0 offset from top boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 42.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_front_face_to_back() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits front face (+Z) at z=52.0 (2.0 past boundary)
        let position = Vec3::new(0.0, 0.0, 52.0);
        let result = boundary.calculate_teleport_position(position);

        // Should wrap to back face at z=-48.0 (2.0 offset from back boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, -48.0);
    }

    #[test]
    fn test_teleport_back_face_to_front() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits back face (-Z) at z=-57.0 (7.0 past boundary)
        let position = Vec3::new(0.0, 0.0, -57.0);
        let result = boundary.calculate_teleport_position(position);

        // Should wrap to front face at z=43.0 (7.0 offset from front boundary)
        assert_eq!(result.x, 0.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 43.0);
    }

    #[test]
    fn test_teleport_preserves_offset_on_other_axes() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits right face with Y and Z offsets
        let position = Vec3::new(55.0, 20.0, -10.0);
        let result = boundary.calculate_teleport_position(position);

        // X should wrap, but Y and Z should remain unchanged
        assert_eq!(result.x, -45.0);
        assert_eq!(result.y, 20.0);
        assert_eq!(result.z, -10.0);
    }

    #[test]
    fn test_teleport_edge_wrapping() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits both right face (+X) and top face (+Y)
        let position = Vec3::new(53.0, 52.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

        // Both axes should wrap independently
        assert_eq!(result.x, -47.0); // Wrapped from right to left
        assert_eq!(result.y, -48.0); // Wrapped from top to bottom
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_corner_wrapping() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity exits all three faces (corner case)
        let position = Vec3::new(55.0, 58.0, 52.0);
        let result = boundary.calculate_teleport_position(position);

        // All three axes should wrap independently
        assert_eq!(result.x, -45.0);
        assert_eq!(result.y, -42.0);
        assert_eq!(result.z, -48.0);
    }

    #[test]
    fn test_teleport_large_offset() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Entity far past boundary (offset = 150.0, larger than boundary itself)
        let position = Vec3::new(200.0, 0.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

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
        let boundary = Boundary {
            transform: Transform {
                translation: Vec3::new(100.0, 50.0, -25.0),
                scale: Vec3::new(200.0, 100.0, 50.0),
                ..default()
            },
            ..default()
        };
        // boundary_min = (0, 0, -50), boundary_max = (200, 100, 0)

        // Test right face wrap
        let position = Vec3::new(205.0, 50.0, -25.0);
        let result = boundary.calculate_teleport_position(position);
        assert_eq!(result.x, 5.0); // Offset 5.0 from left boundary
        assert_eq!(result.y, 50.0);
        assert_eq!(result.z, -25.0);

        // Test top face wrap
        let position = Vec3::new(100.0, 103.0, -25.0);
        let result = boundary.calculate_teleport_position(position);
        assert_eq!(result.x, 100.0);
        assert_eq!(result.y, 3.0); // Offset 3.0 from bottom boundary
        assert_eq!(result.z, -25.0);
    }

    #[test]
    fn test_teleport_exactly_at_boundary() {
        let boundary = create_test_boundary(Vec3::new(100.0, 100.0, 100.0));

        // Position exactly at boundary (should not wrap, as condition is >= not >)
        let position = Vec3::new(50.0, 0.0, 0.0);
        let result = boundary.calculate_teleport_position(position);

        // At x=50.0 (boundary_max), should wrap to boundary_min
        assert_eq!(result.x, -50.0);
        assert_eq!(result.y, 0.0);
        assert_eq!(result.z, 0.0);
    }

    #[test]
    fn test_teleport_asymmetric_boundary() {
        // Test with different dimensions on each axis
        let boundary = create_test_boundary(Vec3::new(200.0, 50.0, 80.0));
        // boundary_min = (-100, -25, -40), boundary_max = (100, 25, 40)

        // Test X axis (larger)
        let position = Vec3::new(110.0, 0.0, 0.0);
        let result = boundary.calculate_teleport_position(position);
        assert_eq!(result.x, -90.0);

        // Test Y axis (smaller)
        let position = Vec3::new(0.0, 30.0, 0.0);
        let result = boundary.calculate_teleport_position(position);
        assert_eq!(result.y, -20.0);

        // Test Z axis (medium)
        let position = Vec3::new(0.0, 0.0, -45.0);
        let result = boundary.calculate_teleport_position(position);
        assert_eq!(result.z, 35.0);
    }
}
