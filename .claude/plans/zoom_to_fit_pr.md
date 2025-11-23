# ZoomToFit Feature for bevy_panorbit_camera

## Vision
Transform the current game-specific zoom-to-fit capability into a generic, reusable feature for `bevy_panorbit_camera` that can be contributed as a PR.

## Core Design Principles

### 1. **Simplicity**: Inserting a marker component is all that's needed
```rust
// User code - that's it!
commands.entity(asteroid).insert(ZoomToFit);
```

### 2. **Observer-Driven**: No manual system invocation required
- `OnAdd` observer detects when `ZoomToFit` added to target entity
- Automatically inserts internal state on camera
- Update system runs while state exists
- Cleanup is automatic

### 3. **Generic**: Works with any entity
- Get bounds from entity's components: `CustomBounds` > `Aabb` > `Transform`
- Center calculated from actual bounds (no hardcoded `Vec3::ZERO`)
- No game-specific types (`Boundary` resource, `GameAction`, etc.)
- Pure geometric operations

### 4. **Feature-Gated**: Optional dependency
```toml
bevy_panorbit_camera = { version = "0.x", features = ["zoom_to_fit"] }
```

## API Design

### User-Facing Component (Simple Marker)
```rust
/// Mark an entity to be framed by the camera.
///
/// When inserted on an entity, the camera will automatically zoom to fit it.
/// The entity should have one of:
/// - `CustomBounds` component (custom world-space bounds)
/// - `Aabb` component (axis-aligned bounding box)
/// - `Transform` component (treated as point target)
///
/// # Example
/// ```rust
/// // Zoom to an asteroid with AABB
/// commands.entity(asteroid).insert(ZoomToFit);
///
/// // Zoom to a custom region
/// commands.spawn((
///     Transform::default(),
///     CustomBounds(vec![
///         Vec3::new(-10.0, -10.0, -10.0),
///         Vec3::new(10.0, 10.0, 10.0),
///     ]),
///     ZoomToFit,
/// ));
/// ```
#[derive(Component)]
pub struct ZoomToFit;
```

### Custom Bounds Component (Optional)
```rust
/// Provides custom world-space bounds for zoom-to-fit.
///
/// Use this when you want to specify exact bounds instead of using `Aabb`.
/// Useful for non-entity regions like game boundaries, grids, etc.
#[derive(Component)]
pub struct CustomBounds(pub Vec<Vec3>);
```

### Internal State Component (Auto-Managed)
```rust
/// Internal state for active zoom animation.
///
/// This is automatically added to the camera by the `OnAdd<ZoomToFit>` observer
/// and removed when convergence completes. Users should not interact with this directly.
#[derive(Component)]
struct ZoomToFitState {
    /// Which entity we're zooming to
    target_entity: Entity,

    /// Current iteration count
    iteration_count: usize,

    /// Saved smoothness values for restoration
    saved_zoom_smoothness: f32,
    saved_pan_smoothness: f32,
}
```

### Configuration Resource
```rust
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct ZoomToFitConfig {
    /// Margin around bounds (0.0 = tight fit, 0.2 = 20% margin)
    pub margin: f32,

    /// Tolerance for convergence (0.001 = 0.1% tolerance)
    pub margin_tolerance: f32,

    /// How fast to converge (0.18 = 18% adjustment per frame)
    pub convergence_rate: f32,

    /// Safety limit
    pub max_iterations: usize,
}

impl Default for ZoomToFitConfig {
    fn default() -> Self {
        Self {
            margin: 0.15,
            margin_tolerance: 0.002,
            convergence_rate: 0.18,
            max_iterations: 200,
        }
    }
}
```

### Debug Visualization
```rust
/// Add to camera to visualize zoom-to-fit process.
///
/// Shows target bounds, screen margins, and convergence status.
#[derive(Component, Default)]
pub struct ZoomToFitDebug {
    pub show_bounds: bool,        // Green box for target
    pub show_margins: bool,        // Yellow box for margins
    pub show_screen_edges: bool,   // Red lines for screen
    pub show_status: bool,         // Text overlay
}
```

### Plugin
```rust
pub struct ZoomToFitPlugin;

impl Plugin for ZoomToFitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomToFitConfig>()
            .register_type::<ZoomToFitConfig>()
            .add_observer(on_add_zoom_to_fit)
            .add_systems(Update, update_zoom_to_fit)
            .add_systems(Update, draw_zoom_to_fit_debug.run_if(
                any_with_component::<ZoomToFitDebug>
            ));
    }
}
```

## How It Works

### Flow Diagram
```
1. User: commands.entity(asteroid).insert(ZoomToFit)
                    ↓
2. Observer: On<Add, ZoomToFit> fires
                    ↓
3. Observer: Find camera with PanOrbitCamera
                    ↓
4. Observer: Save camera smoothness, disable it
                    ↓
5. Observer: Insert ZoomToFitState on camera (points to asteroid)
                    ↓
6. Update System: Queries cameras with ZoomToFitState
                    ↓
7. Update System: Get target entity's bounds (CustomBounds/Aabb/Transform)
                    ↓
8. Update System: Calculate center from actual bounds (NOT Vec3::ZERO!)
                    ↓
9. Update System: Run convergence algorithm (adjust focus & radius)
                    ↓
10. Update System: Check if converged (balanced & fitted)
                    ↓
11. Update System: If yes, remove ZoomToFitState from camera
                    ↓
12. Update System: Restore camera smoothness on removal
                    ↓
