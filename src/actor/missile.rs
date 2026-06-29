use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::action::TriggerState;
use bevy_enhanced_input::action::events as input_events;
use bevy_enhanced_input::prelude::Action;
use bevy_enhanced_input::prelude::ActionOf;
use bevy_inspector_egui::InspectorOptions;
use bevy_kana::Position;
use input_events::Start;

use super::Teleporter;
use super::constants::GLTF_ROTATION_X;
use super::constants::LOCKED_AXES_2D;
use super::constants::MAX_MISSILE_ANGULAR_VELOCITY;
use super::constants::MAX_MISSILE_LINEAR_VELOCITY;
use super::constants::MISSILE_BASE_VELOCITY;
use super::constants::MISSILE_COLLIDER_MARGIN;
use super::constants::MISSILE_COLLISION_DAMAGE;
use super::constants::MISSILE_ENTITY_NAME;
use super::constants::MISSILE_FORWARD_DISTANCE_SCALAR;
use super::constants::MISSILE_HEALTH;
use super::constants::MISSILE_MASS;
use super::constants::MISSILE_RESTITUTION;
use super::constants::MISSILE_SCALE;
use super::constants::MISSILE_SPAWN_TIMER_SECONDS;
use super::constants::NO_GRAVITY_SCALE;
use super::game_layer::GameLayer;
use super::settings;
use super::settings::ActorSettings;
use super::settings::ColliderType;
use super::settings::Spawnability;
use super::spaceship::ContinuousFire;
use super::spaceship::Spaceship;
use super::teleport::TeleportStatus;
use crate::camera::RenderLayer;
use crate::despawn;
use crate::input::ShipControlsContext;
use crate::input::ShipFire;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;

// TODO: Decide whether `Missile` should stay a unit `Component` or gain private
// state before adding a constructor; current `Missile` instances come from
// `spawn_missile`.
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    LockedAxes = LOCKED_AXES_2D
)]
pub(super) struct Missile;

pub(super) struct MissilePlugin;

impl Plugin for MissilePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(initialize_missile)
            .add_observer(on_fire_input)
            .add_systems(
                FixedUpdate,
                fire_missile_continuous.in_set(InGameSet::UserInput),
            )
            .add_systems(
                FixedUpdate,
                missile_movement.in_set(InGameSet::EntityUpdates),
            );
    }
}

#[derive(Resource, Reflect, InspectorOptions, Debug, Clone, Deref, DerefMut)]
#[reflect(Resource)]
pub(super) struct MissileSettings {
    #[deref]
    pub(super) actor_settings:          ActorSettings,
    pub(super) forward_distance_scalar: f32,
    pub(super) base_velocity:           f32,
}

impl Default for MissileSettings {
    fn default() -> Self {
        Self {
            actor_settings:          ActorSettings {
                spawnability:             Spawnability::Enabled,
                angular_damping:          None,
                collider_margin:          MISSILE_COLLIDER_MARGIN,
                collider_type:            ColliderType::Cuboid,
                collision_damage:         MISSILE_COLLISION_DAMAGE,
                collision_layers:         CollisionLayers::new(
                    [GameLayer::Missile],
                    [GameLayer::Asteroid],
                ),
                gravity_scale:            NO_GRAVITY_SCALE,
                health:                   MISSILE_HEALTH,
                linear_damping:           None,
                locked_axes:              LockedAxes::new().lock_translation_z(),
                mass:                     MISSILE_MASS,
                max_angular_velocity:     MAX_MISSILE_ANGULAR_VELOCITY,
                max_linear_velocity:      MAX_MISSILE_LINEAR_VELOCITY,
                render_layer:             RenderLayer::Game,
                restitution:              MISSILE_RESTITUTION,
                restitution_combine_rule: CoefficientCombine::Max,
                rigid_body:               RigidBody::Dynamic,
                scene:                    Handle::default(),
                spawn_timer_seconds:      Some(MISSILE_SPAWN_TIMER_SECONDS),
                transform:                Transform::from_rotation(
                    Quat::from_rotation_x(GLTF_ROTATION_X) * Quat::from_rotation_z(PI),
                )
                .with_scale(Vec3::splat(MISSILE_SCALE)),
                spawn_timer:              None,
            },
            forward_distance_scalar: MISSILE_FORWARD_DISTANCE_SCALAR,
            base_velocity:           MISSILE_BASE_VELOCITY,
        }
    }
}

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
#[reflect(Component)]
pub(super) struct MissilePosition {
    pub(super) total_distance:    f32,
    pub(super) traveled_distance: f32,
    remaining_distance:           f32,
    pub(super) last_position:     Option<Position>,
}

