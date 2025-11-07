# Merge Actor Struct Plan

## Problem Analysis

We currently have **duplicate actor marker structs** defined in multiple locations:

### Current State

1. **`Missile`** - defined in TWO places:
   - `src/actor/missile.rs:27-36` - Contains data fields for tracking state
   - `src/actor/actor_spawner.rs:370-382` - Marker component with `#[require]` attributes

2. **`Spaceship`** - defined in TWO places:
   - `src/actor/spaceship.rs:9-11` - Empty marker component
   - `src/actor/actor_spawner.rs:397-408` - Marker component with `#[require]` attributes

3. **`Nateroid`** - defined in ONE place:
   - `src/actor/actor_spawner.rs:384-395` - Marker component with `#[require]` attributes
   - No separate definition in `nateroid.rs`

### How They're Currently Used

**spawn_actor flow:**
```rust
// actor_spawner.rs:529-591
match config.actor_kind {
    ActorKind::Missile => commands.spawn((
        Missile,  // <-- marker from actor_spawner.rs
        // ... other components
    )),
}
```

**After spawn_actor:**
```rust
// missile.rs:110-118
let missile = Missile::new(distance);  // <-- data struct from missile.rs
spawn_actor(...).insert(missile);  // <-- inserts SECOND Missile component

// spaceship.rs:33-35
spawn_actor(...).insert(Spaceship);  // <-- redundant! already spawned

// nateroid.rs:42
spawn_actor(...);  // <-- no .insert(), relies on spawn_actor's marker
```

### The Problem

1. **Name collision** - Two different `Missile` types with same name in different modules
2. **Redundancy** - `Spaceship` marker inserted twice (once in spawn_actor, once after)
3. **Inconsistency** - `Missile` has data fields separate from marker, but `Spaceship` doesn't
4. **Confusion** - Hard to understand which marker is "the real one"
5. **Maintenance burden** - Changes to required components must be made in actor_spawner.rs, not in the logical module

## Goal

Consolidate to **ONE struct per actor type** in their **respective modules**, maintaining all functionality.

## Solution Design

### Approach

1. **Move all marker structs** from `actor_spawner.rs` to their logical modules
2. **Merge data fields** (for `Missile`) into the marker struct
3. **Keep `#[require]` attributes** on the moved structs
4. **Update actor_spawner.rs** to import instead of define
5. **Remove redundant `.insert()` calls**

### Key Decision: Missile Data Fields (No Default Needed)

The `Missile` struct has data fields that need custom initialization:
```rust
Missile::new(total_distance: f32) -> Self {
    Missile {
        total_distance,
        traveled_distance: 0.,
        remaining_distance: 0.,
        last_position: None,
        last_teleport_position: None,
    }
}
```

**Important:** Despite having `#[require]` attributes, the `Missile` component itself does NOT need `Default`. Only the REQUIRED components (Transform, Teleporter, etc.) need `Default`, not the component doing the requiring. This is confirmed by Bevy's own examples (e.g., breakout.rs where Wall has `#[require]` but no `Default`).

**Rationale:** Not implementing `Default` prevents invalid states (Missile with total_distance: 0.0) and follows the principle of "make invalid states unrepresentable".

## Implementation Steps

### Step 1: Update `src/actor/missile.rs`

**Location:** `src/actor/missile.rs:27-36`

**Current:**
```rust
#[derive(Reflect, Copy, Clone, Component, Debug)]
#[reflect(Component)]
pub struct Missile {
    pub total_distance: f32,
    pub traveled_distance: f32,
    remaining_distance: f32,
    pub last_position: Option<Vec3>,
    last_teleport_position: Option<Vec3>,
}
```

**New:**
```rust
#[derive(Component, Reflect, Copy, Clone, Debug)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Missile {
    pub total_distance: f32,
    pub traveled_distance: f32,
    remaining_distance: f32,
    pub last_position: Option<Vec3>,
    last_teleport_position: Option<Vec3>,
}

// Keep the existing new() method - no Default needed
```

**Changes:**
- Add `#[require(...)]` attributes from actor_spawner.rs:370-382
- Keep existing data fields and `new()` method (NO Default implementation needed)
- Import physics constants from actor_spawner (see Step 1b)
- Preserve `Copy` trait (see rationale below)

**Copy Trait Rationale:**

