use avian3d::prelude::*;
use bevy::app::App;
use bevy::app::Plugin;
use bevy::camera::visibility::RenderLayers;
use bevy::color::Color;
use bevy::color::palettes::tailwind;
use bevy::math::Dir3;
use bevy::math::Vec3;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use crate::actor::Aabb;
use crate::actor::Deaderoid;
use crate::actor::Teleporter;
use crate::camera::RenderLayer;
use crate::game_input::GameAction;
use crate::game_input::toggle_active;
use crate::orientation::CameraOrientation;
use crate::playfield::Boundary;
use crate::playfield::boundary_face::BoundaryFace;
use crate::state::IsPaused;
use crate::state::PlayingGame;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PortalGizmo>()
            .init_resource::<PortalConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<PortalConfig>::default()
                    .run_if(toggle_active(false, GameAction::PortalInspector)),
            )
            .add_systems(
                Update,
                (
                    update_portal_config.run_if(in_state(PlayingGame)),
                    init_portals.run_if(in_state(IsPaused::NotPaused)),
                    update_approaching_portals.run_if(in_state(IsPaused::NotPaused)),
                    update_emerging_portals.run_if(in_state(IsPaused::NotPaused)),
                    draw_approaching_portals.run_if(in_state(PlayingGame)),
                    draw_emerging_portals.run_if(in_state(PlayingGame)),
                )
                    .chain(),
            );
    }
}

#[derive(Debug, Default, Reflect, GizmoConfigGroup)]
pub struct PortalGizmo {}

fn update_portal_config(
    mut config_store: ResMut<GizmoConfigStore>,
    portal_config: Res<PortalConfig>,
) {
    let (config, _) = config_store.config_mut::<PortalGizmo>();
    config.line.width = portal_config.line_width;
    config.line.joints = GizmoLineJoint::Round(portal_config.line_joints);
    config.render_layers = RenderLayers::from_layers(RenderLayer::Game.layers());
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct PortalConfig {
    color_approaching:             Color,
    color_approaching_deaderoid:   Color,
    color_emerging:                Color,
    #[inspector(min = 0.0, max = std::f32::consts::PI, display = NumberDisplay::Slider)]
    pub direction_change_factor:   f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub distance_approach:         f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub distance_shrink:           f32,
    #[inspector(min = 1.0, max = 30.0, display = NumberDisplay::Slider)]
    pub fadeout_duration:          f32,
    #[inspector(min = 0, max = 40, display = NumberDisplay::Slider)]
    line_joints:                   u32,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    line_width:                    f32,
    #[inspector(min = 0.001, max = 1.0, display = NumberDisplay::Slider)]
    pub minimum_radius:            f32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider)]
    pub movement_smoothing_factor: f32,
    #[inspector(min = 1., max = 10., display = NumberDisplay::Slider)]
    pub portal_scalar:             f32,
    #[inspector(min = 1., max = 10., display = NumberDisplay::Slider)]
    pub portal_smallest:           f32,
    #[inspector(min = 3, max = 256, display = NumberDisplay::Slider)]
    resolution:                    u32,
}

impl Default for PortalConfig {
    fn default() -> Self {
        Self {
            color_approaching:           Color::from(tailwind::BLUE_600),
            color_approaching_deaderoid: Color::from(tailwind::RED_600),
            color_emerging:              Color::from(tailwind::YELLOW_800),
            direction_change_factor:     0.75,
            distance_approach:           0.5,
            distance_shrink:             0.25,
            fadeout_duration:            14.,
            line_joints:                 4,
            line_width:                  2.,
            minimum_radius:              0.1,
            movement_smoothing_factor:   0.08,
            portal_scalar:               2.,
            portal_smallest:             5.,
            resolution:                  128,
        }
    }
}

#[derive(Component, Default)]
pub struct ActorPortals {
    pub approaching: Option<Portal>,
    pub emerging:    Option<Portal>,
}

#[derive(Resource, Clone, Debug)]
pub struct Portal {
    pub actor_direction:            Vec3,
    pub actor_distance_to_wall:     f32,
    pub boundary_distance_approach: f32,
    pub boundary_distance_shrink:   f32,
    pub color:                      Color,
    pub face:                       BoundaryFace,
    pub face_count:                 usize,
    fade_out_started:               Option<f32>,
    pub position:                   Vec3,
    pub radius:                     f32,
}