13. Done! Target is now framed
```

### Multiple Targets
If `ZoomToFit` is inserted on a new entity while already zooming:
- **Last one wins**: Observer replaces existing `ZoomToFitState` with new target
- Previous zoom animation stops immediately
- New zoom starts fresh

### Bounds Priority
When getting bounds from target entity:
1. **First**: Check for `CustomBounds` component (explicit bounds)
2. **Second**: Check for `Aabb` component (mesh/collider bounds)
3. **Third**: Use `Transform.translation` (point target)

### Center Calculation (Fixes Vec3::ZERO Issue)
```rust
// OLD (hardcoded):
let boundary_center = Vec3::ZERO;

// NEW (calculated from actual bounds):
let boundary_center = bounds.iter().sum::<Vec3>() / bounds.len() as f32;
```

## Implementation Plan

### Phase 1: Refactor Current Code (In nateroids)
**Goal**: Isolate zoom logic into feature-gated module that mimics bevy_panorbit_camera structure

1. **Create new module structure**
   ```
   src/camera/zoom_to_fit/
   ├── mod.rs              # Plugin, public API
   ├── component.rs        # ZoomToFit, CustomBounds components
   ├── config.rs           # ZoomToFitConfig resource
   ├── state.rs            # ZoomToFitState (internal)
   ├── observer.rs         # on_add_zoom_to_fit
   ├── system.rs           # update_zoom_to_fit
   ├── geometry.rs         # ScreenSpaceBoundary, projections
   └── debug.rs            # Gizmo visualization
   ```

2. **Feature gate in Cargo.toml**
   ```toml
   [features]
   default = ["zoom_to_fit"]
   zoom_to_fit = []
   ```

3. **Extract current zoom.rs → zoom_to_fit module**
   - Move all zoom logic
   - Keep same behavior initially
   - Ensure tests still pass

### Phase 2: Make It Generic
**Goal**: Remove all game-specific dependencies and implement target-based design

#### 2.1: Replace Boundary Resource with Entity
**Current problem**: `Boundary` is a resource (game-specific)

**Solution**: Make it an entity with `Transform` + `CustomBounds`

```rust
// In nateroids startup:
fn spawn_boundary(mut commands: Commands) {
    let corners = calculate_boundary_corners();

    commands.spawn((
        Transform::default(),
        CustomBounds(corners),
        // Don't add ZoomToFit here - add it when user presses Z
    ));
}