The merged `Missile` struct retains the `Copy` trait from the original data struct for these reasons:

1. **All fields are Copy-compatible**: `f32` and `Option<Vec3>` both implement `Copy`
2. **No conflict with #[require]**: The `Copy` trait is compatible with required components - it only affects how the struct itself is passed, not its component behavior
3. **Performance benefit**: Small structs (40 bytes) benefit from `Copy` semantics in queries where components are accessed by value
4. **Future flexibility**: While no current code requires `Copy` (despawn.rs:22 was refactored to use reference pattern `for (entity, missile)` instead of `for (entity, &missile)`), keeping `Copy` maintains the option for future optimizations

**Note:** The current marker struct in actor_spawner.rs does NOT have `Copy`, so adding it to the merged struct is a slight enhancement. The codebase is Copy-independent, so removing this trait in the future would not break any existing code.

**Transform in #[require] - Why Keep It When We Override?**

The `#[require(Transform)]` attribute provides a default `Transform::default()` that is **always overridden** during spawning (Step 5a calculates a custom transform). Despite this, we keep Transform in the requirement list because:

1. **Safety net**: Ensures Transform exists even if Missile is spawned via alternative code paths (testing, debugging, future extensions)
2. **Documentation**: Makes it explicit that Transform is semantically required for Missile to function
3. **Consistency**: Other required components (Teleporter, ActorPortals, etc.) DO use their defaults - keeping Transform makes the list complete
4. **Type guarantees**: Queries can rely on Transform always being present with `&Missile`

The alternative (manually adding Transform in every spawn call) is more error-prone than accepting the negligible overhead of creating and immediately overwriting a default Transform.

### Step 1b: Add imports to missile.rs

**Location:** Update imports section (after line 1)

**Add the following imports:**
```rust
use avian3d::prelude::*;
use crate::actor::actor_spawner::{LOCKED_AXES_2D, ZERO_GRAVITY};
use crate::playfield::ActorPortals;
```

**Note:**
- `avian3d::prelude::*` provides physics types: RigidBody, GravityScale, LockedAxes, CollisionEventsEnabled, Collider, Aabb, Mass, Restitution, LinearVelocity, AngularVelocity, CollisionLayers
- `LOCKED_AXES_2D`, `ZERO_GRAVITY` are physics constants defined in actor_spawner.rs (see Step 4a)
- `ActorPortals` is required by the `#[require]` attribute
- `Teleporter` is already imported at line 5
- `bevy::prelude::*` (already imported) provides: Transform, Name, SceneRoot, and other common Bevy types

### Step 2: Update `src/actor/spaceship.rs`

**Location:** `src/actor/spaceship.rs:9-11`

**Current:**
```rust
#[derive(Reflect, Component, Debug)]
#[reflect(Component)]
pub struct Spaceship;
```

**Add imports after line 1:**
```rust
use bevy::prelude::*;
use avian3d::prelude::*;

use crate::actor::Teleporter;
use crate::actor::actor_spawner::{LOCKED_AXES_SPACESHIP, ZERO_GRAVITY, spawn_actor};
use crate::actor::actor_template::SpaceshipConfig;
use crate::actor::spaceship_control::SpaceshipControl;
use crate::playfield::ActorPortals;
use crate::schedule::InGameSet;
use crate::state::GameState;
```

**New struct definition:**
```rust
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,
    LockedAxes = LOCKED_AXES_SPACESHIP
)]
pub struct Spaceship;
```

**Changes:**
- Add `#[require(...)]` attributes from actor_spawner.rs:397-408 (NO Default derive needed)
- Add imports for:
  - `avian3d::prelude::*` (provides RigidBody, GravityScale, LockedAxes, CollisionEventsEnabled)
  - `crate::actor::Teleporter` (required component)
  - `crate::playfield::ActorPortals` (required component)
  - `crate::actor::actor_spawner::{LOCKED_AXES_SPACESHIP, ZERO_GRAVITY}` (physics constants)

**Call site update:**
- Remove `.insert(Spaceship)` at line 35

### Step 3: Update `src/actor/nateroid.rs`

**Location:** `src/actor/nateroid.rs` (currently has no struct definition)

