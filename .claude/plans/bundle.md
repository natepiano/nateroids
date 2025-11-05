# Actor Spawning System Simplification Plan

**Date**: 2025-11-04
**Goal**: Migrate from deprecated bundle-based spawning to Bevy 0.17+ component tuple pattern
**Complexity**: Low ⭐⭐⚪⚪⚪

## Executive Summary

The current actor spawning system uses deprecated bundles (`ActorBundle`) but has a solid architecture with inspector-compatible configuration resources. The migration to Bevy 0.16+ patterns requires minimal changes: remove the bundle struct and replace it with direct component tuple spawning. All existing functionality, including inspector support and spawn behavior, will be preserved.

---

## Current System Analysis

### Architecture Overview

**File**: `src/actor/actor_spawner.rs`

#### Key Components:

1. **`ActorConfig` Resource** (line 85-152)
   - 20+ configuration fields per actor type
   - Inspector-compatible via `bevy_inspector_egui`
   - Contains behavior enums:
     - `SpawnPositionBehavior` - Controls position calculation
     - `VelocityBehavior` - Controls velocity calculation
   - Handles rotation logic, timer management

2. **`ActorBundle` Struct** (line 274-363) **[TO BE REMOVED]**
   - Bundles 18 components together:
     - `ActorKind`, `Aabb`, `Collider`, `CollisionDamage`
     - `CollisionLayers`, `GravityScale`, `Health`, `LockedAxes`
     - `RigidBody`, `Restitution`, `Mass`, `RenderLayers`
     - `SceneRoot`, `Teleporter`, `Transform`
     - `LinearVelocity`, `AngularVelocity`, `ActorPortals`
   - Implements `new()` constructor from config
   - Note: `Name` and `CollisionEventsEnabled` are added via `.insert()` after spawn

3. **Configuration Pattern** (used throughout codebase):
   ```rust
   #[derive(Resource, Reflect, InspectorOptions)]
   pub struct MissileConfig(pub ActorConfig);
   ```

4. **`spawn_actor` Function** (line 499) **[TO BE MODIFIED]**
   - Current signature:
     ```rust
     pub fn spawn_actor<'a>(
         commands: &'a mut Commands,
         config: &ActorConfig,
         boundary: Option<Res<Boundary>>,
         parent: Option<(&Transform, &LinearVelocity, &Aabb)>,
     ) -> EntityCommands<'a>
     ```
   - Creates `ActorBundle` and spawns it

5. **Observer** (line 51-63) **[KEEP UNCHANGED]**
   - `propagate_render_layers_on_spawn`
   - Triggers on `Add<Children>`
   - Recursively propagates `RenderLayers` to scene children

### Current Spawn Call Sites

All of these remain unchanged:

1. **Spaceship** (`spaceship.rs`):
   ```rust
   spawn_actor(&mut commands, &spaceship_config.0, None, None)
       .insert(SpaceshipControl::generate_input_map())
       .insert(Spaceship);
   ```

2. **Missile** (`missile.rs`):
   ```rust
   spawn_actor(
       &mut commands,
       &missile_config.0,
       None,
       Some((spaceship_transform, spaceship_linear_velocity, aabb)),
   )
   .insert(missile);
   ```

3. **Nateroid** (`nateroid.rs`):
   ```rust
   spawn_actor(&mut commands, nateroid_config, Some(boundary), None);
   ```

### What Works Well

✅ Inspector integration allows runtime configuration tweaking
✅ Single source of truth for actor configuration
✅ Behavior enums provide flexibility
✅ Clean separation of spawn logic from game systems
✅ Observer pattern for render layer propagation

### Complexity Sources

⚠️ Bundle construction requires assembling 18 components each spawn
⚠️ Bundle pattern is deprecated in Bevy 0.16+
⚠️ Complex parent-relative calculations mixed with config data

---

## Research Findings

### Bevy 0.16+ Required Components Philosophy

**Core Principle**: "When spawning an entity, there should be a 'driving concept' component."

#### Key Changes in Bevy 0.16:

