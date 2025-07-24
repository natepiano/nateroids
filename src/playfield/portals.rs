use crate::{
    actor::{
        Aabb,
        Teleporter,
    },
    global_input::{
        GlobalAction,
        toggle_active,
    },
    orientation::CameraOrientation,
    playfield::{
        Boundary,
        boundary_face::BoundaryFace,
    },
    state::PlayingGame,
};
use bevy::{
    app::{
        App,
        Plugin,
    },
    color::{
        Color,
        palettes::tailwind,
    },
    math::{
        Dir3,
        Vec3,
    },
    prelude::*,
};
use bevy_inspector_egui::{
    inspector_options::std_options::NumberDisplay,
    prelude::*,
    quick::ResourceInspectorPlugin,
};
use bevy_rapier3d::dynamics::Velocity;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<PortalGizmo>()
            .init_resource::<PortalConfig>()
            .register_type::<PortalConfig>()
            .add_plugins(
                ResourceInspectorPlugin::<PortalConfig>::default()
                    .run_if(toggle_active(false, GlobalAction::PortalInspector)),
            )
            .add_systems(
                Update,
                (
                    update_portal_config,
                    init_portals,
                    draw_approaching_portals,
                    draw_emerging_portals,
                )
                    .run_if(in_state(PlayingGame)),
            );
    }
}

#[derive(Debug, Default, Reflect, GizmoConfigGroup)]
pub struct PortalGizmo {}

fn update_portal_config(mut config_store: ResMut<GizmoConfigStore>, portal_config: Res<PortalConfig>) {
    let (config, _) = config_store.config_mut::<PortalGizmo>();
    config.line.width = portal_config.line_width;
    config.line.joints = GizmoLineJoint::Round(portal_config.line_joints);
}

#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
struct PortalConfig {
    color_approaching:             Color,
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
            color_approaching:         Color::from(tailwind::BLUE_600),
            color_emerging:            Color::from(tailwind::YELLOW_800),
            direction_change_factor:   0.75,
            distance_approach:         0.5,
            distance_shrink:           0.25,
            fadeout_duration:          14.,
            line_joints:               4,
            line_width:                2.,
            minimum_radius:            0.1,
            movement_smoothing_factor: 0.08,
            portal_scalar:             2.,
            portal_smallest:           5.,
            resolution:                128,
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
    pub face:                       BoundaryFace,
    fade_out_started:               Option<f32>,
    pub normal:                     Dir3,
    pub position:                   Vec3,
    pub radius:                     f32,
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            actor_direction:            Vec3::ZERO,
            actor_distance_to_wall:     0.,
            boundary_distance_approach: 0.,
            boundary_distance_shrink:   0.,
            face:                       BoundaryFace::Right,
            fade_out_started:           None,
            normal:                     Dir3::X,
            position:                   Vec3::ZERO,
            radius:                     0.,
        }
    }
}

fn init_portals(
    mut q_actor: Query<(&Aabb, &Transform, &Velocity, &Teleporter, &mut ActorPortals)>,
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

    for (aabb, transform, velocity, teleporter, mut visual) in q_actor.iter_mut() {
        let radius = aabb.max_dimension().max(portal_config.portal_smallest) * portal_config.portal_scalar;

        let portal_position = transform.translation;
        let actor_direction = velocity.linvel.normalize_or_zero();

        let portal = Portal {
            actor_direction,
            position: portal_position,
            boundary_distance_approach,
            boundary_distance_shrink,
            radius,
            ..default()
        };

        handle_approaching_visual(&boundary, portal.clone(), &portal_config, &time, &mut visual);
        handle_emerging_visual(portal.clone(), &portal_config, teleporter, &time, &mut visual);
    }
}

fn handle_emerging_visual(
    portal: Portal,
    portal_config: &Res<PortalConfig>,
    teleporter: &Teleporter,
    time: &Res<Time>,
    visual: &mut Mut<ActorPortals>,
) {
    if teleporter.just_teleported {
        if let Some(normal) = teleporter.last_teleported_normal {
            // establish the existence of an emerging
            if let Some(face) = BoundaryFace::from_normal(normal) {
                visual.emerging = Some(Portal {
                    actor_distance_to_wall: 0.0,
                    face,
                    normal,
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
    if let Some(collision_point) = boundary.find_edge_point(portal.position, portal.actor_direction) {
        let actor_distance_to_wall = portal.position.distance(collision_point);

        if actor_distance_to_wall <= portal.boundary_distance_approach {
            let normal = boundary.get_normal_for_position(collision_point);
            let position = smooth_circle_position(visual, collision_point, normal, portal_config);

            if let Some(face) = BoundaryFace::from_normal(normal) {
                visual.approaching = Some(Portal {
                    actor_distance_to_wall,
                    face,
                    normal,
                    position,
                    ..portal
                });
                return;
            }
        }
    }

    // If we reach this point, we've teleported
    if let Some(approaching) = &mut visual.approaching {
        if approaching.fade_out_started.is_none() {
            // Start fade-out
            approaching.fade_out_started = Some(time.elapsed_secs());
        }
    }
}

// updated to handle two situations
// 1. if you switch direction on approach, the circle used to jump away fast
// implemented a smoothing factor to alleviate this
//
// 2. with the smoothing factor, it can cause the circle to draw on the wrong
//    wall if
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
        if approaching.normal.dot(current_boundary_wall_normal.as_vec3())
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

fn draw_approaching_portals(
    time: Res<Time>,
    boundary: Res<Boundary>,
    config: Res<PortalConfig>,
    orientation: Res<CameraOrientation>,
    mut q_portals: Query<&mut ActorPortals>,
    mut gizmos: Gizmos<PortalGizmo>,
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

            // Draw the portal with the updated radius
            boundary.draw_portal(
                &mut gizmos,
                approaching,
                config.color_approaching,
                config.resolution,
                &orientation,
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
        let scale_factor =
            (approaching.actor_distance_to_wall / approaching.boundary_distance_shrink).clamp(0.0, 1.0);
        min_radius + (max_radius - min_radius) * scale_factor
    }
}

fn draw_emerging_portals(
    time: Res<Time>,
    boundary: Res<Boundary>,
    config: Res<PortalConfig>,
    orientation: Res<CameraOrientation>,
    mut q_portals: Query<&mut ActorPortals>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    for mut portal in q_portals.iter_mut() {
        if let Some(ref mut emerging) = portal.emerging {
            if let Some(emerging_start) = emerging.fade_out_started {
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
                    boundary.draw_portal(
                        &mut gizmos,
                        emerging,
                        config.color_emerging,
                        config.resolution,
                        &orientation,
                    );
                }

                // Remove visual after the emerging duration is complete
                if elapsed_time >= emerging_duration {
                    portal.emerging = None;
                }
            }
        }
    }
}