**Add imports after line 1:**
```rust
use bevy::prelude::*;
use avian3d::prelude::*;

use crate::actor::Teleporter;
use crate::actor::actor_spawner::{LOCKED_AXES_2D, ZERO_GRAVITY, spawn_actor};
use crate::actor::actor_template::NateroidConfig;
use crate::global_input::GlobalAction;
use crate::global_input::toggle_active;
use crate::playfield::ActorPortals;
use crate::playfield::Boundary;
use crate::schedule::InGameSet;
```

**Add struct definition after imports:**
```rust
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,
    LockedAxes = LOCKED_AXES_2D
)]
pub struct Nateroid;
```

**Changes:**
- Move struct from actor_spawner.rs:384-395 (NO Default derive needed)
- Add imports for:
  - `avian3d::prelude::*` (provides RigidBody, GravityScale, LockedAxes, CollisionEventsEnabled)
  - `crate::actor::Teleporter` (required component)
  - `crate::playfield::ActorPortals` (required component)
  - `crate::actor::actor_spawner::{LOCKED_AXES_2D, ZERO_GRAVITY}` (physics constants)

**No call site changes needed** - already working correctly

### Step 4: Update `src/actor/actor_spawner.rs`

**⚠️ CRITICAL: ATOMIC OPERATION REQUIRED**

Steps 4a-final and 4b must be performed together as a single atomic change. **Do not attempt to compile between removing struct definitions (4a-final) and adding imports (4b).** The code at line 59 (`propagate_render_layers_on_spawn` query) and lines 529-591 (`spawn_actor` match statement) both reference the `Missile`, `Nateroid`, and `Spaceship` types. Compiling after 4a-final but before 4b will produce compilation errors.

**Correct sequence:**
1. Step 4a: Convert functions to constants (compilable ✓)
2. Step 4a-interim: Update existing structs to use constants (compilable ✓)
3. Step 4a-final + 4b: Remove struct definitions and add imports atomically (must be single edit)
4. Step 4c onwards: Continue with remaining updates

#### Step 4a: Convert helper functions to public constants

**Replace lines 354-367** (convert functions to constants to eliminate duplication):
```rust
// Replace private helper functions with public constants
pub const ZERO_GRAVITY: GravityScale = GravityScale(0.0);
pub const LOCKED_AXES_2D: LockedAxes = LockedAxes::new().lock_translation_z();
pub const LOCKED_AXES_SPACESHIP: LockedAxes = LockedAxes::new()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_translation_z();
```

**Rationale:** Converting to constants eliminates code duplication across missile.rs, spaceship.rs, and nateroid.rs while following DRY principles. The constants are compile-time evaluated and provide a single source of truth for physics configuration.

**Verification:** Run `cargo check` - should compile successfully (existing structs still use old function calls, updated in next step).

#### Step 4a-interim: Update existing structs to use new constants

**⚠️ Required before removing helper functions** - The existing struct definitions (lines 368-404) currently reference the helper functions in their `#[require]` attributes. These must be updated to use the new constants before the functions can be safely removed.

**Update Missile struct (lines 368-378):**
```rust
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,        // Changed from zero_gravity()
    LockedAxes = LOCKED_AXES_2D         // Changed from locked_axes_2d()
)]
pub struct Missile;
```

**Update Nateroid struct (lines 380-391):**
```rust
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,        // Changed from zero_gravity()
    LockedAxes = LOCKED_AXES_2D         // Changed from locked_axes_2d()
)]
pub struct Nateroid;
```

**Update Spaceship struct (lines 393-404):**
```rust
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = ZERO_GRAVITY,            // Changed from zero_gravity()
    LockedAxes = LOCKED_AXES_SPACESHIP      // Changed from locked_axes_spaceship()
)]
pub struct Spaceship;
```

**Verification:** Run `cargo check` - should compile successfully. All `#[require]` attributes now use constants instead of function calls.

#### Step 4a-final: Remove struct definitions

**Remove lines 368-404** (struct definitions will be imported from respective modules):
```rust
// DELETE these marker structs (note: they incorrectly derive Default):
#[derive(Component, Default, Reflect)]
pub struct Missile;

#[derive(Component, Default, Reflect)]
pub struct Nateroid;

#[derive(Component, Default, Reflect)]
pub struct Spaceship;
```

**Note:** The current marker structs incorrectly derive `Default`. Components with `#[require]` attributes do NOT need `Default` themselves - only the required components need it. The new consolidated structs in their respective modules correctly omit `Default`.

