use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::action::TriggerState;
use bevy_enhanced_input::action::events as input_events;
use bevy_enhanced_input::prelude::Action;
use bevy_enhanced_input::prelude::ActionOf;

use super::Teleporter;
use super::actor_settings::LOCKED_AXES_2D;
use super::actor_settings::insert_configured_components;
use super::actor_template::MissileSettings;
use super::spaceship::ContinuousFire;
use super::spaceship::Spaceship;
use crate::input::ShipControlsContext;
use crate::input::ShipFire;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::traits::TransformExt;

pub struct MissilePlugin;

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

// todo: #rustquestion - how can i make it so that new has to be used and
// DrawDirection isn't constructed directly - i still need the fields visible
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Missile;

#[derive(Component, Reflect, Copy, Clone, Debug, Default)]
#[reflect(Component)]
pub struct MissilePosition {
    pub total_distance:     f32,
    pub traveled_distance:  f32,
    remaining_distance:     f32,
    pub last_position:      Option<Vec3>,
    last_teleport_position: Option<Vec3>, // Add this field
}

impl MissilePosition {
    pub const fn new(total_distance: f32) -> Self {
        Self {
            total_distance,
            traveled_distance: 0.,
            remaining_distance: 0.,
            last_position: None,
            last_teleport_position: None,
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
    mut settings: ResMut<MissileSettings>,
    transform_and_linvel: Single<(&Transform, &LinearVelocity), With<Spaceship>>,
) {
    let missile_position = MissilePosition::new(boundary.max_missile_distance());

    let (spaceship_transform, spaceship_velocity) = *transform_and_linvel;

    let transform = initialize_transform(spaceship_transform, &settings);

    // Calculate velocity: forward direction * base_velocity + spaceship velocity
    let (linear_velocity, angular_velocity) = calculate_missile_velocity(
        spaceship_transform,
        spaceship_velocity,
        settings.base_velocity,
    );

    commands
        .entity(missile.entity)
        .insert(missile_position)
        .insert(transform)
        .insert(linear_velocity)
        .insert(angular_velocity);

    insert_configured_components(&mut commands, &mut settings.actor_settings, missile.entity);
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

    Transform::from_trs(
        spawn_position,
        combined_rotation,
        missile_settings.actor_settings.transform.scale,
    )
}

fn on_fire_input(_trigger: On<input_events::Start<ShipFire>>, mut commands: Commands) {
    commands.run_system_cached(fire_missile_command);
}

/// Reusable on-demand command for firing a single missile.
fn fire_missile_command(
    mut commands: Commands,
    continuous_fire_enabled: Single<Option<&ContinuousFire>, With<Spaceship>>,
    missile_settings: Res<MissileSettings>,
) {
    if !missile_settings.spawnable {
        return;
    }

    if continuous_fire_enabled.is_some() {
        return;
    }

    commands.spawn((Missile, Name::new("Missile")));
}

fn fire_missile_continuous(
    mut commands: Commands,
    continuous_fire_enabled: Single<Option<&ContinuousFire>, With<Spaceship>>,
    mut missile_settings: ResMut<MissileSettings>,
    fire_state: ShipFireStateQuery,
    time: Res<Time>,
) {
    if continuous_fire_enabled.is_none() || !missile_settings.spawnable {
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

    commands.spawn((Missile, Name::new("Missile")));
}

/// we update missile movement so that it can be despawned after it has traveled
/// its total distance
fn missile_movement(mut query: Query<(&Transform, &mut MissilePosition, &Teleporter)>) {
    for (transform, mut missile, teleporter) in &mut query {
        let current_position = transform.translation;

        if let Some(last_position) = missile.last_position {
            // Calculate the distance traveled since the last update
            let distance_traveled = if teleporter.just_teleported {
                0.0
            } else {
                last_position.distance(current_position)
            };

            // Update the total traveled distance
            missile.traveled_distance += distance_traveled;
            missile.remaining_distance = missile.total_distance - missile.traveled_distance;

            // Update the last teleport position if the missile wrapped
            if teleporter.just_teleported {
                missile.last_teleport_position = Some(current_position);
            }
        }

        // Always update last_position
        missile.last_position = Some(current_position);
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