// In input handling:
fn handle_zoom_input(
    mut commands: Commands,
    input: Res<ActionState<GameAction>>,
    boundary_query: Query<Entity, With<CustomBounds>>,
) {
    if input.just_pressed(&GameAction::ZoomToFit) {
        if let Ok(boundary_entity) = boundary_query.single() {
            commands.entity(boundary_entity).insert(ZoomToFit);
        }
    }
}
```

#### 2.2: Implement Observer-Based Initialization

**Replace `start_zoom_to_fit` system with observer:**

```rust
fn on_add_zoom_to_fit(
    trigger: On<Add, ZoomToFit>,
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut PanOrbitCamera, Option<&ZoomToFitState>)>,
) {
    let target_entity = trigger.entity;

    // Find the camera (TODO: support multiple cameras with camera selection)
    let Ok((camera_entity, mut pan_orbit, existing_state)) = cameras.single_mut() else {
        warn!("No PanOrbitCamera found for ZoomToFit");
        return;
    };

    // If already zooming to something else, this replaces it (last wins)
    if let Some(_existing) = existing_state {
        debug!("Replacing existing zoom target");
        commands.entity(camera_entity).remove::<ZoomToFitState>();
    }

    // Save current smoothness values
    let saved_zoom = pan_orbit.zoom_smoothness;
    let saved_pan = pan_orbit.pan_smoothness;

    // Disable smoothness for immediate response
    pan_orbit.zoom_smoothness = 0.0;
    pan_orbit.pan_smoothness = 0.0;

    // Insert state on camera
    commands.entity(camera_entity).insert(ZoomToFitState {
        target_entity,
        iteration_count: 0,
        saved_zoom_smoothness: saved_zoom,
        saved_pan_smoothness: saved_pan,
    });

    debug!("Starting zoom to entity {:?}", target_entity);
}
```

#### 2.3: Update System to Query State and Target

**Replace camera-centric query with state-based query:**

```rust
fn update_zoom_to_fit(
    mut commands: Commands,
    config: Res<ZoomToFitConfig>,

    // Cameras with active zoom state
    mut cameras: Query<(
        Entity,
        &GlobalTransform,
        &mut PanOrbitCamera,
        &Projection,
        &Camera,
        &mut ZoomToFitState,
    )>,

    // Target entities with bounds
    targets: Query<(
        &Transform,
        Option<&CustomBounds>,
        Option<&Aabb>,
    )>,
) {
    for (cam_entity, cam_global, mut pan_orbit, proj, cam, mut state) in &mut cameras {
        // Get target entity's bounds
        let Ok((transform, custom_bounds, aabb)) = targets.get(state.target_entity) else {
            // Target no longer exists or missing ZoomToFit
            debug!("Target entity no longer valid, stopping zoom");
            commands.entity(cam_entity).remove::<ZoomToFitState>();
            continue;
        };

        // Get bounds with priority: CustomBounds > Aabb > Transform
        let bounds = if let Some(custom) = custom_bounds {
            custom.0.clone()
        } else if let Some(aabb) = aabb {
            aabb_to_world_corners(aabb, transform)
        } else {
            // Point target
            vec![transform.translation]
        };

        // Calculate center from ACTUAL bounds (NOT Vec3::ZERO!)
        let boundary_center = if bounds.is_empty() {
            warn!("No bounds for target entity");
            continue;
        } else {
            bounds.iter().sum::<Vec3>() / bounds.len() as f32
        };

        // Calculate screen-space projections
        let Projection::Perspective(perspective) = proj else {
            continue;
        };

        let aspect_ratio = if let Some(viewport_size) = cam.logical_viewport_size() {
            viewport_size.x / viewport_size.y
        } else {
            perspective.aspect_ratio
        };

        let Some(margins) = ScreenSpaceBoundary::from_world_points(
            &bounds,
            cam_global,
            perspective,
            aspect_ratio,
            config.margin,
        ) else {
            // Boundary behind camera, move camera back
            debug!(
                "Iteration {}: Boundary behind camera, moving back",
                state.iteration_count
            );
            pan_orbit.target_focus = boundary_center;
            pan_orbit.target_radius *= 1.5;
            pan_orbit.force_update = true;
            state.iteration_count += 1;
            continue;
        };

        // Run convergence algorithm (mostly unchanged from current code)
        let current_radius = pan_orbit.target_radius;

        // Use boundary_center instead of Vec3::ZERO in phase 1!
        let target_focus = calculate_target_focus(
            pan_orbit.target_focus,
            current_radius,
            &margins,
            cam_global,
            boundary_center,  // <-- Pass actual center
        );

        let target_radius = calculate_target_radius(
            current_radius,
            margins.span(),
            &margins,
            &config,
        );

        // Apply convergence
        let focus_delta = target_focus - pan_orbit.target_focus;
        let radius_delta = target_radius - current_radius;
        let rate = config.convergence_rate;

        pan_orbit.target_focus += focus_delta * rate;
        pan_orbit.target_radius = current_radius + radius_delta * rate;
        pan_orbit.force_update = true;

        state.iteration_count += 1;

        // Check convergence
        let balanced = margins.is_balanced(config.margin_tolerance);
        let fitted = margins.is_fitted(config.margin_tolerance);

        if balanced && fitted {
            debug!("Converged after {} iterations", state.iteration_count);

            // Restore smoothness before removing state
            pan_orbit.zoom_smoothness = state.saved_zoom_smoothness;
            pan_orbit.pan_smoothness = state.saved_pan_smoothness;

            commands.entity(cam_entity).remove::<ZoomToFitState>();
            continue;
        }

        // Safety limit
        if state.iteration_count >= config.max_iterations {
            warn!("Max iterations reached without convergence");

            pan_orbit.zoom_smoothness = state.saved_zoom_smoothness;
            pan_orbit.pan_smoothness = state.saved_pan_smoothness;

            commands.entity(cam_entity).remove::<ZoomToFitState>();
        }
    }
}
```

#### 2.4: Fix Vec3::ZERO in calculate_target_focus

**Update signature and implementation:**

```rust
fn calculate_target_focus(
    current_focus: Vec3,
    current_radius: f32,
    margins: &ScreenSpaceBoundary,
    cam_global: &GlobalTransform,
    boundary_center: Vec3,  // <-- NEW: pass actual center
) -> Vec3 {
    let focus_to_boundary_distance = (current_focus - boundary_center).length();
    let far_from_boundary_threshold = current_radius * 0.5;

    if focus_to_boundary_distance > far_from_boundary_threshold {
        // Phase 1: Move toward actual boundary center (NOT Vec3::ZERO!)
        boundary_center
    } else {
        // Phase 2: Fine-tune using screen-space centering
        let (center_x, center_y) = margins.center();
        let cam_rot = cam_global.rotation();
        let cam_right = cam_rot * Vec3::X;
        let cam_up = cam_rot * Vec3::Y;

        let world_offset_x = center_x * margins.avg_depth;
        let world_offset_y = center_y * margins.avg_depth;
        let focus_correction = cam_right * world_offset_x + cam_up * world_offset_y;

        current_focus + focus_correction
    }
}
```

#### 2.5: Update ScreenSpaceBoundary

**Rename method and make it work with arbitrary points:**

```rust
impl ScreenSpaceBoundary {
    // OLD:
    // pub fn from_camera_view(boundary: &Boundary, ...) -> Option<Self>