#### Step 4b: Add imports (perform immediately after 4a)

**Add to imports section:**
```rust
use crate::actor::missile::Missile;
use crate::actor::nateroid::Nateroid;
use crate::actor::spaceship::Spaceship;
```

**Verification:** After completing both 4a and 4b together, run `cargo check` to verify compilation succeeds. The query at line 59 and spawn_actor match at lines 529-591 will now use the imported types.

#### Step 4c: Update query in propagate_render_layers_on_spawn

**Location:** Line 59

**No changes needed** - query will work with imported types:
```rust
q_parents: Query<&RenderLayers, Or<(With<Missile>, With<Nateroid>, With<Spaceship>)>>,
```

#### Step 4d: Update spawn_actor function

**Location:** Lines 529-591

**No changes needed** - spawn_actor will use imported types, functionality remains the same.

#### Step 4e: Make helper functions public for missile.rs

**Location:** Lines 282 and ~490 (ActorConfig impl block)

**Change `apply_rotations` to public:**

Find line 282:
```rust
fn apply_rotations(
    config: &ActorConfig,
    parent_transform: Option<&Transform>,
    transform: &mut Transform,
)
```

Change to:
```rust
pub fn apply_rotations(
    config: &ActorConfig,
    parent_transform: Option<&Transform>,
    transform: &mut Transform,
)
```

**Change `ActorConfig::calculate_spawn_transform` to public:**

Find the method in the ActorConfig impl block (~line 490):
```rust
fn calculate_spawn_transform(
    &self,
    parent: Option<&Transform>,
    boundary: Option<Res<Boundary>>,
) -> Transform {
```

Change to:
```rust
pub fn calculate_spawn_transform(
    &self,
    parent: Option<&Transform>,
    boundary: Option<Res<Boundary>>,
) -> Transform {
```

**Rationale:** Step 5a refactors missile spawning to call these functions directly instead of using spawn_actor. These functions must be public for missile.rs to access them.

### Step 5: Update call sites

#### Step 5a: `src/actor/missile.rs:110-118`

**Current:**
```rust
let missile = Missile::new(boundary_config.max_missile_distance());

spawn_actor(
    &mut commands,
    &missile_config.0,
    None,
    Some((spaceship_transform, spaceship_linear_velocity, aabb)),
)
.insert(missile);
```

**New:**
```rust
let config = &missile_config.0;
let missile = Missile::new(boundary_config.max_missile_distance());

// Calculate transform
let mut transform = config.calculate_spawn_transform(
    Some(spaceship_transform),
    None,
);
apply_rotations(config, Some(spaceship_transform), &mut transform);

// Calculate velocity
let (linear_velocity, angular_velocity) = config
    .velocity_behavior
    .calculate_velocity(Some(spaceship_linear_velocity), Some(spaceship_transform));

// Spawn with consolidated Missile component
commands.spawn((
    missile,  // Brings Transform, Teleporter, ActorPortals, etc. via #[require]
    config.actor_kind,
    transform,  // Override the Transform from #[require]
    config.aabb.clone(),
    config.collider.clone(),
    CollisionDamage(config.collision_damage),
    config.collision_layers,
    Health(config.health),
    Restitution {
        coefficient: config.restitution,
        combine_rule: config.restitution_combine_rule,
    },
    Mass(config.mass),
    RenderLayers::from_layers(config.render_layer.layers()),
    SceneRoot(config.scene.clone()),
    linear_velocity,
    angular_velocity,
    Name::new("Missile"),
));
```

**Add necessary imports:**
```rust
use bevy::camera::visibility::RenderLayers;
use crate::actor::actor_spawner::{ActorKind, CollisionDamage, Health, apply_rotations};
use crate::camera::RenderLayer;
```

**Import coverage analysis:**
The new spawning code uses many types. Here's where they come from:
- **From bevy::prelude::*** (already imported line 1): `Transform`, `Name`, `Commands`, `Entity`, `SceneRoot`
- **From avian3d::prelude::*** (added in Step 1b): `Aabb`, `Collider`, `Mass`, `Restitution`, `LinearVelocity`, `AngularVelocity`, `CollisionLayers`
- **From bevy::camera::visibility**: `RenderLayers` (NEW - added above)
- **From crate::actor::actor_spawner**: `ActorKind`, `CollisionDamage`, `Health`, `apply_rotations` (NEW - added above)
- **From crate::camera**: `RenderLayer` (NEW - added above)
- **From crate::actor::Teleporter**: Already imported at line 5
- **From crate::playfield::ActorPortals**: Added in Step 1b

