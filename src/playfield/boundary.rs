use bevy::camera::visibility::RenderLayers;
use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCamera;

use super::boundary_face::BoundaryFace;
use super::constants::BOUNDARY_CELL_COUNT;
use super::constants::BOUNDARY_DEFAULT_VIEWPORT_SIZE;
use super::constants::BOUNDARY_GRID_ALPHA;
use super::constants::BOUNDARY_GRID_LINE_WIDTH;
use super::constants::BOUNDARY_LINE_WIDTH_MULTIPLIER;
use super::constants::BOUNDARY_NORMAL_EPSILON;
use super::constants::BOUNDARY_OUTER_ALPHA;
use super::constants::BOUNDARY_OUTER_LINE_WIDTH;
use super::constants::BOUNDARY_OVEREXTENSION_EPSILON;
use super::constants::BOUNDARY_SCALAR;
use super::constants::BOUNDARY_SNAP_EPSILON;
use super::constants::CORNER_COLOR_FRONT_BACK_XY;
use super::constants::CORNER_COLOR_LEFT_RIGHT_YZ;
use super::constants::CORNER_COLOR_TOP_BOTTOM_XZ;
use super::constants::DEADEROID_APPROACHING_COLOR;
use super::portals::Portal;
use super::portals::PortalGizmo;
use super::types::BoundaryGizmo;
use super::types::FlattenIntersections;
use super::types::GridGizmo;
use super::types::Intersection;
use super::types::MultiFaceGeometry;
use super::types::PortalGeometry;
use crate::camera::CameraMoveList;
use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::orientation::CameraOrientation;
use crate::splash::SplashText;
use crate::state::GameState;

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
            .add_systems(Update, apply_boundary_config)
            .add_systems(
                Update,
                draw_boundary.run_if(in_state(GameState::Splash).or(in_state(GameState::InGame))),
            )
            .add_systems(Update, fade_boundary_in)
            .add_observer(start_boundary_fade);
    }
}

fn apply_boundary_config(mut config_store: ResMut<GizmoConfigStore>, boundary: Res<Boundary>) {
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
#[allow(clippy::struct_field_names)] // "boundary_" prefix distinguishes from grid_line_width
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
        Self {
            cell_count:          BOUNDARY_CELL_COUNT,
            // Start with alpha 0 - will be faded in during splash screen
            grid_color:          Color::from(tailwind::BLUE_500).with_alpha(0.0),
            outer_color:         Color::from(tailwind::BLUE_500).with_alpha(0.0),
            grid_line_width:     BOUNDARY_GRID_LINE_WIDTH,
            boundary_line_width: BOUNDARY_OUTER_LINE_WIDTH,
            boundary_scalar:     BOUNDARY_SCALAR,
            transform:           Transform::from_scale(
                BOUNDARY_SCALAR * BOUNDARY_CELL_COUNT.as_vec3(),
            ),
        }
    }
}

/// Component that triggers a fade-in animation for the `Boundary` gizmo
/// Lerps the `Boundary` resource's color alphas from 0.0 to target values over time
#[derive(Component)]
struct BoundaryFadeIn(Timer);