    // NEW:
    pub fn from_world_points(
        world_points: &[Vec3],
        cam_global: &GlobalTransform,
        perspective: &PerspectiveProjection,
        aspect_ratio: f32,
        margin_multiplier: f32,
    ) -> Option<Self> {
        // Same logic, but takes points directly instead of Boundary resource
        ...
    }
}
```

#### 2.6: Remove GameAction Dependency

**Before:**
```rust
.add_systems(Update, start_zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)))
```

**After:**
```rust
// Input handling is now game-specific, not part of zoom_to_fit module
fn handle_zoom_input(
    mut commands: Commands,
    input: Res<ActionState<GameAction>>,
    boundary: Query<Entity, With<CustomBounds>>,
) {
    if input.just_pressed(&GameAction::ZoomToFit) {
        if let Ok(entity) = boundary.single() {
            commands.entity(entity).insert(ZoomToFit);
        }
    }
}
```

### Phase 3: Debug Visualization
**Goal**: Visual feedback for development and debugging

1. **Add debug component to camera (not target)**
   ```rust
   // Enable debug visualization
   commands.entity(camera).insert(ZoomToFitDebug {
       show_bounds: true,
       show_margins: true,
       show_screen_edges: true,
       show_status: true,
   });
   ```

2. **Implement gizmo system**
   ```rust
   fn draw_zoom_to_fit_debug(
       mut gizmos: Gizmos,
       cameras: Query<(
           &GlobalTransform,
           &Projection,
           &Camera,
           &ZoomToFitDebug,
           &ZoomToFitState,
       ), With<PanOrbitCamera>>,
       targets: Query<(&Transform, Option<&CustomBounds>, Option<&Aabb>)>,
       config: Res<ZoomToFitConfig>,
   ) {
       for (cam_global, proj, cam, debug, state) in &cameras {
           let Ok((transform, custom_bounds, aabb)) = targets.get(state.target_entity) else {
               continue;
           };

           // Get bounds
           let bounds = if let Some(custom) = custom_bounds {
               custom.0.clone()
           } else if let Some(aabb) = aabb {
               aabb_to_world_corners(aabb, transform)
           } else {
               vec![transform.translation]
           };

           if debug.show_bounds {
               // Draw green wireframe box around target
               draw_bounds_box(&mut gizmos, &bounds, Color::GREEN);
           }

           if debug.show_margins {
               // Calculate and draw margin boundaries
               let Projection::Perspective(perspective) = proj else { continue };
               let aspect_ratio = cam.logical_viewport_size()
                   .map(|s| s.x / s.y)
                   .unwrap_or(perspective.aspect_ratio);

               if let Some(margins) = ScreenSpaceBoundary::from_world_points(
                   &bounds,
                   cam_global,
                   perspective,
                   aspect_ratio,
                   config.margin,
               ) {
                   draw_margin_box(&mut gizmos, &margins, cam_global, Color::YELLOW);
               }
           }

           if debug.show_screen_edges {
               draw_screen_edges(&mut gizmos, cam_global, proj, cam);
           }

           if debug.show_status {
               // Show iteration count, convergence status
               gizmos.text(
                   transform.translation + Vec3::Y * 5.0,
                   format!("Iteration: {}", state.iteration_count),
                   Color::WHITE,
               );
           }
       }
   }
   ```

3. **Game integration**
   ```rust
   // Toggle debug view with 'B' key
   fn toggle_zoom_debug(
       mut commands: Commands,
       input: Res<ActionState<GameAction>>,
       cameras: Query<(Entity, Option<&ZoomToFitDebug>), With<PanOrbitCamera>>,
   ) {
       if input.just_pressed(&GameAction::BoundaryBox) {
           if let Ok((camera, debug)) = cameras.single() {
               if debug.is_some() {
                   commands.entity(camera).remove::<ZoomToFitDebug>();
               } else {
                   commands.entity(camera).insert(ZoomToFitDebug {
                       show_bounds: true,
                       show_margins: true,
                       show_screen_edges: false,
                       show_status: true,
                   });
               }
           }
       }
   }
   ```

### Phase 4: Example Implementation
**Goal**: Demonstrate feature in standalone example

**Create `examples/zoom_to_fit.rs`:**

```rust
//! Demonstrates the zoom-to-fit feature.
//!
//! Controls:
//! - 1-3: Zoom to different objects
//! - 4: Zoom to all objects (via bounding entity)
//! - D: Toggle debug visualization
//! - Space: Stop zoom

use bevy::prelude::*;
use bevy_panorbit_camera::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(ZoomToFitPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

#[derive(Component)]
struct TargetObject(&'static str);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            radius: 50.0,
            ..default()
        },
    ));

    // Target 1: Small cube at origin
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(2.0, 2.0, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_translation(Vec3::ZERO),
        // Aabb auto-computed from mesh
        TargetObject("Cube"),
    ));

    // Target 2: Large sphere offset
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(8.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
        Transform::from_translation(Vec3::new(20.0, 10.0, -15.0)),
        TargetObject("Sphere"),
    ));

    // Target 3: Wide rectangle
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(60.0, 4.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.8))),
        Transform::from_translation(Vec3::new(0.0, -5.0, 0.0)),
        TargetObject("Rectangle"),
    ));

    // Target 4: Invisible entity with CustomBounds encompassing all objects
    let all_bounds = vec![
        Vec3::new(-30.0, -7.0, -25.0),
        Vec3::new(28.0, 18.0, 5.0),
    ];
    commands.spawn((
        Transform::default(),
        CustomBounds(all_bounds),
        TargetObject("All Objects"),
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_rotation_x(-0.5)),
    ));

    info!("Press 1-3 to zoom to objects, 4 for all, D for debug, Space to stop");
}