impl MissilePosition {
    pub(super) const fn new(total_distance: f32) -> Self {
        Self {
            total_distance,
            traveled_distance: 0.,
            remaining_distance: 0.,
            last_position: None,
        }
    }
}

type ShipFireStateQuery<'w, 's> = Single<
    'w,
    's,
    &'static TriggerState,
    (With<Action<ShipFire>>, With<ActionOf<ShipControlsContext>>),
>;

fn initialize_missile(
    missile: On<Add, Missile>,
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut missile_settings: ResMut<MissileSettings>,
    transform_and_linear_velocity: Single<(&Transform, &LinearVelocity), With<Spaceship>>,
) {
    let missile_position = MissilePosition::new(boundary.max_missile_distance());

    let (spaceship_transform, spaceship_velocity) = *transform_and_linear_velocity;

    let transform = initialize_transform(spaceship_transform, &missile_settings);

    // Calculate velocity: forward direction * base_velocity + spaceship velocity
    let (linear_velocity, angular_velocity) = calculate_missile_velocity(
        spaceship_transform,
        spaceship_velocity,
        missile_settings.base_velocity,
    );

    commands
        .entity(missile.entity)
        .insert(missile_position)
        .insert(transform)
        .insert(linear_velocity)
        .insert(angular_velocity);

    settings::insert_configured_components(
        &mut commands,
        &mut missile_settings.actor_settings,
        missile.entity,
    );
}

fn initialize_transform(
    spaceship_transform: &Transform,
    missile_settings: &MissileSettings,
) -> Transform {
    // Calculate transform and velocity from spaceship position
    let forward = spaceship_transform.forward();
    let spawn_position =
        spaceship_transform.translation + forward * missile_settings.forward_distance_scalar;

    // Combine rotations: spaceship rotation * missile settings rotation
    let combined_rotation =
        spaceship_transform.rotation * missile_settings.actor_settings.transform.rotation;

    Transform {
        translation: spawn_position,
        rotation:    combined_rotation,
        scale:       missile_settings.actor_settings.transform.scale,
    }
}

fn on_fire_input(_trigger: On<Start<ShipFire>>, mut commands: Commands) {
    commands.run_system_cached(fire_missile_command);
}

/// Reusable on-demand command for firing a single `Missile`.
fn fire_missile_command(
    mut commands: Commands,
    continuous_fire_enabled: Single<Option<&ContinuousFire>, With<Spaceship>>,
    missile_settings: Res<MissileSettings>,
) {
    if missile_settings.spawnability == Spawnability::Disabled {
        return;
    }

    if continuous_fire_enabled.is_some() {
        return;
    }

    commands.spawn((Missile, Name::new(MISSILE_ENTITY_NAME)));
}

fn fire_missile_continuous(
    mut commands: Commands,
    continuous_fire_enabled: Single<Option<&ContinuousFire>, With<Spaceship>>,
    mut missile_settings: ResMut<MissileSettings>,
    fire_state: ShipFireStateQuery,
    time: Res<Time>,
) {
    if continuous_fire_enabled.is_none() || missile_settings.spawnability == Spawnability::Disabled
    {
        return;
    }

    let Some(timer) = missile_settings.spawn_timer.as_mut() else {
        return;
    };
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    if **fire_state == TriggerState::None {
        return;
    }

    commands.spawn((Missile, Name::new(MISSILE_ENTITY_NAME)));
}

/// Updates `MissilePosition` from `Missile` movement so missiles despawn after
/// reaching `MissilePosition::total_distance`.
fn missile_movement(
    mut commands: Commands,
    mut missile_position_query: Query<(Entity, &Transform, &mut MissilePosition, &Teleporter)>,
) {
    for (entity, transform, mut missile_position, teleporter) in &mut missile_position_query {
        let current_position = Position(transform.translation);

        if let Some(last_position) = missile_position.last_position {
            // Calculate the distance traveled since the last update
            let distance_traveled = if teleporter.teleport_status == TeleportStatus::JustTeleported
            {
                0.0
            } else {
                last_position.distance(current_position)
            };

            // Update the total traveled distance
            missile_position.traveled_distance += distance_traveled;
            missile_position.remaining_distance =
                missile_position.total_distance - missile_position.traveled_distance;
        }

        // Always update the last position.
        missile_position.last_position = Some(current_position);

        if missile_position.traveled_distance >= missile_position.total_distance {
            despawn::despawn(&mut commands, entity);
        }
    }
}

fn calculate_missile_velocity(
    spaceship_transform: &Transform,
    spaceship_velocity: &LinearVelocity,
    base_velocity: f32,
) -> (LinearVelocity, AngularVelocity) {
    let forward = spaceship_transform.forward();
    let mut velocity = forward * base_velocity;
    velocity += **spaceship_velocity;
    (LinearVelocity(velocity), AngularVelocity::ZERO)
}