**Types NOT needing import:**
- `VelocityBehavior`: Accessed via field access (`config.velocity_behavior.calculate_velocity()`), not direct type usage. No import needed.
- `ActorConfig`: Accessed via `missile_config.0` field. Type is already accessible through `MissileConfig`.
- `SpawnPosition`, `RotationBehavior`, etc.: Only used within ActorConfig, not directly referenced in missile.rs.

**Note:** The `apply_rotations` and `calculate_spawn_transform` functions are made public in Step 4e.

#### Step 5b: `src/actor/spaceship.rs:33-35`

**Current:**
```rust
spawn_actor(&mut commands, &spaceship_config.0, None, None)
    .insert(SpaceshipControl::generate_input_map())
    .insert(Spaceship);
```

**New:**
```rust
spawn_actor(&mut commands, &spaceship_config.0, None, None)
    .insert(SpaceshipControl::generate_input_map());
```

**Remove** the `.insert(Spaceship)` line since spawn_actor already spawns with the marker.

#### Step 5c: `src/actor/nateroid.rs:42`

**Current:**
```rust
spawn_actor(&mut commands, nateroid_config, Some(boundary), None);
```

**No change needed** - already correct

### Step 6: Update `src/actor/mod.rs`

**Location:** `src/actor/mod.rs`

**Add public exports:**
```rust
pub use crate::actor::missile::Missile;
pub use crate::actor::nateroid::Nateroid;
pub use crate::actor::spaceship::Spaceship;
```

This makes the consolidated types available at the module level.

## Testing Checklist

After implementation, verify:

### Compilation
- [ ] `cargo build` succeeds
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo +nightly fmt` completes

### Runtime Behavior
- [ ] Spaceship spawns correctly with all required components
- [ ] Spaceship controls work (movement, rotation, fire)
- [ ] Missiles spawn correctly when firing
- [ ] Missile tracking logic works (distance traveled, teleportation)
- [ ] Missiles despawn after traveling max distance
- [ ] Nateroids spawn periodically
- [ ] Nateroids have correct physics (velocity, collisions)
- [ ] Collision detection works for all actors
- [ ] Render layers propagate to children correctly
- [ ] Teleportation works for all actors
- [ ] Component queries work (With<Missile>, With<Spaceship>, With<Nateroid>)

### Reflection/Inspection
- [ ] Missile inspector shows data fields correctly
- [ ] Spaceship inspector shows components
- [ ] Nateroid inspector shows components
- [ ] Reflection system can access all components

## Risks and Mitigations

### Risk 1: Breaking existing queries
**Mitigation:** Search for all `With<Missile>`, `With<Spaceship>`, `With<Nateroid>` queries - should work identically

### Risk 2: Missile data lost during spawn
**Mitigation:** Careful testing of missile spawning and tracking logic

### Risk 3: Required components not applying
**Mitigation:** Verify in inspector that all required components are present

### Risk 4: Compilation errors from duplicate types
**Mitigation:** Remove old definitions before adding imports to avoid conflicts

## Order of Implementation

To minimize compilation errors:

1. Step 1 - Missile (most complex)
2. Step 3 - Nateroid (no conflicts)
3. Step 2 - Spaceship (simple)
4. Step 4 - actor_spawner.rs (imports after definitions exist)
5. Step 5 - Call sites (after actor_spawner updated)
6. Step 6 - mod.rs exports
7. Testing

## Summary

This plan consolidates duplicate actor marker structs into single, canonical definitions in their logical modules. It maintains all current functionality while improving code organization, maintainability, and clarity.

**Key changes:**
- ONE `Missile` struct in missile.rs with data fields + #[require]
- ONE `Spaceship` struct in spaceship.rs with #[require]
- ONE `Nateroid` struct in nateroid.rs with #[require]
- actor_spawner.rs imports these instead of defining them
- Remove redundant `.insert()` calls
- Refactor missile spawning to spawn directly

**Benefits:**
- Clear ownership: each actor module owns its marker struct
- No name collisions or confusion
- Easier to modify required components (change in logical location)
- Consistent pattern across all actors
- Less code duplication