impl Portal {
    /// Returns the normal direction for this portal's face
    pub fn normal(&self) -> Dir3 { self.face.to_dir3() }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            actor_direction:            Vec3::ZERO,
            actor_distance_to_wall:     0.,
            boundary_distance_approach: 0.,
            boundary_distance_shrink:   0.,
            color:                      Color::WHITE,
            face:                       BoundaryFace::Right,
            face_count:                 1,
            fade_out_started:           None,
            position:                   Vec3::ZERO,
            radius:                     0.,
        }
    }
}

fn init_portals(
    mut q_actor: Query<(
        &Aabb,
        &Transform,
        &LinearVelocity,
        &Teleporter,
        &mut ActorPortals,
        Option<&Deaderoid>,
    )>,
    boundary: Res<Boundary>,
    portal_config: Res<PortalConfig>,
    time: Res<Time>,
) {
    // todo #handle3d
    let boundary_size = boundary
        .transform
        .scale
        .x
        .min(boundary.transform.scale.y)
        .min(boundary.transform.scale.z);
    let boundary_distance_approach = boundary_size * portal_config.distance_approach;
    let boundary_distance_shrink = boundary_size * portal_config.distance_shrink;

    for (aabb, transform, velocity, teleporter, mut visual, deaderoid) in q_actor.iter_mut() {
        let radius =
            aabb.max_dimension().max(portal_config.portal_smallest) * portal_config.portal_scalar;

        let portal_position = transform.translation;
        let actor_direction = velocity.normalize_or_zero();

        let color = if deaderoid.is_some() {
            portal_config.color_approaching_deaderoid
        } else {
            portal_config.color_approaching
        };

        let portal = Portal {
            actor_direction,
            position: portal_position,
            boundary_distance_approach,
            boundary_distance_shrink,
            color,
            radius,
            ..default()
        };

        handle_approaching_visual(
            &boundary,
            portal.clone(),
            &portal_config,
            &time,
            &mut visual,
        );
        handle_emerging_visual(
            portal.clone(),
            &portal_config,
            teleporter,
            &time,
            &mut visual,
            &boundary,
        );
    }
}

/// Checks if a position is way beyond the boundary (physics burst).
/// Prevents drawing portals when actors burst past boundary due to high physics stress.
fn is_physics_burst(position: Vec3, boundary: &Boundary) -> bool {
    let boundary_half_size = boundary.transform.scale / 2.0;
    let max_distance_from_center = position.distance(boundary.transform.translation);
    let boundary_diagonal = boundary_half_size.length();
    max_distance_from_center > boundary_diagonal * 2.0
}

/// Snaps position to boundary and calculates the correct face for the snapped position.
/// Recalculates face because snapping can move position to a different face (especially at
/// corners).
fn snap_and_get_face(
    position: Vec3,
    initial_normal: Dir3,
    boundary: &Boundary,
) -> (Vec3, Option<BoundaryFace>) {
    let snapped_position = boundary.snap_position_to_boundary_face(position, initial_normal);
    let final_normal = boundary.get_normal_for_position(snapped_position);
    let face = BoundaryFace::from_normal(final_normal);
    (snapped_position, face)
}

fn handle_emerging_visual(
    portal: Portal,
    portal_config: &Res<PortalConfig>,
    teleporter: &Teleporter,
    time: &Res<Time>,
    visual: &mut Mut<ActorPortals>,
    boundary: &Res<Boundary>,
) {
    if teleporter.just_teleported {
        if let Some(normal) = teleporter.last_teleported_normal {
            // establish the existence of an emerging
            if let Some(face) = BoundaryFace::from_normal(normal)
                && let Some(teleported_position) = teleporter.last_teleported_position
            {
                // If actor burst way past boundary, don't create emerging portal
                if is_physics_burst(teleported_position, boundary) {
                    visual.emerging = None;
                    return;
                }

                // Snap to boundary face and recalculate face to prevent corner glitches
                let (snapped_position, final_face) =
                    snap_and_get_face(teleported_position, normal, boundary);

                visual.emerging = Some(Portal {
                    actor_distance_to_wall: 0.0,
                    face: final_face.unwrap_or(face),
                    position: snapped_position,
                    fade_out_started: Some(time.elapsed_secs()),
                    ..portal
                });
            }
        }
    }
    // once the radius gets small enough we can eliminate it
    else if let Some(ref mut emerging) = visual.emerging {
        // Check if the radius has shrunk to a small value (near zero)
        if emerging.radius <= portal_config.minimum_radius {
            visual.emerging = None; // Remove the visual
        }
    }
}