fn handle_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    camera: Query<Entity, With<PanOrbitCamera>>,
    targets: Query<(Entity, &TargetObject)>,
    current_zoom: Query<Entity, With<ZoomToFit>>,
) {
    let Ok(cam_entity) = camera.single() else { return };

    // Remove existing ZoomToFit markers
    for entity in &current_zoom {
        commands.entity(entity).remove::<ZoomToFit>();
    }

    // Zoom to specific target
    if keys.just_pressed(KeyCode::Digit1) {
        if let Some((entity, _)) = targets.iter().find(|(_, t)| t.0 == "Cube") {
            commands.entity(entity).insert(ZoomToFit);
            info!("Zooming to Cube");
        }
    }

    if keys.just_pressed(KeyCode::Digit2) {
        if let Some((entity, _)) = targets.iter().find(|(_, t)| t.0 == "Sphere") {
            commands.entity(entity).insert(ZoomToFit);
            info!("Zooming to Sphere");
        }
    }

    if keys.just_pressed(KeyCode::Digit3) {
        if let Some((entity, _)) = targets.iter().find(|(_, t)| t.0 == "Rectangle") {
            commands.entity(entity).insert(ZoomToFit);
            info!("Zooming to Rectangle");
        }
    }

    if keys.just_pressed(KeyCode::Digit4) {
        if let Some((entity, _)) = targets.iter().find(|(_, t)| t.0 == "All Objects") {
            commands.entity(entity).insert(ZoomToFit);
            info!("Zooming to All Objects");
        }
    }

    // Toggle debug
    if keys.just_pressed(KeyCode::KeyD) {
        if let Ok(cam) = camera.single() {
            commands.entity(cam).insert(ZoomToFitDebug::default());
        }
    }

    // Stop zoom
    if keys.just_pressed(KeyCode::Space) {
        for entity in &current_zoom {
            commands.entity(entity).remove::<ZoomToFit>();
        }
        info!("Zoom stopped");
    }
}
```

### Phase 5: Adapt nateroids Game
**Goal**: Migrate existing game to use new generic API

1. **Convert Boundary to entity**
   ```rust
   // OLD: Boundary resource
   commands.insert_resource(Boundary { corners: [...] });

   // NEW: Boundary entity
   commands.spawn((
       Transform::default(),
       CustomBounds(boundary_corners),
       BoundaryMarker, // Optional game-specific marker
   ));
   ```

2. **Update input handling**
   ```rust
   // OLD:
   .add_systems(Update, start_zoom_to_fit.run_if(just_pressed(GameAction::ZoomToFit)))

   // NEW:
   fn handle_zoom_to_fit_input(
       mut commands: Commands,
       input: Res<ActionState<GameAction>>,
       boundary: Query<Entity, With<BoundaryMarker>>,
   ) {
       if input.just_pressed(&GameAction::ZoomToFit) {
           if let Ok(boundary_entity) = boundary.single() {
               commands.entity(boundary_entity).insert(ZoomToFit);
           }
       }
   }
   ```

3. **Remove old zoom.rs module**
   - Delete `src/camera/zoom.rs`
   - Update `src/camera/mod.rs`

4. **Verify identical behavior**
   - Press Z to zoom
   - Check convergence works the same
   - Verify debug visualization

### Phase 6: Documentation
**Goal**: Comprehensive docs for users and contributors

1. **API Documentation**
   ```rust
   /// Automatically animates a [`PanOrbitCamera`] to frame a target entity.
   ///
   /// Insert this component on any entity to make the camera zoom to fit it.
   /// The entity should have one of:
   /// - [`CustomBounds`]: Explicit world-space bounds
   /// - [`Aabb`]: Axis-aligned bounding box (from mesh)
   /// - [`Transform`]: Treated as point target
   ///
   /// # Example
   /// ```rust
   /// // Zoom to an asteroid
   /// commands.entity(asteroid).insert(ZoomToFit);
   ///
   /// // Zoom to a custom region
   /// commands.spawn((
   ///     Transform::default(),
   ///     CustomBounds(my_bounds),
   ///     ZoomToFit,
   /// ));
   /// ```
   ///
   /// The component is automatically removed when the zoom completes.
   #[derive(Component)]
   pub struct ZoomToFit;
   ```

2. **Module-level docs**
   ```rust
   //! # Zoom-to-Fit Feature
   //!
   //! Provides automatic camera framing for [`PanOrbitCamera`].
   //!
   //! ## Quick Start
   //!
   //! ```rust
   //! use bevy::prelude::*;
   //! use bevy_panorbit_camera::prelude::*;
   //!
   //! fn setup(mut commands: Commands) {
   //!     // Spawn camera
   //!     commands.spawn((
   //!         Camera3d::default(),
   //!         PanOrbitCamera::default(),
   //!     ));
   //!
   //!     // Spawn target with AABB
   //!     let target = commands.spawn((
   //!         Mesh3d(...),
   //!         Transform::default(),
   //!         // Aabb computed automatically
   //!     )).id();
   //!
   //!     // Zoom to it!
   //!     commands.entity(target).insert(ZoomToFit);
   //! }
   //! ```
   //!
   //! ## How It Works
   //!
   //! 1. Insert `ZoomToFit` on target entity
   //! 2. Observer detects addition, inserts state on camera
   //! 3. System iteratively adjusts camera focus and radius
   //! 4. Converges when target is centered with proper margins
   //! 5. Automatically cleans up when done
   //!
   //! ## Configuration
   //!
   //! Adjust convergence behavior via [`ZoomToFitConfig`] resource:
   //!
   //! ```rust
   //! app.insert_resource(ZoomToFitConfig {
   //!     margin: 0.2,              // 20% margin around target
   //!     convergence_rate: 0.25,   // 25% adjustment per frame
   //!     ..default()
   //! });
   //! ```
   //!
   //! ## Debug Visualization
   //!
   //! Add [`ZoomToFitDebug`] to camera to see the zoom process:
   //!
   //! ```rust
   //! commands.entity(camera).insert(ZoomToFitDebug::default());
   //! ```
   ```

3. **README section**
   ```markdown
   ## Zoom-to-Fit (Optional Feature)

   Enable with `features = ["zoom_to_fit"]` in your `Cargo.toml`.

   Automatically animate the camera to frame a target entity:

   ```rust
   commands.entity(my_object).insert(ZoomToFit);
   ```

   See `examples/zoom_to_fit.rs` for a complete example.
   ```

### Phase 7: Prepare for PR
**Goal**: Extract code ready to copy into bevy_panorbit_camera fork

1. **Module is already isolated** in `src/camera/zoom_to_fit/`

2. **Verify zero game dependencies**
   - No `Boundary` resource
   - No `GameAction` enum
   - Only `bevy` + `bevy_panorbit_camera` types

3. **Extraction script**
   ```bash
   #!/bin/bash
   # Copy zoom_to_fit module to bevy_panorbit_camera fork
   FORK_DIR="../bevy_panorbit_camera_fork"

   cp -r src/camera/zoom_to_fit "$FORK_DIR/src/"
   cp examples/zoom_to_fit.rs "$FORK_DIR/examples/"

   echo "Extracted zoom_to_fit feature to $FORK_DIR"
   echo "Next steps:"
   echo "1. Update Cargo.toml with zoom_to_fit feature"
   echo "2. Add feature gate to src/lib.rs"
   echo "3. Test example: cargo run --example zoom_to_fit --features zoom_to_fit"
   ```

4. **PR Checklist**
   - [ ] Feature behind `zoom_to_fit` flag
   - [ ] All public APIs documented
   - [ ] Example demonstrates all capabilities
   - [ ] **Both Perspective and Orthographic projection support**
   - [ ] No game-specific code
   - [ ] No new dependencies
   - [ ] CI passes (test, clippy, fmt)
   - [ ] Follows bevy_panorbit_camera style

5. **Draft PR Description**
   ```markdown
   # Add Zoom-to-Fit Feature

   ## Overview

   Adds optional `zoom_to_fit` feature that provides automatic camera framing.
   Simply insert a marker component on any entity to zoom to it.

   ## Usage

   ```rust
   // Mark entity to frame
   commands.entity(my_object).insert(ZoomToFit);
   ```

   The camera automatically:
   - Detects the target via observer
   - Calculates bounds from `CustomBounds`, `Aabb`, or `Transform`
   - Iteratively adjusts to frame it with configurable margins
   - Cleans up when done

   ## Key Features

   - **Simple API**: Just insert a component
   - **Observer-driven**: Fully automatic lifecycle
   - **Generic**: Works with any entity with bounds
   - **Full projection support**: Works with both Perspective and Orthographic cameras
   - **Configurable**: Adjust margins, convergence rate, etc.
   - **Debug visualization**: Optional gizmos for development

   ## Example

   See `examples/zoom_to_fit.rs` for complete demonstration.

   ## Implementation Notes

   - Iterative convergence (robust to edge cases)
   - Temporarily disables camera smoothing
   - Restores settings on completion
   - No new dependencies

   ## Testing

   Extensively tested in [nateroids game](link) with various target types.
   ```

### Phase 8: Orthographic Projection Support
**Goal**: Implement zoom-to-fit for orthographic cameras to support all bevy_panorbit_camera projection types

Since bevy_panorbit_camera supports both Perspective and Orthographic projections, zoom-to-fit must work with both for a complete feature.

#### Key Differences

**Perspective (current implementation):**
- Objects get smaller/larger with distance (perspective division)
- Zoom adjusts camera radius (distance from focus point)
- Uses FOV calculations to determine required distance
- Formula: `distance = size / tan(fov/2)`

**Orthographic (new):**
- Objects stay same size regardless of distance (no perspective division)
- Zoom adjusts orthographic scale factor
- Uses viewport dimensions to determine required scale
- Formula: `scale = viewport_size / target_size`

#### Implementation Strategy

**1. Update projection handling in `update_zoom_to_fit`:**

```rust
fn update_zoom_to_fit(
    mut commands: Commands,
    config: Res<ZoomToFitConfig>,
    mut cameras: Query<(
        Entity,
        &GlobalTransform,
        &mut PanOrbitCamera,
        &Projection,
        &Camera,
        &mut ZoomToFitState,
    )>,
    targets: Query<(
        &Transform,
        Option<&CustomBounds>,
        Option<&Aabb>,
    )>,
) {
    for (cam_entity, cam_global, mut pan_orbit, projection, cam, mut state) in &mut cameras {
        // Get target bounds (same for both projection types)
        let Ok((transform, custom_bounds, aabb)) = targets.get(state.target_entity) else {
            commands.entity(cam_entity).remove::<ZoomToFitState>();
            continue;
        };

        let bounds = get_bounds(custom_bounds, aabb, transform);
        let boundary_center = calculate_center(&bounds);

        // Branch based on projection type
        match projection {
            Projection::Perspective(perspective) => {
                update_perspective_zoom(
                    &mut pan_orbit,
                    &bounds,
                    boundary_center,
                    cam_global,
                    perspective,
                    cam,
                    &config,
                    &mut state,
                );
            }
            Projection::Orthographic(orthographic) => {
                update_orthographic_zoom(
                    &mut pan_orbit,
                    &bounds,
                    boundary_center,
                    cam_global,
                    orthographic,
                    cam,
                    &config,
                    &mut state,
                );
            }
        }

        // Check convergence (same for both)
        if is_converged(&state, &config) {
            pan_orbit.zoom_smoothness = state.saved_zoom_smoothness;
            pan_orbit.pan_smoothness = state.saved_pan_smoothness;
            commands.entity(cam_entity).remove::<ZoomToFitState>();
        }
    }
}
```

**2. Implement `update_orthographic_zoom` function:**

```rust
fn update_orthographic_zoom(
    pan_orbit: &mut PanOrbitCamera,
    bounds: &[Vec3],
    boundary_center: Vec3,
    cam_global: &GlobalTransform,
    orthographic: &OrthographicProjection,
    camera: &Camera,
    config: &ZoomToFitConfig,
    state: &mut ZoomToFitState,
) {
    // Step 1: Adjust focus to center target (same concept as perspective)
    let focus_to_boundary_distance = (pan_orbit.target_focus - boundary_center).length();
    let far_from_boundary_threshold = pan_orbit.target_radius * 0.5;

    if focus_to_boundary_distance > far_from_boundary_threshold {
        // Phase 1: Move toward boundary center
        let focus_delta = boundary_center - pan_orbit.target_focus;
        pan_orbit.target_focus += focus_delta * config.convergence_rate;
        pan_orbit.force_update = true;
        state.iteration_count += 1;
        return;
    }

    // Step 2: Calculate required orthographic scale
    // Project bounds to screen space
    let view_proj = orthographic.get_clip_from_view()
        * cam_global.compute_matrix().inverse();

    let mut screen_bounds = ScreenBounds::new();
    for &world_pos in bounds {
        let clip_pos = view_proj.project_point3(world_pos);

        // Check if behind camera
        if clip_pos.z < 0.0 {
            // Move camera back
            pan_orbit.target_radius *= 1.5;
            pan_orbit.force_update = true;
            state.iteration_count += 1;
            return;
        }

        screen_bounds.extend(clip_pos.x, clip_pos.y);
    }

    // Step 3: Calculate target scale with margins
    let viewport_size = camera.logical_viewport_size()
        .unwrap_or(Vec2::new(1920.0, 1080.0));

    let aspect_ratio = viewport_size.x / viewport_size.y;

    // NDC coordinates range from -1 to 1 (total size = 2)
    let current_width_ndc = screen_bounds.width();
    let current_height_ndc = screen_bounds.height();

    // Add margin (e.g., 0.15 means target should take up 85% of viewport)
    let target_width_ndc = 2.0 * (1.0 - config.margin);
    let target_height_ndc = 2.0 * (1.0 - config.margin);

    // Scale factor needed to fit width and height
    let scale_for_width = current_width_ndc / target_width_ndc;
    let scale_for_height = current_height_ndc / target_height_ndc;

    // Use the larger scale to ensure both dimensions fit
    let target_scale = scale_for_width.max(scale_for_height);

    // Step 4: Apply convergence to PanOrbitCamera
    // Note: PanOrbitCamera might control orthographic scale via zoom_smoothness
    // or we might need to modify the Projection directly
    // This depends on how bevy_panorbit_camera implements orthographic zoom

    // Approach 1: If PanOrbitCamera has orthographic_scale field
    if let Some(current_scale) = pan_orbit.orthographic_scale {
        let scale_delta = target_scale - current_scale;
        pan_orbit.orthographic_scale = Some(current_scale + scale_delta * config.convergence_rate);
    }

    // Approach 2: If we need to modify projection directly (requires &mut Projection)
    // orthographic.scale = target_scale;

    pan_orbit.force_update = true;
    state.iteration_count += 1;

    // Step 5: Fine-tune focus based on screen-space centering
    let center_x = (screen_bounds.min_x + screen_bounds.max_x) / 2.0;
    let center_y = (screen_bounds.min_y + screen_bounds.max_y) / 2.0;

    if center_x.abs() > config.margin_tolerance || center_y.abs() > config.margin_tolerance {
        // Convert screen-space offset to world-space
        let cam_right = cam_global.right();
        let cam_up = cam_global.up();

        // Scale offset by orthographic scale
        let world_offset_x = center_x * orthographic.scale * aspect_ratio;
        let world_offset_y = center_y * orthographic.scale;

        let focus_correction = cam_right * world_offset_x + cam_up * world_offset_y;
        pan_orbit.target_focus += focus_correction * config.convergence_rate;
    }
}

