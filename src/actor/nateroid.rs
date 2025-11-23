use std::collections::VecDeque;
use std::ops::Range;

use avian3d::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use rand::Rng;

use super::Teleporter;
use super::actor_config::Health;
use super::actor_config::LOCKED_AXES_2D;
use super::actor_config::insert_configured_components;
use super::actor_template::GameLayer;
use super::actor_template::NateroidConfig;
use crate::asset_loader;
use crate::asset_loader::SceneAssets;
use crate::despawn::despawn;
use crate::game_input::GameAction;
use crate::game_input::just_pressed;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
use crate::traits::TransformExt;
use crate::traits::UsizeExt;

// half the size of the boundary and only in the x,y plane
const SPAWN_WINDOW: Vec3 = Vec3::new(0.5, 0.5, 0.0);

#[derive(Resource)]
pub struct NateroidSpawnStats {
    /// Ring buffer tracking last N spawn attempts (true = success, false = failure)
    pub attempts:          VecDeque<bool>,
    pub last_warning_time: f32,
}

impl Default for NateroidSpawnStats {
    fn default() -> Self {
        Self {
            attempts:          VecDeque::with_capacity(50),
            last_warning_time: 0.0,
        }
    }
}

impl NateroidSpawnStats {
    const MAX_ATTEMPTS: usize = 50;

    pub fn record_attempt(&mut self, success: bool) {
        self.attempts.push_back(success);
        if self.attempts.len() > Self::MAX_ATTEMPTS {
            self.attempts.pop_front();
        }
    }

    pub fn success_rate(&self) -> f32 {
        if self.attempts.is_empty() {
            1.0 // No data - assume field is not crowded
        } else {
            let successes = self.attempts.iter().filter(|&&success| success).count();
            successes.to_f32() / self.attempts.len().to_f32()
        }
    }

    pub fn attempts_count(&self) -> usize { self.attempts.len() }

    pub fn successes_count(&self) -> usize {
        self.attempts.iter().filter(|&&success| success).count()
    }
}

pub struct NateroidPlugin;

impl Plugin for NateroidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NateroidSpawnStats>()
            .add_systems(
                OnEnter(asset_loader::AssetsState::Loaded),
                precompute_death_materials.after(super::actor_config::initialize_actor_configs),
            )
            .add_observer(initialize_nateroid)
            .add_systems(
                Update,
                (
                    apply_nateroid_materials_to_children,
                    debug_mesh_components.after(apply_nateroid_materials_to_children),
                    spawn_nateroid.in_set(InGameSet::EntityUpdates),
                    despawn_testaroid_on_teleport.in_set(InGameSet::EntityUpdates),
                    spawn_testaroid
                        .in_set(InGameSet::EntityUpdates)
                        .run_if(just_pressed(GameAction::SpawnTestaroid)),
                    spawn_test_missile
                        .in_set(InGameSet::EntityUpdates)
                        .run_if(just_pressed(GameAction::SpawnTestMissile)),
                    despawn_test_missiles.in_set(InGameSet::EntityUpdates),
                ),
            );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Nateroid;

#[derive(Component, Debug)]
pub struct Deaderoid {
    pub initial_scale:          Vec3,
    pub target_shrink:          f32,
    pub shrink_duration:        f32,
    pub elapsed_time:           f32,
    pub current_shrink:         f32,
    pub current_material_index: usize,
}

/// Precomputed materials for nateroid death animation at different transparency levels
#[derive(Resource)]
pub struct NateroidDeathMaterials {
    pub materials: Vec<Vec<Handle<StandardMaterial>>>,
}

/// Test nateroid component with configurable spawn position and velocity
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Testaroid {
    pub position: Vec3,
    pub velocity: Vec3,
}

fn spawn_nateroid(mut commands: Commands, mut config: ResMut<NateroidConfig>, time: Res<Time>) {
    if !config.spawnable {
        return;
    }

    let Some(spawn_timer) = config.spawn_timer.as_mut() else {
        return;
    };
    spawn_timer.tick(time.delta());

    if !spawn_timer.just_finished() {
        return;
    }

    commands.spawn((Nateroid, Name::new("Nateroid")));
}

fn despawn_testaroid_on_teleport(
    mut commands: Commands,
    query: Query<(Entity, &Teleporter), With<Testaroid>>,
) {
    for (entity, teleporter) in query.iter() {
        if teleporter.just_teleported {
            commands.entity(entity).insert(Health(-1.0));
        }
    }
}

fn spawn_testaroid(mut commands: Commands) {
    let testaroid = Testaroid {
        position: Vec3::new(-159., -85., 0.),
        velocity: Vec3::new(-20., 0., 0.),
    };

    commands.spawn((Nateroid, Name::new("Nateroid"), testaroid));
}

