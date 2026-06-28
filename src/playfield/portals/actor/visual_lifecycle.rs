use avian3d::prelude::*;
use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use bevy_kana::Position;

use super::boundary_geometry;
use super::boundary_geometry::PhysicsBurst;
use crate::actor;
use crate::actor::TeleportStatus;
use crate::actor::Teleporter;
use crate::despawn::Deaderoid;
use crate::orientation::CameraOrientation;
use crate::playfield::Boundary;
use crate::playfield::BoundaryVolume;
use crate::playfield::boundary::PortalActorKind;
use crate::playfield::constants::PORTAL_MIN_RADIUS_FRACTION;
use crate::playfield::portals::Portal;
use crate::playfield::portals::PortalGizmo;
use crate::playfield::portals::constants::DEFAULT_PORTAL_FACE_COUNT;
use crate::playfield::portals::settings::PortalSettings;

#[derive(Component, Default)]
pub(crate) struct ActorPortals {
    pub(super) approaching_portal: Option<Portal>,
    pub(super) emerging_portal:    Option<Portal>,
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
            portal,
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
    if teleporter.teleport_status == TeleportStatus::JustTeleported
        && let Some(teleported_position) = teleporter.position
    {
        if boundary_geometry::physics_burst(teleported_position, boundary_transform)
            == PhysicsBurst::Active
        {
            actor_portals.emerging_portal = None;
            return;
        }

        let initial_face =
            boundary_geometry::get_face_for_position(teleported_position, boundary_transform);
        let (snapped_position, final_face) = boundary_geometry::snap_and_get_face(
            teleported_position,
            initial_face,
            boundary_transform,
        );

        actor_portals.emerging_portal = Some(Portal {
            actor_distance_to_wall: 0.0,
            boundary_face: final_face,
            position: snapped_position,
            fade_out_started: Some(time.elapsed_secs()),
            ..portal
        });
    } else if let Some(ref mut emerging_portal) = actor_portals.emerging_portal
        && emerging_portal.radius <= portal_settings.minimum_radius
    {
        actor_portals.emerging_portal = None;
    }
}

fn handle_approaching_visual(
    boundary_transform: &Transform,
    portal: Portal,
    portal_settings: &PortalSettings,
    time: &Time,
    actor_portals: &mut Mut<ActorPortals>,
) {
    if let Some(collision_point) = boundary_geometry::find_edge_point(
        portal.position,
        portal.actor_direction,
        boundary_transform,
    ) {
        let actor_distance_to_wall = portal.position.distance(collision_point);

        if actor_distance_to_wall <= portal.boundary_distance_approach {
            let face =
                boundary_geometry::get_face_for_position(collision_point, boundary_transform);
            let temporary_portal = Portal {
                position: collision_point,
                boundary_face: face,
                radius: portal.radius,
                ..portal
            };
            let current_face_count =
                Boundary::calculate_portal_face_count(&temporary_portal, boundary_transform);
            let previous_face_count = actor_portals
                .approaching_portal
                .as_ref()
                .map_or(DEFAULT_PORTAL_FACE_COUNT, |approaching_portal| {
                    approaching_portal.face_count
                });

            let smoothed_position = if current_face_count == previous_face_count {
                boundary_geometry::smooth_circle_position(
                    actor_portals,
                    collision_point,
                    face,
                    portal_settings,
                )
            } else {
                collision_point
            };

            let (snapped_position, snapped_face) =
                boundary_geometry::snap_and_get_face(smoothed_position, face, boundary_transform);

            actor_portals.approaching_portal = Some(Portal {
                actor_distance_to_wall,
                boundary_face: snapped_face,
                face_count: current_face_count,
                position: snapped_position,
                ..portal
            });
            return;
        }
    }

    if let Some(approaching_portal) = &mut actor_portals.approaching_portal {
        if boundary_geometry::physics_burst(portal.position, boundary_transform)
            == PhysicsBurst::Active
        {
            actor_portals.approaching_portal = None;
        } else if approaching_portal.fade_out_started.is_none() {
            approaching_portal.fade_out_started = Some(time.elapsed_secs());
        }
    }
}