struct ScreenBounds {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl ScreenBounds {
    fn new() -> Self {
        Self {
            min_x: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            min_y: f32::INFINITY,
            max_y: f32::NEG_INFINITY,
        }
    }

    fn extend(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
    }

    fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    fn height(&self) -> f32 {
        self.max_y - self.min_y
    }
}
```

**3. Investigation needed:**

Before implementing, check how `bevy_panorbit_camera` handles orthographic zoom:
- Does `PanOrbitCamera` have an `orthographic_scale` field?
- Or does it modify `Projection::Orthographic.scale` directly?
- How does mouse wheel zoom work for orthographic cameras?

Look at: `bevy_panorbit_camera` source code, specifically:
- `PanOrbitCamera` struct definition
- Zoom input handling for orthographic projection
- Any existing orthographic-specific fields or methods

**4. Update example to demonstrate both:**

Add to `examples/zoom_to_fit.rs`:
```rust
// Add key binding to toggle projection
if keys.just_pressed(KeyCode::KeyP) {
    if let Ok((entity, projection)) = cameras.single_mut() {
        let new_projection = match projection {
            Projection::Perspective(_) => {
                Projection::Orthographic(OrthographicProjection {
                    scale: 50.0,
                    ..default()
                })
            }
            Projection::Orthographic(_) => {
                Projection::Perspective(PerspectiveProjection {
                    fov: 0.785,  // 45 degrees
                    ..default()
                })
            }
        };

        commands.entity(entity).insert(new_projection);
        info!("Toggled projection mode");
    }
}
```

**5. Testing checklist:**

- [ ] Orthographic zoom fits small objects correctly
- [ ] Orthographic zoom fits large objects correctly
- [ ] Orthographic zoom fits wide vs. tall objects correctly
- [ ] Margin settings work the same as perspective
- [ ] Convergence rate feels similar to perspective
- [ ] Switching projection mid-zoom works correctly
- [ ] Debug visualization shows correct bounds for both modes

#### Simplified Approach (If Complex)

If the full convergence algorithm is too complex for orthographic, consider a simpler direct calculation:

```rust
fn update_orthographic_zoom_simple(
    pan_orbit: &mut PanOrbitCamera,
    bounds: &[Vec3],
    boundary_center: Vec3,
    orthographic: &OrthographicProjection,
    camera: &Camera,
    config: &ZoomToFitConfig,
) {
    // 1. Set focus to boundary center (instant)
    pan_orbit.target_focus = boundary_center;

    // 2. Calculate bounding box size in world space
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);