fn spawn_test_missile(mut commands: Commands, boundary: Res<Boundary>) {
    use rand::Rng;
    let mut rng = rand::rng();

    // Pick a random corner from the 4 front corners (positive z to ensure heading away from z=0)
    let half_size = boundary.transform.scale / 2.0;
    let corner_signs = Vec3::new(
        if rng.random::<bool>() { 1.0 } else { -1.0 },
        if rng.random::<bool>() { 1.0 } else { -1.0 },
        1.0, // Always positive z (front wall) to avoid crossing z=0 before reaching corner
    );
    let corner = boundary.transform.translation + half_size * corner_signs;

    // Target the corner directly (small offset for variety but guaranteed corner hit)
    let target_offset_radius = 1.0; // Very small offset to add variety
    let offset = Vec3::new(
        rng.random_range(-target_offset_radius..target_offset_radius),
        rng.random_range(-target_offset_radius..target_offset_radius),
        rng.random_range(-target_offset_radius..target_offset_radius),
    );
    let target = corner + offset;

    // Spawn near z=0 plane at random x,y position within boundary (z=5 to avoid immediate despawn)
    let spawn_position = Vec3::new(
        rng.random_range(-half_size.x..half_size.x)
            .mul_add(0.8, boundary.transform.translation.x), /* 80% of boundary */
        rng.random_range(-half_size.y..half_size.y)
            .mul_add(0.8, boundary.transform.translation.y),
        5.0, // Slightly offset from z=0 to avoid immediate despawn
    );

    // Calculate direction from spawn to target
    let direction = (target - spawn_position).normalize_or_zero();

    // Velocity aimed at target (moderate speed to see what's happening)
    let velocity = direction * 80.0;

    let test_missile = super::missile::TestMissile {
        position: spawn_position,
        velocity,
    };

    commands.spawn((
        super::missile::Missile,
        Name::new("TestMissile"),
        test_missile,
    ));
}

fn despawn_test_missiles(
    mut commands: Commands,
    query: Query<(Entity, &Transform, &super::missile::TestMissile)>,
) {
    for (entity, transform, _) in query.iter() {
        // Despawn when z crosses 0 (returned from back to front)
        if transform.translation.z.abs() < 0.5 {
            despawn(&mut commands, entity);
        }
    }
}

/// System that applies custom materials to nateroid mesh children (donut and icing)
fn apply_nateroid_materials_to_children(
    mut commands: Commands,
    nateroid_query: Query<Entity, (With<Nateroid>, Added<Children>)>,
    mesh_query: Query<(Entity, Option<&Name>), With<Mesh3d>>,
    children_query: Query<&Children>,
    scene_assets: Res<SceneAssets>,
) {
    let Some(donut_material) = &scene_assets.nateroid_donut_material else {
        return;
    };
    let Some(icing_material) = &scene_assets.nateroid_icing_material else {
        return;
    };

    for nateroid_entity in nateroid_query.iter() {
        debug!("Applying materials to nateroid {nateroid_entity:?} mesh children");

        let mut donut_count = 0;
        let mut icing_count = 0;

        // Iterate over all descendants to find mesh entities
        for descendant in children_query.iter_descendants(nateroid_entity) {
            if let Ok((mesh_entity, name)) = mesh_query.get(descendant) {
                // Debug: log the actual mesh name
                if let Some(name) = name {
                    debug!("Found mesh with name: '{}'", name.as_str());
                } else {
                    info!("Found mesh with no Name component");
                }

                // Match mesh name to appropriate material
                let material = if let Some(name) = name {
                    let name_str = name.as_str().to_lowercase();
                    if name_str.contains("donut") {
                        debug!("  -> Matched as donut");
                        donut_count += 1;
                        donut_material.clone()
                    } else if name_str.contains("icing") {
                        debug!("  -> Matched as icing");
                        icing_count += 1;
                        icing_material.clone()
                    } else {
                        info!("  -> Unknown mesh name, defaulting to donut material");
                        donut_count += 1;
                        donut_material.clone()
                    }
                } else {
                    info!("  -> No name, defaulting to donut material");
                    donut_count += 1;
                    donut_material.clone()
                };

                commands
                    .entity(mesh_entity)
                    .insert(MeshMaterial3d(material));
            }
        }

        debug!("Applied materials: {donut_count} donut, {icing_count} icing");
    }
}