impl Boundary {
    /// Analyzes portal geometry relative to boundary faces
    fn classify_portal_geometry(&self, portal: &Portal) -> PortalGeometry {
        let overextended_faces = self.get_overextended_faces_for(portal);

        if overextended_faces.is_empty() {
            PortalGeometry::SingleFace
        } else if overextended_faces.len() == 1 {
            PortalGeometry::MultiFace(MultiFaceGeometry::Edge {
                overextended: overextended_faces[0],
            })
        } else {
            PortalGeometry::MultiFace(MultiFaceGeometry::Corner {
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

        // Collect all faces from the geometry (primary from portal.face + overextended)
        let all_faces_in_corner = match multiface {
            MultiFaceGeometry::Edge { overextended } => vec![portal.face, *overextended],
            MultiFaceGeometry::Corner { overextended } => {
                let mut faces = vec![portal.face];
                faces.extend(overextended);
                faces
            },
        };

        let mut face_count = 0;

        // Calculate constrained intersections for each face
        for &face in &all_faces_in_corner {
            let face_points = face.get_face_points(&min, &max);
            let intersections = intersect_portal_with_rectangle(portal, &face_points).to_vec();

            // Only count faces with exactly 2 intersection points
            if intersections.len() == 2 {
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
        // Extract overextended faces from geometry (primary is always portal.face)
        let primary_face = portal.face;
        let overextended_faces = match geometry {
            MultiFaceGeometry::Edge { overextended } => vec![*overextended],
            MultiFaceGeometry::Corner { overextended } => overextended.clone(),
        };

        // Calculate boundary extents for constraint checking
        let half_size = self.transform.scale / 2.0;
        let min = self.transform.translation - half_size;
        let max = self.transform.translation + half_size;

        // Collect ALL faces that need arcs (primary + overextended)
        let mut all_faces_for_drawing = vec![primary_face];
        all_faces_for_drawing.extend(overextended_faces.iter());

        let mut face_arcs = Vec::new();

        // Calculate constrained intersections for each face
        for &face in &all_faces_for_drawing {
            let face_points = face.get_face_points(&min, &max);
            let intersections = intersect_portal_with_rectangle(portal, &face_points).to_vec();

            // Only draw arcs for faces with exactly 2 intersection points
            if intersections.len() == 2 {
                face_arcs.push((face, intersections));
            }
        }

        // Draw all arcs
        for (face, points) in face_arcs {
            let face_color = get_portal_color(is_deaderoid, geometry, face, color);

            match geometry {
                MultiFaceGeometry::Edge { .. } if face == primary_face => {
                    // Primary face (contains actual portal.position) at edge uses complex arc logic
                    // with TAU angle inversion
                    Self::draw_primary_face_arc(
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
                    // The single Edge overextended face
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
    fn draw_primary_face_arc(
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

        if (dist_to_min_x - min_dist).abs() < BOUNDARY_NORMAL_EPSILON {
            Dir3::NEG_X
        } else if (dist_to_max_x - min_dist).abs() < BOUNDARY_NORMAL_EPSILON {
            Dir3::X
        } else if (dist_to_min_y - min_dist).abs() < BOUNDARY_NORMAL_EPSILON {
            Dir3::NEG_Y
        } else if (dist_to_max_y - min_dist).abs() < BOUNDARY_NORMAL_EPSILON {
            Dir3::Y
        } else if (dist_to_min_z - min_dist).abs() < BOUNDARY_NORMAL_EPSILON {
            Dir3::NEG_Z
        } else if (dist_to_max_z - min_dist).abs() < BOUNDARY_NORMAL_EPSILON {
            Dir3::Z
        } else {
            // Fallback to Y
            Dir3::Y
        }
    }

    pub fn find_edge_point(&self, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let boundary_min = self.transform.translation - self.transform.scale / 2.0;
        let boundary_max = self.transform.translation + self.transform.scale / 2.0;

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
        .unwrap_or(BOUNDARY_DEFAULT_VIEWPORT_SIZE);
    let camera_distance = camera_transform
        .translation()
        .distance(boundary.transform.translation);
    let world_height_at_boundary = 2.0 * camera_distance * (perspective.fov / 2.0).tan();
    let world_units_per_pixel = world_height_at_boundary / viewport_size.y;

    // Gizmo lines are centered on edges
    // Empirically tuned multiplier to account for gizmo rendering
    let total_line_width = boundary.grid_line_width + boundary.boundary_line_width;
    let outer_scale = boundary.transform.scale
        + Vec3::splat(total_line_width * world_units_per_pixel * BOUNDARY_LINE_WIDTH_MULTIPLIER);

    outer_boundary_gizmo.primitive_3d(
        &Cuboid::from_size(outer_scale),
        Isometry3d::new(boundary.transform.translation, Quat::IDENTITY),
        boundary.outer_color,
    );
}

const fn get_portal_color(
    is_deaderoid: bool,
    geometry: &MultiFaceGeometry,
    face: BoundaryFace,
    default_color: Color,
) -> Color {
    if is_deaderoid {
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
        default_color
    }
}

fn intersect_portal_with_rectangle(
    portal: &Portal,
    rectangle_points: &[Vec3; 4],
) -> [Intersection; 4] {
    [
        intersect_circle_with_line_segment(portal, rectangle_points[0], rectangle_points[1]),
        intersect_circle_with_line_segment(portal, rectangle_points[1], rectangle_points[2]),
        intersect_circle_with_line_segment(portal, rectangle_points[2], rectangle_points[3]),
        intersect_circle_with_line_segment(portal, rectangle_points[3], rectangle_points[0]),
    ]
}

fn intersect_circle_with_line_segment(portal: &Portal, start: Vec3, end: Vec3) -> Intersection {
    let edge = end - start;
    let center_to_start = start - portal.position;

    let a = edge.dot(edge);
    let b = 2.0 * center_to_start.dot(edge);
    // FMA optimization (faster + more precise): dot(center_to_start) - radius²
    let c = portal
        .radius
        .mul_add(-portal.radius, center_to_start.dot(center_to_start));

    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return Intersection::NoneFound;
    }

    let t1 = (-b + discriminant.sqrt()) / (2.0 * a);
    let t2 = (-b - discriminant.sqrt()) / (2.0 * a);

    let t1_valid = (0.0..=1.0).contains(&t1);
    let t2_valid = (0.0..=1.0).contains(&t2) && (t1 - t2).abs() > 1e-6;

    match (t1_valid, t2_valid) {
        (false, false) => Intersection::NoneFound,
        (true, false) => Intersection::One(start + t1 * edge),
        (false, true) => Intersection::One(start + t2 * edge),
        (true, true) => Intersection::Two(start + t1 * edge, start + t2 * edge),
    }
}

/// Observer that triggers when `SplashText` is removed
/// Starts the boundary fade-in animation by spawning an entity with `BoundaryFadeIn`
fn start_boundary_fade(
    _trigger: On<Remove, SplashText>,
    mut commands: Commands,
    camera_query: Query<&CameraMoveList>,
) {
    // Get remaining time from camera animation
    let remaining_time_ms = camera_query
        .iter()
        .next()
        .map_or(0.0, CameraMoveList::remaining_time_ms);

    // Convert milliseconds to seconds for Timer
    let duration_secs = remaining_time_ms / 1000.0;

    // Spawn entity with fade timer
    commands.spawn(BoundaryFadeIn(Timer::from_seconds(
        duration_secs,
        TimerMode::Once,
    )));
}

/// System that fades in the boundary gizmo by lerping alpha values
fn fade_boundary_in(
    mut commands: Commands,
    time: Res<Time>,
    mut boundary: ResMut<Boundary>,
    mut fade_query: Query<(Entity, &mut BoundaryFadeIn)>,
) {
    for (entity, mut fade) in &mut fade_query {
        fade.0.tick(time.delta());

        // Calculate interpolation factor (0.0 to 1.0)
        let t = fade.0.fraction();

        // Lerp alpha from 0.0 to target values
        let grid_alpha = BOUNDARY_GRID_ALPHA * t;
        let outer_alpha = BOUNDARY_OUTER_ALPHA * t;

        // Update boundary colors
        boundary.grid_color = Color::from(tailwind::BLUE_500).with_alpha(grid_alpha);
        boundary.outer_color = Color::from(tailwind::BLUE_500).with_alpha(outer_alpha);

        // Log progress occasionally
        if fade.0.elapsed_secs() % 0.5 < 0.016 {
            debug!(
                "🎨 Boundary fade progress: {:.1}% (grid α={:.3}, outer α={:.3})",
                t * 100.0,
                grid_alpha,
                outer_alpha
            );
        }

        // Remove component when fade is complete
        if fade.0.is_finished() {
            debug!("✅ Boundary fade complete!");
            commands.entity(entity).despawn();
        }
    }
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
