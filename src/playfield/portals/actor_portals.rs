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

        let initial_face = Boundary::get_face_for_position(teleported_position, boundary_transform);
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
        Boundary::find_edge_point(portal.position, portal.actor_direction, boundary_transform)
    {
        let actor_distance_to_wall = portal.position.distance(collision_point);

        if actor_distance_to_wall <= portal.boundary_distance_approach {
            let face = Boundary::get_face_for_position(collision_point, boundary_transform);
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
        Boundary::snap_position_to_boundary_face(position, initial_face, boundary_transform);
    let final_face = Boundary::get_face_for_position(snapped_position, boundary_transform);
    (snapped_position, final_face)
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