fn handle_approaching_visual(
    boundary: &Res<Boundary>,
    portal: Portal,
    portal_config: &Res<PortalConfig>,
    time: &Res<Time>,
    visual: &mut Mut<ActorPortals>,
) {
    if let Some(collision_point) = boundary.find_edge_point(portal.position, portal.actor_direction)
    {
        let actor_distance_to_wall = portal.position.distance(collision_point);

        if actor_distance_to_wall <= portal.boundary_distance_approach {
            let normal = boundary.get_normal_for_position(collision_point);

            // Create temporary portal at collision point to calculate face count BEFORE smoothing
            let face = BoundaryFace::from_normal(normal).unwrap_or(BoundaryFace::Right);
            let temp_portal = Portal {
                position: collision_point,
                face,
                radius: portal.radius,
                ..portal.clone()
            };
            let current_face_count = boundary.calculate_portal_face_count(&temp_portal);

            // Get previous face count
            let previous_face_count = visual
                .approaching
                .as_ref()
                .map(|p| p.face_count)
                .unwrap_or(1);

            // Disable smoothing on any topology change to prevent off-plane artifacts
            let smoothed_position = if current_face_count != previous_face_count {
                collision_point
            } else {
                smooth_circle_position(visual, collision_point, normal, portal_config)
            };

            // Snap to boundary face and recalculate face to prevent corner glitches
            let (snapped_position, face) = snap_and_get_face(smoothed_position, normal, boundary);

            if let Some(face) = face {
                visual.approaching = Some(Portal {
                    actor_distance_to_wall,
                    face,
                    face_count: current_face_count,
                    position: snapped_position,
                    ..portal
                });
                return;
            }
        }
    }

    // If we reach this point, actor is not approaching
    // Check if actor burst way beyond boundary (physics stress) or teleported normally
    if let Some(approaching) = &mut visual.approaching {
        if is_physics_burst(portal.position, boundary) {
            // Actor burst way past boundary - immediately remove
            visual.approaching = None;
        } else if approaching.fade_out_started.is_none() {
            // Normal teleport - start fadeout
            approaching.fade_out_started = Some(time.elapsed_secs());
        }
    }
}

// updated to handle two situations
// 1. if you switch direction on approach, the circle used to jump away fast
// implemented a smoothing factor to alleviate this
//
// 2. with the smoothing factor, it can cause the circle to draw on the wrong wall if
// you are close to two walls and switch from the one to the other
// so we need to switch to the new collision point in that case
//
// extracted for readability/complexity
fn smooth_circle_position(
    visual: &mut Mut<ActorPortals>,
    collision_point: Vec3,
    current_boundary_wall_normal: Dir3,
    portal_config: &Res<PortalConfig>,
) -> Vec3 {
    if let Some(approaching) = &visual.approaching {
        // Adjust this value to control smoothing (0.0 to 1.0)
        let smoothing_factor = portal_config.movement_smoothing_factor;

        // Only smooth the position if the normal hasn't changed significantly
        // circle_direction_change_factor = threshold for considering normals "similar"
        // approaching carries the last normal, current carries this frame's normal
        if approaching
            .normal()
            .dot(current_boundary_wall_normal.as_vec3())
            > portal_config.direction_change_factor
        {
            approaching.position.lerp(collision_point, smoothing_factor)
        } else {
            // If normal changed significantly, jump to new position
            collision_point
        }
    } else {
        collision_point
    }
}

