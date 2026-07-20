use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::world_serialization::WorldAsset;
use bevy::world_serialization::WorldAssetRoot;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

use super::aabb::PendingCollider;
use super::constants::ACTOR_COLLIDER_MARGIN_MAX;
use super::constants::ACTOR_COLLIDER_MARGIN_MIN;
use super::constants::ACTOR_DAMPING_MAX;
use super::constants::ACTOR_DAMPING_MIN;
use super::constants::ACTOR_MASS_MAX;
use super::constants::ACTOR_MASS_MIN;
use super::constants::ACTOR_MAX_VELOCITY_MAX;
use super::constants::ACTOR_MAX_VELOCITY_MIN;
use super::constants::ACTOR_RESTITUTION_MAX;
use super::constants::ACTOR_RESTITUTION_MIN;
use super::missile::Missile;
use super::missile::MissileSettings;
use super::nateroid::Nateroid;
use super::nateroid::NateroidSettings;
use super::spaceship::Spaceship;
use super::spaceship::SpaceshipSettings;
use crate::asset_loader::AssetsState;
use crate::asset_loader::SceneAssets;
use crate::camera::RenderLayer;
use crate::input::InspectMissileSwitch;
use crate::input::InspectNateroidSwitch;
use crate::input::InspectSpaceshipSwitch;
use crate::switches;
use crate::switches::Switch;

// `ActorSettingsPlugin` initializes the actor setting resources when
// `AssetsState::Loaded` starts: `ActorSettings`, `MissileSettings`,
// `NateroidSettings`, and `SpaceshipSettings`.
pub(super) struct ActorSettingsPlugin;

impl Plugin for ActorSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AssetsState::Loaded), initialize_actors)
            .add_observer(propagate_render_layers_on_spawn)
            .add_plugins(
                ResourceInspectorPlugin::<MissileSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectMissile)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<NateroidSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectNateroid)),
            )
            .add_plugins(
                ResourceInspectorPlugin::<SpaceshipSettings>::default()
                    .run_if(switches::is_switch_on(Switch::InspectSpaceship)),
            );
        bind_action_switch!(
            app,
            InspectMissileSwitch,
            MissileInspectorEvent,
            Switch::InspectMissile
        );
        bind_action_switch!(
            app,
            InspectNateroidSwitch,
            NateroidInspectorEvent,
            Switch::InspectNateroid
        );
        bind_action_switch!(
            app,
            InspectSpaceshipSwitch,
            SpaceshipInspectorEvent,
            Switch::InspectSpaceship
        );
    }
}

event!(MissileInspectorEvent);
event!(NateroidInspectorEvent);
event!(SpaceshipInspectorEvent);

#[derive(Reflect, InspectorOptions, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Spawnability {
    #[default]
    Enabled,
    Disabled,
}

#[derive(Reflect, InspectorOptions, Clone, Debug)]
#[reflect(InspectorOptions)]
pub(crate) struct ActorSettings {
    pub(crate) spawnability:             Spawnability,
    #[inspector(
        min = ACTOR_DAMPING_MIN,
        max = ACTOR_DAMPING_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) angular_damping:          Option<f32>,
    #[inspector(
        min = ACTOR_COLLIDER_MARGIN_MIN,
        max = ACTOR_COLLIDER_MARGIN_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) collider_margin:          f32,
    pub(crate) collider_type:            ColliderType,
    pub(crate) collision_damage:         f32,
    pub(crate) collision_layers:         CollisionLayers,
    pub(crate) gravity_scale:            f32,
    pub(crate) health:                   f32,
    #[inspector(
        min = ACTOR_DAMPING_MIN,
        max = ACTOR_DAMPING_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) linear_damping:           Option<f32>,
    pub(crate) locked_axes:              LockedAxes,
    #[inspector(
        min = ACTOR_MASS_MIN,
        max = ACTOR_MASS_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) mass:                     f32,
    #[inspector(
        min = ACTOR_MAX_VELOCITY_MIN,
        max = ACTOR_MAX_VELOCITY_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) max_angular_velocity:     f32,
    #[inspector(
        min = ACTOR_MAX_VELOCITY_MIN,
        max = ACTOR_MAX_VELOCITY_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) max_linear_velocity:      f32,
    pub(crate) render_layer:             RenderLayer,
    #[inspector(
        min = ACTOR_RESTITUTION_MIN,
        max = ACTOR_RESTITUTION_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) restitution:              f32,
    pub(crate) restitution_combine_rule: CoefficientCombine,
    pub(crate) rigid_body:               RigidBody,
    #[reflect(ignore)]
    pub(crate) scene:                    Handle<WorldAsset>,
    pub(crate) spawn_timer_seconds:      Option<f32>,
    pub(crate) transform:                Transform,
    #[reflect(ignore)]
    pub(crate) spawn_timer:              Option<Timer>,
}