    for &point in bounds {
        min = min.min(point);
        max = max.max(point);
    }

    let size = max - min;

    // 3. Calculate required scale (largest dimension with margin)
    let viewport_size = camera.logical_viewport_size()
        .unwrap_or(Vec2::new(1920.0, 1080.0));
    let aspect_ratio = viewport_size.x / viewport_size.y;

    let width_scale = size.x / (aspect_ratio * (1.0 - config.margin));
    let height_scale = size.y / (1.0 - config.margin);

    let target_scale = width_scale.max(height_scale);

    // 4. Apply scale (assuming PanOrbitCamera has orthographic_scale)
    pan_orbit.orthographic_scale = Some(target_scale);
    pan_orbit.force_update = true;
}
```

This simpler version:
- Instantly centers on target (no convergence)
- Directly calculates required scale
- No iterative process
- Good for first implementation, can add convergence later

## Migration Path

### For nateroids:

**Before:**
```rust
// Boundary is a resource
commands.insert_resource(Boundary { ... });

// Input triggers system directly
.add_systems(Update, start_zoom_to_fit.run_if(just_pressed(...)))

// Hardcoded Vec3::ZERO in convergence
```

**After:**
```rust
// Boundary is an entity
commands.spawn((
    Transform::default(),
    CustomBounds(corners),
    BoundaryMarker,
));