fn update_approaching_portals(
    time: Res<Time>,
    config: Res<PortalConfig>,
    mut q_portals: Query<&mut ActorPortals>,
) {
    for mut portal in q_portals.iter_mut() {
        if let Some(ref mut approaching) = portal.approaching {
            let radius = get_approaching_radius(approaching);

            // handle fadeout and get rid of it if we're past duration
            // otherwise proceed
            if let Some(fade_out_start) = approaching.fade_out_started {
                // Calculate the elapsed time since fade-out started
                let elapsed_time = time.elapsed_secs() - fade_out_start;

                // Fade out over n seconds
                let fade_out_duration = config.fadeout_duration;
                if elapsed_time >= fade_out_duration || approaching.radius < config.minimum_radius {
                    // Remove visual after fade-out is complete
                    portal.approaching = None;
                    continue;
                }

                // Calculate the current reduction based on elapsed time
                let fade_factor = (1.0 - (elapsed_time / fade_out_duration)).clamp(0.0, 1.0);
                approaching.radius *= fade_factor;
            } else {
                // Apply the normal proximity-based scaling
                approaching.radius = radius;
            }
        }
    }
}

fn draw_approaching_portals(
    boundary: Res<Boundary>,
    config: Res<PortalConfig>,
    orientation: Res<CameraOrientation>,
    q_portals: Query<(&ActorPortals, Option<&Deaderoid>)>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    for (portal, deaderoid) in q_portals.iter() {
        if let Some(ref approaching) = portal.approaching {
            // Compute color based on current deaderoid status, not stored color
            let portal_color = if deaderoid.is_some() {
                config.color_approaching_deaderoid
            } else {
                config.color_approaching
            };

            boundary.draw_portal(
                &mut gizmos,
                approaching,
                portal_color,
                config.resolution,
                &orientation,
                deaderoid.is_some(),
            );
        }
    }
}

// extracted for readability
fn get_approaching_radius(approaching: &mut Portal) -> f32 {
    // 0.5 corresponds to making sure that the aabb's of an actor fits
    // once radius shrinks down - we make sure the aabb always fits
    // for now not parameterizing but maybe i'll care in the future
    let max_radius = approaching.radius;
    let min_radius = max_radius * 0.5;

    // Calculate the radius based on proximity to the boundary
    // as it's approaching we keep it at a fixed size until we enter the shrink zone
    if approaching.actor_distance_to_wall > approaching.boundary_distance_shrink {
        max_radius
    } else {
        let scale_factor = (approaching.actor_distance_to_wall
            / approaching.boundary_distance_shrink)
            .clamp(0.0, 1.0);
        min_radius + (max_radius - min_radius) * scale_factor
    }
}

fn update_emerging_portals(
    time: Res<Time>,
    config: Res<PortalConfig>,
    mut q_portals: Query<&mut ActorPortals>,
) {
    for mut portal in q_portals.iter_mut() {
        if let Some(ref mut emerging) = portal.emerging
            && let Some(emerging_start) = emerging.fade_out_started
        {
            // Calculate the elapsed time since the emerging process started
            let elapsed_time = time.elapsed_secs() - emerging_start;

            // Define the total duration for the emerging process
            let emerging_duration = config.fadeout_duration;

            // Calculate the progress based on elapsed time
            let progress = (elapsed_time / emerging_duration).clamp(0.0, 1.0);

            // Interpolate the radius from the full size down to zero
            let initial_radius = emerging.radius;
            let radius = initial_radius * (1.0 - progress); // Scale down as progress increases

            if radius > 0.0 {
                emerging.radius = radius;
            }

            // Remove visual after the emerging duration is complete
            if elapsed_time >= emerging_duration {
                portal.emerging = None;
            }
        }
    }
}

fn draw_emerging_portals(
    boundary: Res<Boundary>,
    config: Res<PortalConfig>,
    orientation: Res<CameraOrientation>,
    q_portals: Query<(&ActorPortals, Option<&Deaderoid>)>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    for (portal, deaderoid) in q_portals.iter() {
        if let Some(ref emerging) = portal.emerging {
            // Compute color based on current deaderoid status, not stored color
            let portal_color = if deaderoid.is_some() {
                config.color_approaching_deaderoid
            } else {
                config.color_emerging
            };

            boundary.draw_portal(
                &mut gizmos,
                emerging,
                portal_color,
                config.resolution,
                &orientation,
                deaderoid.is_some(),
            );
        }
    }
}