/// Diagnostic system to check mesh entity components
fn debug_mesh_components(
    nateroid_query: Query<&Children, With<Nateroid>>,
    mesh_query: Query<
        (
            Entity,
            &Mesh3d,
            Option<&MeshMaterial3d<StandardMaterial>>,
            Option<&ViewVisibility>,
            Option<&RenderLayers>,
            Option<&Transform>,
            Option<&GlobalTransform>,
        ),
        With<Mesh3d>,
    >,
    all_children_query: Query<&Children>,
    meshes: Res<Assets<Mesh>>,
) {
    for children in nateroid_query.iter() {
        let mut to_visit: Vec<Entity> = children.iter().collect();
        let mut visited = std::collections::HashSet::new();

        while let Some(child) = to_visit.pop() {
            if !visited.insert(child) {
                continue;
            }

            if let Ok((
                entity,
                mesh3d,
                material,
                visibility,
                render_layers,
                transform,
                global_transform,
            )) = mesh_query.get(child)
            {
                // Check if the mesh asset actually has data
                let mesh_data = meshes.get(&mesh3d.0);
                let vertex_count = mesh_data.map_or(0, Mesh::count_vertices);

                debug!(
                    "Mesh entity {:?}: has_material={}, visible={:?}, vertices={}, render_layers={:?}, scale={:?}, global_pos={:?}",
                    entity,
                    material.is_some(),
                    visibility.map(|v| v.get()),
                    vertex_count,
                    render_layers,
                    transform.map(|t| t.scale),
                    global_transform.map(GlobalTransform::translation)
                );

                if vertex_count == 0 {
                    warn!("Mesh entity {:?} has ZERO vertices!", entity);
                }
            }

            if let Ok(grandchildren) = all_children_query.get(child) {
                to_visit.extend(grandchildren.iter());
            }
        }
    }
}

fn initialize_nateroid(
    nateroid: On<Add, Nateroid>,
    mut commands: Commands,
    boundary: Res<Boundary>,
    mut config: ResMut<NateroidConfig>,
    spatial_query: SpatialQuery,
    mut spawn_stats: ResMut<NateroidSpawnStats>,
    time: Res<Time>,
    test_query: Query<&Testaroid>,
) {
    // Check if this is a testaroid
    if let Ok(testaroid) = test_query.get(nateroid.entity) {
        // Testaroid: spawn with configured position and velocity
        // Dies immediately, death velocity drags portal along wall toward corner
        let scale = config.actor_config.transform.scale;
        let transform = Transform::from_translation(testaroid.position).with_scale(scale);

        commands.entity(nateroid.entity).insert((
            transform,
            LinearVelocity(testaroid.velocity),
            AngularVelocity(Vec3::ZERO),
        ));

        insert_configured_components(&mut commands, &mut config.actor_config, nateroid.entity);

        // Material will be applied by apply_nateroid_materials_to_children system

        // Kill immediately so it has approaching portal when it becomes deaderoid
        commands.entity(nateroid.entity).insert(Health(-1.0));
        return;
    }

    // Normal nateroid initialization
    let current_time = time.elapsed_secs();

    let Some(transform) = initialize_transform(&boundary, &config, &spatial_query) else {
        spawn_stats.record_attempt(false);
        commands.entity(nateroid.entity).despawn();

        // Check if we should output warning (once per second)
        if current_time - spawn_stats.last_warning_time >= 1.0 {
            let success_rate = spawn_stats.success_rate() * 100.0;
            warn!(
                "Nateroid spawn: {} / {} attempts ({:.0}%) in the last {} spawns",
                spawn_stats.successes_count(),
                spawn_stats.attempts_count(),
                success_rate,
                spawn_stats.attempts_count()
            );
            spawn_stats.last_warning_time = current_time;
        }
        return;
    };

    spawn_stats.record_attempt(true);

    // Check if we should output stats (once per second, even on success)
    if current_time - spawn_stats.last_warning_time >= 1.0 {
        let success_rate = spawn_stats.success_rate() * 100.0;
        let successes = spawn_stats.successes_count();
        let attempts = spawn_stats.attempts_count();

        // Only warn if there were failures
        if successes < attempts {
            warn!(
                "Nateroid spawn: {} / {} attempts ({:.0}%) in the last {} spawns",
                successes, attempts, success_rate, attempts
            );
        }
        spawn_stats.last_warning_time = current_time;
    }

    // Calculate random velocities for nateroid
    let (linear_velocity, angular_velocity) =
        calculate_nateroid_velocity(config.linear_velocity, config.angular_velocity);

    commands
        .entity(nateroid.entity)
        .insert(transform)
        .insert(linear_velocity)
        .insert(angular_velocity);

    insert_configured_components(&mut commands, &mut config.actor_config, nateroid.entity);

    // Material will be applied by apply_nateroid_materials_to_children system
}

