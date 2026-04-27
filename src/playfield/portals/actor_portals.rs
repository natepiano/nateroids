use avian3d::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy_kana::Position;

use super::Portal;
use super::PortalGizmo;
use super::portal_settings::PortalSettings;
use crate::actor;
use crate::actor::Deaderoid;
use crate::actor::TeleportStatus;
use crate::actor::Teleporter;
use crate::orientation::CameraOrientation;
use crate::playfield::Boundary;
use crate::playfield::BoundaryVolume;
use crate::playfield::boundary::PortalActorKind;
use crate::playfield::boundary_face::BoundaryFace;
use crate::playfield::constants::BOUNDARY_SNAP_EPSILON;
use crate::playfield::constants::PORTAL_MIN_RADIUS_FRACTION;
use crate::playfield::constants::PORTAL_PHYSICS_BURST_MULTIPLIER;

#[derive(Component, Default)]
pub struct ActorPortals {
    approaching: Option<Portal>,
    emerging:    Option<Portal>,
}

pub(super) fn init_portals(
    mut actor_query: Query<(
        &Aabb,
        &Transform,
        &LinearVelocity,
        &Teleporter,
        &mut ActorPortals,
    )>,
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    portal_settings: Res<PortalSettings>,
    time: Res<Time>,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    let boundary_size = boundary_transform
        .scale
        .x
        .min(boundary_transform.scale.y)
        .min(boundary_transform.scale.z);
    let boundary_distance_approach = boundary_size * portal_settings.distance_approach;
    let boundary_distance_shrink = boundary_size * portal_settings.distance_shrink;

    for (aabb, transform, velocity, teleporter, mut actor_portals) in &mut actor_query {
        let radius =
            actor::aabb_max_dimension(aabb).max(portal_settings.smallest) * portal_settings.scalar;

        let portal = Portal {
            actor_direction: velocity.normalize_or_zero(),
            position: Position(transform.translation),
            boundary_distance_approach,
            boundary_distance_shrink,
            radius,
            ..default()
        };

        handle_approaching_visual(
            boundary_transform,
            portal.clone(),
            &portal_settings,
            &time,
            &mut actor_portals,
        );
        handle_emerging_visual(
            portal.clone(),
            &portal_settings,
            teleporter,
            &time,
            &mut actor_portals,
            boundary_transform,
        );
    }
}

fn handle_emerging_visual(
    portal: Portal,
    portal_settings: &PortalSettings,
    teleporter: &Teleporter,
    time: &Time,
    actor_portals: &mut Mut<ActorPortals>,
    boundary_transform: &Transform,
) {
    if teleporter.status == TeleportStatus::JustTeleported
        && let Some(teleported_position) = teleporter.position
    {
        if is_physics_burst(teleported_position, boundary_transform) {
            actor_portals.emerging = None;
            return;
        }

        let initial_face = get_face_for_position(teleported_position, boundary_transform);
        let (snapped_position, final_face) =
            snap_and_get_face(teleported_position, initial_face, boundary_transform);

        actor_portals.emerging = Some(Portal {
            actor_distance_to_wall: 0.0,
            face: final_face,
            position: snapped_position,
            fade_out_started: Some(time.elapsed_secs()),
            ..portal
        });
    } else if let Some(ref mut emerging) = actor_portals.emerging
        && emerging.radius <= portal_settings.minimum_radius
    {
        actor_portals.emerging = None;
    }
}

fn handle_approaching_visual(
    boundary_transform: &Transform,
    portal: Portal,
    portal_settings: &PortalSettings,
    time: &Time,
    actor_portals: &mut Mut<ActorPortals>,
) {
    if let Some(collision_point) =
        find_edge_point(portal.position, portal.actor_direction, boundary_transform)
    {
        let actor_distance_to_wall = portal.position.distance(collision_point);

        if actor_distance_to_wall <= portal.boundary_distance_approach {
            let face = get_face_for_position(collision_point, boundary_transform);
            let temp_portal = Portal {
                position: collision_point,
                face,
                radius: portal.radius,
                ..portal
            };
            let current_face_count =
                Boundary::calculate_portal_face_count(&temp_portal, boundary_transform);
            let previous_face_count = actor_portals
                .approaching
                .as_ref()
                .map_or(1, |approaching| approaching.face_count);

            let smoothed_position = if current_face_count == previous_face_count {
                smooth_circle_position(actor_portals, collision_point, face, portal_settings)
            } else {
                collision_point
            };

            let (snapped_position, snapped_face) =
                snap_and_get_face(smoothed_position, face, boundary_transform);

            actor_portals.approaching = Some(Portal {
                actor_distance_to_wall,
                face: snapped_face,
                face_count: current_face_count,
                position: snapped_position,
                ..portal
            });
            return;
        }
    }

    if let Some(approaching) = &mut actor_portals.approaching {
        if is_physics_burst(portal.position, boundary_transform) {
            actor_portals.approaching = None;
        } else if approaching.fade_out_started.is_none() {
            approaching.fade_out_started = Some(time.elapsed_secs());
        }
    }
}

fn is_physics_burst(position: Position, boundary_transform: &Transform) -> bool {
    let boundary_half_size = boundary_transform.scale / 2.0;
    let max_distance_from_center = position.distance(boundary_transform.translation);
    let boundary_diagonal = boundary_half_size.length();
    max_distance_from_center > boundary_diagonal * PORTAL_PHYSICS_BURST_MULTIPLIER
}