1. **Bundles Deprecated**
   - All built-in Bevy bundles removed
   - `Transform` now automatically adds `GlobalTransform`
   - `SceneRoot(handle)` is the primary component for scenes

2. **Modern Spawn Pattern**:
   ```rust
   commands.spawn((
       Component1,
       Component2::with_config(...),
       Transform::from_xyz(x, y, z),
       Name::new("Entity"),
   ))
   ```

3. **Physics Integration** (Avian3D):
   ```rust
   commands.spawn((
       Transform::default(),
       RigidBody::Dynamic,
       Collider::sphere(1.0),
       LinearVelocity::default(),
       // ... more components as needed
   ));
   ```

### Patterns from Hana Codebase

**File**: `hana/crates/hana/src/movable/cursor/spawn.rs`

Direct component tuple spawn with material configuration:
```rust
commands.spawn((
    Cursor { state: CursorMode::Disabled },
    Transform {
        translation: INITIAL_CURSOR_POSITION,
        scale: Vec3::splat(CURSOR_SIZE),
        ..default()
    },
    Mesh3d(mesh),
    MeshMaterial3d(material),
    Pickable::IGNORE,
    Visibility::Visible,
    Name::new("Cursor"),
));
```

**Key Insight**: Large component tuples with extensive configuration are acceptable and idiomatic in 0.16+

### Observer Patterns from Hana

**File**: `hana/crates/hana/src/movable/state/observers.rs`

Hana uses observers extensively for lifecycle management:
- Minimal spawn (just the driving component)
- Observers add remaining components
- State management through observer chains

**For Nateroids**: We could adopt this pattern in the future, but it's not necessary for this migration.

---

## Migration Strategy

### Recommended Approach: Minimal Refactor

This approach provides the best balance of simplicity and maintainability:

**Why This Approach**:
1. ✅ **Removes deprecated pattern** - No more bundle usage
2. ✅ **Maintains all functionality** - Exact same behavior
3. ✅ **Keeps inspector support** - Config resources unchanged
4. ✅ **No call site changes** - All spawn calls work as-is
5. ✅ **Minimal code changes** - Easy to review and test
6. ✅ **Future-proof** - Aligns with Bevy 0.16+ patterns

### Alternative Approaches Considered

#### Option B: Observer-Based Composition
- Split `ActorConfig` into smaller configs
- Use observers to add rendering/physics components
- **Verdict**: More complex, no clear benefit for current needs

#### Option C: Component-Based Config
- Make each config field a component
- Use component hooks for initialization
- **Verdict**: Loses inspector convenience, too aggressive

---

## Implementation Plan

### Phase 1: Remove Bundle Struct (Keep Helper Functions)

**File**: `src/actor/actor_spawner.rs`

**Lines to remove**: 274-363 (ActorBundle struct and its new() method only)

**IMPORTANT**: The `apply_rotations` helper function (lines 367-385) should be **kept and converted to a standalone function**. It was "extracted here for readability" and is still valuable.

Delete:
```rust
#[derive(Bundle)]
pub struct ActorBundle {
    pub actor_kind: ActorKind,
    pub aabb: Aabb,
    pub collider: Collider,
    pub collision_damage: CollisionDamage,
    pub collision_layers: CollisionLayers,
    pub gravity_scale: GravityScale,
    pub health: Health,
    pub locked_axes: LockedAxes,
    pub rigid_body: RigidBody,
    pub restitution: Restitution,
    pub mass_properties: Mass,
    pub render_layers: RenderLayers,
    pub scene_root: SceneRoot,
    pub teleporter: Teleporter,
    pub transform: Transform,
    pub linear_velocity: LinearVelocity,
    pub angular_velocity: AngularVelocity,
    pub wall_visualizer: ActorPortals,
}

impl ActorBundle {
    pub fn new(
        config: &ActorConfig,
        parent: Option<(&Transform, &LinearVelocity, &Aabb)>,
        boundary: Option<Res<Boundary>>,
    ) -> Self {
        // ... entire implementation
    }
}
```

### Phase 2: Update spawn_actor Function

**File**: `src/actor/actor_spawner.rs`

**Line**: 499 (current function start)