// Input inserts component on target
fn handle_input(...) {
    if pressed {
        commands.entity(boundary).insert(ZoomToFit);
    }
}

// Center calculated from actual bounds
```

### For bevy_panorbit_camera users:

```rust
// Zoom to object with AABB
commands.entity(asteroid).insert(ZoomToFit);

// Zoom to custom region
let region = commands.spawn((
    Transform::default(),
    CustomBounds(my_bounds),
    ZoomToFit,
)).id();
```

## Key Design Decisions

### 1. Why marker on target instead of camera?
- **More intuitive**: "Frame this object" vs "Frame these bounds"
- **ECS-idiomatic**: Tag the thing you care about
- **Simpler**: Target entity already has bounds (Aabb/Transform)
- **Flexible**: Can mark different entities dynamically

### 2. Why separate ZoomToFitState on camera?
- **Clean API**: User sees simple marker, internals are hidden
- **Camera state**: Iteration count, saved smoothness belong to camera
- **Separation**: User-facing vs. implementation detail

### 3. Why priority CustomBounds > Aabb > Transform?
- **Flexibility**: Explicit bounds override mesh-derived bounds
- **Point targets**: Transform-only entities still work
- **Common case**: Most entities have Aabb from mesh

### 4. Why last-target-wins for multiple ZoomToFit?
- **Simplicity**: No complex queueing or blending
- **Clear behavior**: Latest request takes priority
- **User control**: Can always remove previous markers first

### 5. Why calculate center instead of Vec3::ZERO?
- **Generic**: Works for targets not at origin
- **Correct**: Centers on actual bounds, not world origin
- **Robust**: Handles arbitrary target positions

## Success Criteria

1. **Functionality**
   - [x] Simple marker component on target
   - [x] Observer-based initialization
   - [x] Bounds from CustomBounds/Aabb/Transform
   - [x] Center calculated from actual bounds (no Vec3::ZERO)
   - [x] Feature-gated
   - [x] Debug visualization
   - [ ] **Perspective projection support** (current implementation)
   - [ ] **Orthographic projection support** (Phase 8)

2. **Code Quality**
   - [ ] No game-specific types
   - [ ] Comprehensive documentation
   - [ ] Example demonstrates all features
   - [ ] Follows bevy_panorbit_camera style

3. **Testing**
   - [ ] Works in nateroids
   - [ ] Example runs without errors
   - [ ] Different target types work
   - [ ] Debug visualization accurate

4. **PR Readiness**
   - [ ] Isolated module
   - [ ] No dependencies beyond bevy
   - [ ] CI passes
   - [ ] PR description written

## Open Questions

1. **Multiple cameras?**
   - Current: Uses `.single()` for camera query
   - Future: Camera selection mechanism?
   - Decision: Start simple, extend if needed

2. **Frame-rate independence?**
   - Current: Fixed percentage per frame
   - Better: Use `Time.delta()` for consistent behavior
   - Decision: Yes, multiply convergence_rate by delta_secs

3. **Dynamic bounds?**
   - Should we re-query bounds each frame for animated targets?
   - Pro: Supports moving/scaling targets
   - Con: Small performance cost
   - Decision: Yes, more flexible

4. **Orthographic projection?**
   - Current: Only perspective
   - Requirement: bevy_panorbit_camera supports both perspective AND orthographic
   - Decision: MUST support both for complete PR - see Phase 8 for implementation

5. **Remove ZoomToFit when done?**
   - Current: We remove ZoomToFitState from camera
   - Should we also remove ZoomToFit from target?
   - Decision: No - user might want to re-trigger

## Conclusion

This plan transforms a camera-centric, game-specific feature into a target-centric, generic library component. The key insights:

1. **Simple marker** on target entity (not camera)
2. **Observer-driven** initialization (automatic)
3. **Bounds priority** (CustomBounds > Aabb > Transform)
4. **Calculated center** (from actual bounds, not Vec3::ZERO)
5. **Internal state** on camera (hidden from user)
6. **Complete projection support** (both Perspective and Orthographic)

The result: A dead-simple API that "just works" for any bounded entity with any camera projection, while hiding all complexity behind the scenes. Perfect for a bevy_panorbit_camera feature!

## Implementation Sequence

For a complete PR to bevy_panorbit_camera:

1. **Phase 1-7**: Refactor and genericize perspective implementation (current)
2. **Phase 8**: Add orthographic projection support (required for complete feature)
3. Test both projection types thoroughly
4. Submit PR with both implementations

Note: Orthographic support is not optional - since bevy_panorbit_camera supports both projections, zoom-to-fit must work with both to be a complete, professional feature worth merging.