pub(super) fn update_approaching_portals(
    time: Res<Time>,
    portal_settings: Res<PortalSettings>,
    mut portals_query: Query<&mut ActorPortals>,
) {
    for mut actor_portals in &mut portals_query {
        if let Some(ref mut approaching_portal) = actor_portals.approaching_portal {
            let radius = get_approaching_portal_radius(approaching_portal);

            if let Some(fade_out_start) = approaching_portal.fade_out_started {
                let elapsed_time = time.elapsed_secs() - fade_out_start;
                let fade_out_duration = portal_settings.fadeout_duration;
                let below_minimum = approaching_portal.radius < portal_settings.minimum_radius;
                if elapsed_time >= fade_out_duration || below_minimum {
                    actor_portals.approaching_portal = None;
                    continue;
                }

                let fade_factor = (1.0 - (elapsed_time / fade_out_duration)).clamp(0.0, 1.0);
                approaching_portal.radius *= fade_factor;
            } else {
                approaching_portal.radius = radius;
            }
        }
    }
}

pub(super) fn draw_approaching_portals(
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    portal_settings: Res<PortalSettings>,
    camera_orientation: Res<CameraOrientation>,
    portals_query: Query<(&ActorPortals, Option<&Deaderoid>)>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    for (actor_portals, actor_kind) in portals_query.iter() {
        if let Some(ref approaching_portal) = actor_portals.approaching_portal {
            Boundary::draw_portal(
                &mut gizmos,
                approaching_portal,
                portal_settings.color_approaching,
                portal_settings.resolution,
                &camera_orientation,
                actor_kind.map_or(PortalActorKind::Nateroid, |_| PortalActorKind::Deaderoid),
                boundary_transform,
            );
        }
    }
}

fn get_approaching_portal_radius(approaching_portal: &Portal) -> f32 {
    let max_radius = approaching_portal.radius;
    let min_radius = max_radius * PORTAL_MIN_RADIUS_FRACTION;

    if approaching_portal.actor_distance_to_wall > approaching_portal.boundary_distance_shrink {
        max_radius
    } else {
        let scale_factor = (approaching_portal.actor_distance_to_wall
            / approaching_portal.boundary_distance_shrink)
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
        if let Some(ref mut emerging_portal) = actor_portals.emerging_portal
            && let Some(emerging_start) = emerging_portal.fade_out_started
        {
            let elapsed_time = time.elapsed_secs() - emerging_start;
            let emerging_duration = portal_settings.fadeout_duration;
            let progress = (elapsed_time / emerging_duration).clamp(0.0, 1.0);
            let initial_radius = emerging_portal.radius;
            let radius = initial_radius * (1.0 - progress);

            if radius > 0.0 {
                emerging_portal.radius = radius;
            }

            if elapsed_time >= emerging_duration {
                actor_portals.emerging_portal = None;
            }
        }
    }
}

pub(super) fn draw_emerging_portals(
    boundary_volume_query: Query<&Transform, With<BoundaryVolume>>,
    portal_settings: Res<PortalSettings>,
    camera_orientation: Res<CameraOrientation>,
    portals_query: Query<(&ActorPortals, Option<&Deaderoid>)>,
    mut gizmos: Gizmos<PortalGizmo>,
) {
    let Ok(boundary_transform) = boundary_volume_query.single() else {
        return;
    };

    for (actor_portals, actor_kind) in portals_query.iter() {
        if let Some(ref emerging_portal) = actor_portals.emerging_portal {
            Boundary::draw_portal(
                &mut gizmos,
                emerging_portal,
                portal_settings.color_emerging,
                portal_settings.resolution,
                &camera_orientation,
                actor_kind.map_or(PortalActorKind::Nateroid, |_| PortalActorKind::Deaderoid),
                boundary_transform,
            );
        }
    }
}