**Current implementation**:
```rust
pub fn spawn_actor<'a>(
    commands: &'a mut Commands,
    config: &ActorConfig,
    boundary: Option<Res<Boundary>>,
    parent: Option<(&Transform, &LinearVelocity, &Aabb)>,
) -> EntityCommands<'a> {
    let bundle = ActorBundle::new(config, parent, boundary);
    commands
        .spawn(bundle)
        .insert(Name::new(config.actor_kind.to_string()))
        .insert(CollisionEventsEnabled)
}
```

**New implementation**:
```rust
pub fn spawn_actor<'a>(
    commands: &'a mut Commands,
    config: &ActorConfig,
    boundary: Option<Res<Boundary>>,
    parent: Option<(&Transform, &LinearVelocity, &Aabb)>,
) -> EntityCommands<'a> {
    // Extract parent components
    let parent_transform = parent.map(|(t, _, _)| t);
    let parent_velocity = parent.map(|(_, v, _)| v);
    let parent_aabb = parent.map(|(_, _, a)| a);

    // Calculate spawn transform
    let mut transform = config.calculate_spawn_transform(
        parent_transform.zip(parent_aabb),
        boundary
    );

    // Apply rotation logic using existing helper function
    // NOTE: This preserves current behavior where rotation application happens in two phases:
    // 1. calculate_spawn_transform applies config.rotation (if present)
    // 2. apply_rotations may overwrite it when combining with parent rotation
    // For missiles: calculate_spawn_transform sets config rotation, then apply_rotations
    // overwrites with (spaceship_rotation * config_rotation). The intermediate application
    // is redundant but functionally correct - this is how the current code works.
    apply_rotations(config, parent_transform, &mut transform);

    // Calculate velocities (from ActorBundle::new)
    let (linear_velocity, angular_velocity) = config
        .velocity_behavior
        .calculate_velocity(parent_velocity, parent_transform);

    // Spawn with component tuple (replacing bundle)
    commands
        .spawn((
            config.actor_kind,
            config.aabb.clone(),
            config.collider.clone(),
            CollisionDamage(config.collision_damage),
            config.collision_layers,
            GravityScale(config.gravity_scale),
            Health(config.health),
            config.locked_axes,
            config.rigid_body,
            Restitution {
                coefficient: config.restitution,
                combine_rule: config.restitution_combine_rule,
            },
            Mass(config.mass),
            RenderLayers::from_layers(config.render_layer.layers()),
            SceneRoot(config.scene.clone()),
            Teleporter::default(),
            transform,
            linear_velocity,
            angular_velocity,
            ActorPortals::default(),
            Name::new(config.actor_kind.to_string()),
            CollisionEventsEnabled,
        ))
}

// Keep this helper function - convert from method to standalone function
fn apply_rotations(
    config: &ActorConfig,
    parent_transform: Option<&Transform>,
    transform: &mut Transform,
) {
    let final_rotation = parent_transform
        .map(|t| t.rotation)
        .map(|parent_rot| {
            config
                .rotation
                .map(|initial_rot| parent_rot * initial_rot)
                .unwrap_or(parent_rot)
        })
        .or(config.rotation);

    if let Some(rotation) = final_rotation {
        transform.rotation = rotation;
    }
}
```

**Note**: The `apply_rotations` function was previously a method on `ActorBundle` (with `Self::` prefix). After removing the bundle, it becomes a standalone function in the same module. The implementation stays exactly the same - just remove the `impl ActorBundle` context around it.

### Phase 3: Build, Format, and Test

**Build commands**:
```bash
cargo build && cargo +nightly fmt
cargo clippy
cargo nextest run
```

---

## Testing & Verification

### Automated Tests

Run the following commands:
```bash
cargo build          # Verify compilation
cargo clippy         # Check for warnings
cargo nextest run    # Run test suite
```

### Manual Testing Checklist

Launch the game and verify:

- [ ] **Compilation** - Code compiles without errors
- [ ] **No deprecation warnings** - No bundle-related warnings
- [ ] **Spaceship spawning** - Spawns at correct position and orientation
- [ ] **Missile spawning** - Inherits parent velocity correctly
- [ ] **Missile velocity** - Fires with correct speed relative to spaceship
- [ ] **Nateroid spawning** - Spawns within boundary constraints
- [ ] **Health values** - Match configuration settings
- [ ] **Collision detection** - Missiles hit nateroids correctly
- [ ] **Render layers** - Display correctly (stars layer 0, game layer 1)
- [ ] **Gizmo rendering** - Debug visualizations on correct layers
- [ ] **Inspector functionality** - All actor configs are editable
- [ ] **Game stability** - Runs normally without crashes

---

## Code Changes Summary

### Files Modified

**`src/actor/actor_spawner.rs`**:
- Remove: `ActorBundle` struct and impl (~90 lines)
- Modify: `spawn_actor` function (~40 lines)
- Keep: All other code unchanged

### Component Count

- **Bundle components**: 18 (being replaced)
- **Additional components**: 2 (`Name`, `CollisionEventsEnabled`)
- **Total spawned**: 20 components per actor

### Lines Changed

- **Removed**: ~90 lines (bundle struct and implementation)
- **Modified**: ~40 lines (spawn_actor function)
- **Added**: ~20 lines (inline component tuple)
- **Net change**: ~-70 lines

---

## Risks and Mitigations

### Risk: Component order might matter
**Likelihood**: Low
**Mitigation**: Component order in tuples doesn't affect functionality in Bevy

### Risk: Missing a component during migration
**Likelihood**: Low
**Mitigation**: All 18 bundle components are explicitly listed in the new code; compile will fail if any are missing

### Risk: Calculation logic might be incorrect
**Likelihood**: Very Low
**Mitigation**: Logic is copied directly from `ActorBundle::new()`; no changes to algorithm

### Note: Rotation Application Pattern
**Pattern**: The current code applies rotation in two phases:
1. `calculate_spawn_transform` applies `config.rotation` if present
2. `apply_rotations` may overwrite it when combining with parent rotation

**For missiles spawned from spaceship**: The config rotation is applied first, then immediately overwritten with `(parent_rotation * config_rotation)`. The intermediate application is redundant but functionally correct. This is the existing behavior being preserved - not a bug introduced by this refactor. Future refactoring could optimize this, but it's out of scope for a mechanical bundle removal.

---

## Future Enhancements (Not in This Plan)

These are potential improvements for later:

1. **Split ActorConfig** - Break into smaller, focused config structs
2. **Observer-based composition** - Use observers to add components post-spawn
3. **Component hooks** - Use Bevy's component hooks for initialization
4. **Generic spawn helpers** - Create trait-based spawn system

---

## References

### Bevy Documentation
- Migration guide: Bevy 0.16 required components
- Examples: `3d_shapes.rs`, scene loading patterns

### Hana Codebase Examples
- `hana/crates/hana/src/movable/cursor/spawn.rs` - Direct tuple spawn
- `hana/crates/hana/src/movable/state/observers.rs` - Observer patterns
- `hana/crates/hana/src/camera/primary_camera_plugin.rs` - Large component tuples

### Current Nateroids Code
- `src/actor/actor_spawner.rs` - Current bundle implementation
- `src/actor/spaceship.rs` - Spaceship spawn usage
- `src/actor/missile.rs` - Missile spawn with parent
- `src/actor/nateroid.rs` - Nateroid spawn with boundary

---

## Approval Checklist

Before executing this plan:

- [ ] Review the component list in new `spawn_actor` - all 20 components accounted for?
- [ ] Confirm rotation logic matches current `ActorBundle::new()`
- [ ] Confirm velocity calculation matches current implementation
- [ ] Verify no spawn call sites need changes
- [ ] Confirm observer (`propagate_render_layers_on_spawn`) stays unchanged

---

## Estimated Time

- **Code changes**: 15 minutes
- **Testing**: 30 minutes
- **Total**: 45 minutes

---

## Notes

- This is a mechanical refactor - no logic changes
- All spawn call sites remain unchanged
- Inspector support is preserved
- Observer pattern stays as-is
- Future observer-based improvements can build on this foundation