fn initialize_transform(
    boundary: &Boundary,
    nateroid_config: &NateroidConfig,
    spatial_query: &SpatialQuery,
) -> Option<Transform> {
    const MAX_ATTEMPTS: u32 = 20;

    let bounds = Transform {
        translation: boundary.transform.translation,
        scale: boundary.transform.scale * SPAWN_WINDOW,
        ..default()
    };

    let scale = nateroid_config.actor_config.transform.scale;
    let filter =
        SpatialQueryFilter::from_mask(LayerMask::from([GameLayer::Spaceship, GameLayer::Asteroid]));

    for _ in 0..MAX_ATTEMPTS {
        let position = get_random_position_within_bounds(&bounds);
        let rotation = get_random_rotation();

        let intersections = spatial_query.shape_intersections(
            &nateroid_config.actor_config.collider,
            position,
            rotation,
            &filter,
        );

        if intersections.is_empty() {
            return Some(Transform::from_trs(position, rotation, scale));
        }
    }

    None
}

/// System that precomputes death materials when assets are loaded
fn precompute_death_materials(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    scenes: Res<Assets<Scene>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    nateroid_config: Res<NateroidConfig>,
) {
    // Get the nateroid scene
    let Some(nateroid_scene) = scenes.get(&scene_assets.nateroid) else {
        warn!("Nateroid scene not loaded yet");
        return;
    };

    let initial_alpha = nateroid_config.initial_alpha;
    let target_alpha = nateroid_config.target_alpha;
    // Safe: alpha values are 0.0-1.0, result is small positive integer (~30-40)
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let num_levels = ((initial_alpha - target_alpha) * 100.0) as usize + 1;

    // Collect material handles from the scene's world using try_query
    let mut material_handles = Vec::new();
    if let Some(mut query_state) = nateroid_scene
        .world
        .try_query::<&MeshMaterial3d<StandardMaterial>>()
    {
        for mesh_material in query_state.iter(&nateroid_scene.world) {
            material_handles.push(mesh_material.0.clone());
        }
    }

    if material_handles.is_empty() {
        warn!("No materials found in nateroid scene");
        return;
    }

    debug!(
        "Collected {} material handles from nateroid scene",
        material_handles.len()
    );

    // Precompute materials for each alpha level
    let mut precomputed_materials = Vec::with_capacity(num_levels);
    for level in 0..num_levels {
        // FMA optimization (faster + more precise): initial_alpha - (level as f32 * 0.01)
        let alpha = level.to_f32().mul_add(-0.01, initial_alpha);
        let mut level_materials = Vec::with_capacity(material_handles.len());

        for material_handle in &material_handles {
            if let Some(original_material) = materials.get(material_handle) {
                let mut cloned_material = original_material.clone();
                cloned_material.base_color.set_alpha(alpha);
                cloned_material.alpha_mode = AlphaMode::Blend;
                level_materials.push(materials.add(cloned_material));
            }
        }

        precomputed_materials.push(level_materials);
    }

    let num_material_sets = precomputed_materials.len();
    let num_materials_per_set = material_handles.len();

    // Insert the resource
    commands.insert_resource(NateroidDeathMaterials {
        materials: precomputed_materials,
    });

    debug!(
        "Precomputed {num_material_sets} material sets with {num_materials_per_set} materials each"
    );
}

fn get_random_position_within_bounds(bounds: &Transform) -> Vec3 {
    let mut rng = rand::rng();
    let half_scale = bounds.scale.abs() / 2.0; // Use absolute value to ensure positive scale
    let min = bounds.translation - half_scale;
    let max = bounds.translation + half_scale;

    Vec3::new(
        get_random_component(min.x, max.x, &mut rng),
        get_random_component(min.y, max.y, &mut rng),
        get_random_component(min.z, max.z, &mut rng),
    )
}

fn get_random_component(min: f32, max: f32, rng: &mut impl Rng) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        min // If the range is effectively zero, just return the min value
    } else {
        rng.random_range(min.min(max)..=min.max(max)) // Ensure min is always less than max
    }
}

fn get_random_rotation() -> Quat {
    let mut rng = rand::rng();
    Quat::from_euler(
        EulerRot::XYZ,
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
        rng.random_range(-std::f32::consts::PI..std::f32::consts::PI),
    )
}

fn random_vec3(range_x: Range<f32>, range_y: Range<f32>, range_z: Range<f32>) -> Vec3 {
    let mut rng = rand::rng();
    let x = if range_x.start < range_x.end {
        rng.random_range(range_x)
    } else {
        0.0
    };
    let y = if range_y.start < range_y.end {
        rng.random_range(range_y)
    } else {
        0.0
    };
    let z = if range_z.start < range_z.end {
        rng.random_range(range_z)
    } else {
        0.0
    };

    Vec3::new(x, y, z)
}

fn calculate_nateroid_velocity(linvel: f32, angvel: f32) -> (LinearVelocity, AngularVelocity) {
    (
        LinearVelocity(random_vec3(-linvel..linvel, -linvel..linvel, 0.0..0.0)),
        AngularVelocity(random_vec3(
            -angvel..angvel,
            -angvel..angvel,
            -angvel..angvel,
        )),
    )
}