impl ActorSettings {
    pub(super) fn reset_spawn_timer(&mut self) {
        self.spawn_timer = self
            .spawn_timer_seconds
            .map(|seconds| Timer::from_seconds(seconds, TimerMode::Repeating));
    }
}

#[derive(Reflect, Component, Clone, Debug, Default)]
#[reflect(Component)]
pub(crate) struct Health(pub f32);

#[derive(Reflect, Component, Clone, Debug, Default)]
#[reflect(Component)]
pub(super) struct CollisionDamage(pub f32);

#[derive(Reflect, Debug, Clone, PartialEq, Eq)]
pub(crate) enum ColliderType {
    Ball,
    Cuboid,
}

type ActorRenderLayersQuery<'w, 'a> =
    Query<'w, 'a, &'static RenderLayers, Or<(With<Missile>, With<Nateroid>, With<Spaceship>)>>;

/// Copies parent actor `RenderLayers` onto spawned scene descendants.
fn propagate_render_layers_on_spawn(
    add: On<Add, Children>,
    parent_render_layers_query: ActorRenderLayersQuery,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    // `add.entity` is an actor parent with `Missile`, `Nateroid`, or
    // `Spaceship`; its `Children` were added by scene spawning.
    if let Ok(parent_layers) = parent_render_layers_query.get(add.entity) {
        // `iter_descendants` reaches the GLTF child meshes that need matching
        // `RenderLayers` for the game camera and shadow pass.
        for descendant in children_query.iter_descendants(add.entity) {
            commands.entity(descendant).insert(parent_layers.clone());
        }
    }
}

pub(super) fn initialize_actors(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    let mut nateroid_settings = NateroidSettings::default();
    initialize_actor_settings(
        &mut nateroid_settings.actor_settings,
        &scene_assets.nateroid,
    );
    commands.insert_resource(nateroid_settings);

    let mut missile_settings = MissileSettings::default();
    initialize_actor_settings(&mut missile_settings.actor_settings, &scene_assets.missile);
    commands.insert_resource(missile_settings);

    let mut spaceship_settings = SpaceshipSettings::default();
    initialize_actor_settings(
        &mut spaceship_settings.actor_settings,
        &scene_assets.spaceship,
    );
    commands.insert_resource(spaceship_settings);
}

fn initialize_actor_settings(
    actor_settings: &mut ActorSettings,
    scene_handle: &Handle<WorldAsset>,
) {
    actor_settings.reset_spawn_timer();
    actor_settings.scene = scene_handle.clone();
}

/// Builds the components shared by every configured actor.
pub(super) fn configured_actor_scene(actor_settings: ActorSettings) -> impl Scene {
    let pending_collider = template(move |_| {
        Ok(PendingCollider {
            collider_type: actor_settings.collider_type.clone(),
            margin:        actor_settings.collider_margin,
            rigid_body:    actor_settings.rigid_body,
        })
    });
    let damping = (
        actor_settings
            .linear_damping
            .map(|damping| template_value(LinearDamping(damping))),
        actor_settings
            .angular_damping
            .map(|damping| template_value(AngularDamping(damping))),
    );

    bsn! {
        {pending_collider}
        CollisionDamage({actor_settings.collision_damage})
        template_value(actor_settings.collision_layers)
        GravityScale({actor_settings.gravity_scale})
        Health({actor_settings.health})
        Restitution {
            coefficient: {actor_settings.restitution},
            combine_rule: {actor_settings.restitution_combine_rule},
        }
        Mass({actor_settings.mass})
        MaxAngularSpeed({actor_settings.max_angular_velocity})
        MaxLinearSpeed({actor_settings.max_linear_velocity})
        template_value(actor_settings.render_layer.layers())
        WorldAssetRoot({actor_settings.scene})
        {damping}
    }
}