fn snap_and_get_face(
    position: Position,
    initial_face: BoundaryFace,
    boundary_transform: &Transform,
) -> (Position, BoundaryFace) {
    let snapped_position =
        snap_position_to_boundary_face(position, initial_face, boundary_transform);
    let final_face = get_face_for_position(snapped_position, boundary_transform);
    (snapped_position, final_face)
}

/// Snaps a position to slightly inside the given boundary face.
/// Offsets by epsilon to prevent false-positive overextension detection that would trigger
/// corner wrapping arcs. Clamps perpendicular axes to handle corner/edge teleportation cases.
fn snap_position_to_boundary_face(
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
fn get_face_for_position(position: Position, transform: &Transform) -> BoundaryFace {
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

fn find_edge_point(origin: Position, direction: Vec3, transform: &Transform) -> Option<Position> {
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

fn smooth_circle_position(
    actor_portals: &Mut<ActorPortals>,
    collision_point: Position,
    current_boundary_wall_face: BoundaryFace,
    portal_settings: &PortalSettings,
) -> Position {
    if let Some(approaching) = &actor_portals.approaching {
        let smoothing_factor = portal_settings.movement_smoothing_factor;

        if approaching
            .normal()
            .dot(current_boundary_wall_face.get_normal())
            > portal_settings.direction_change_factor
        {
            approaching.position.lerp(collision_point, smoothing_factor)
        } else {
            collision_point
        }
    } else {
        collision_point
    }
}

pub(super) fn update_approaching_portals(
    time: Res<Time>,
    portal_settings: Res<PortalSettings>,
    mut portals_query: Query<&mut ActorPortals>,
) {
    for mut actor_portals in &mut portals_query {
        if let Some(ref mut approaching) = actor_portals.approaching {
            let radius = get_approaching_radius(approaching);

            if let Some(fade_out_start) = approaching.fade_out_started {
                let elapsed_time = time.elapsed_secs() - fade_out_start;
                let fade_out_duration = portal_settings.fadeout_duration;
                let below_minimum = approaching.radius < portal_settings.minimum_radius;
                if elapsed_time >= fade_out_duration || below_minimum {
                    actor_portals.approaching = None;
                    continue;
                }

                let fade_factor = (1.0 - (elapsed_time / fade_out_duration)).clamp(0.0, 1.0);
                approaching.radius *= fade_factor;
            } else {
                approaching.radius = radius;
            }
        }
    }
}

pub(super) fn draw_approaching_portals(
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    portal_settings: Res<PortalSettings>,
    orientation: Res<CameraOrientation>,
    portals_query: Query<(&ActorPortals, Option<&Deaderoid>)>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    for (actor_portals, actor_kind) in portals_query.iter() {
        if let Some(ref approaching) = actor_portals.approaching {
            Boundary::draw_portal(
                &mut gizmos,
                approaching,
                portal_settings.color_approaching,
                portal_settings.resolution,
                &orientation,
                actor_kind.map_or(PortalActorKind::Nateroid, |_| PortalActorKind::Deaderoid),
                boundary_transform,
            );
        }
    }
}

fn get_approaching_radius(approaching: &Portal) -> f32 {
    let max_radius = approaching.radius;
    let min_radius = max_radius * PORTAL_MIN_RADIUS_FRACTION;

    if approaching.actor_distance_to_wall > approaching.boundary_distance_shrink {
        max_radius
    } else {
        let scale_factor = (approaching.actor_distance_to_wall
            / approaching.boundary_distance_shrink)
            .clamp(0.0, 1.0);
        (max_radius - min_radius).mul_add(scale_factor, min_radius)
    }
}

pub(super) fn update_emerging_portals(
    time: Res<Time>,
    portal_settings: Res<PortalSettings>,
    mut portals_query: Query<&mut ActorPortals>,
) {
    for mut actor_portals in &mut portals_query {
        if let Some(ref mut emerging) = actor_portals.emerging
            && let Some(emerging_start) = emerging.fade_out_started
        {
            let elapsed_time = time.elapsed_secs() - emerging_start;
            let emerging_duration = portal_settings.fadeout_duration;
            let progress = (elapsed_time / emerging_duration).clamp(0.0, 1.0);
            let initial_radius = emerging.radius;
            let radius = initial_radius * (1.0 - progress);

            if radius > 0.0 {
                emerging.radius = radius;
            }

            if elapsed_time >= emerging_duration {
                actor_portals.emerging = None;
            }
        }
    }
}

pub(super) fn draw_emerging_portals(
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    portal_settings: Res<PortalSettings>,
    orientation: Res<CameraOrientation>,
    portals_query: Query<(&ActorPortals, Option<&Deaderoid>)>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    for (actor_portals, actor_kind) in portals_query.iter() {
        if let Some(ref emerging) = actor_portals.emerging {
            Boundary::draw_portal(
                &mut gizmos,
                emerging,
                portal_settings.color_emerging,
                portal_settings.resolution,
                &orientation,
                actor_kind.map_or(PortalActorKind::Nateroid, |_| PortalActorKind::Deaderoid),
                boundary_transform,
            );
        }
    }
}
