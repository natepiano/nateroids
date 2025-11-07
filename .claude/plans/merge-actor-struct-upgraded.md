# Merge Actor Struct Plan (Collaborative Mode)

## EXECUTION PROTOCOL

<Instructions>
For each step in the implementation sequence:

1. **DESCRIBE**: Present the changes with:
   - Summary of what will change and why
   - Code examples showing before/after
   - List of files to be modified
   - Expected impact on the system

2. **AWAIT APPROVAL**: Stop and wait for user confirmation ("go ahead" or similar)

3. **IMPLEMENT**: Make the changes and stop

4. **BUILD & VALIDATE**: Execute the build process:
   ```bash
   cargo check
   ```
   For final validation step:
   ```bash
   cargo build && cargo clippy && cargo +nightly fmt
   ```

5. **CONFIRM**: Wait for user to confirm the build succeeded

6. **MARK COMPLETE**: Update this document to mark the step as ✅ COMPLETED

7. **PROCEED**: Move to next step only after confirmation
</Instructions>

<ExecuteImplementation>
Find the next ⏳ PENDING step in the INTERACTIVE IMPLEMENTATION SEQUENCE below.

For the current step:
1. Follow the <Instructions/> above for executing the step
2. When step is complete, use Edit tool to mark it as ✅ COMPLETED
3. Continue to next PENDING step

If all steps are COMPLETED:
    Display: "✅ Implementation complete! All steps have been executed."
</ExecuteImplementation>

## INTERACTIVE IMPLEMENTATION SEQUENCE

### Step 1: Prepare Constants ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Additive (SAFE)
**Build Status**: ✅ Compiles successfully

**Objective**: Convert helper functions to public constants to eliminate code duplication

**Changes**:
- Replace private helper functions with public const values
- Enables DRY principle across multiple modules
- Uses const fn capability in avian3d 0.4.1

**Files Modified**:
- `src/actor/actor_spawner.rs` (lines 354-367)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 1 Implementation Details" section below

---

### Step 2: Update Existing Structs to Use Constants ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Replacement (SAFE)
**Build Status**: ✅ Compiles successfully
**Dependencies**: Requires Step 1

**Objective**: Update existing struct #[require] attributes to reference new constants

**Changes**:
- Change `GravityScale = zero_gravity()` to `GravityScale = ZERO_GRAVITY`
- Change `LockedAxes = locked_axes_2d()` to `LockedAxes = LOCKED_AXES_2D`
- Change `LockedAxes = locked_axes_spaceship()` to `LockedAxes = LOCKED_AXES_SPACESHIP`
- Prepares for safe removal of helper functions in later step

**Files Modified**:
- `src/actor/actor_spawner.rs` (Missile, Nateroid, Spaceship struct definitions, lines 368-404)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 2 Implementation Details" section below

---

### Step 3: Make Helper Functions Public ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Additive (SAFE)
**Build Status**: ✅ Compiles successfully

**Objective**: Make helper functions public for missile.rs to use

**Changes**:
- Change `fn apply_rotations` to `pub fn apply_rotations`
- Change `fn calculate_spawn_transform` to `pub fn calculate_spawn_transform`
- Required for Step 8 (missile spawning refactor)

**Files Modified**:
- `src/actor/actor_spawner.rs` (line 282 and ActorConfig impl block ~line 490)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 3 Implementation Details" section below

---

### Step 4: Add Missile Struct with #[require] ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Additive (SAFE)
**Build Status**: ✅ Compiles successfully

**Objective**: Add #[require] attributes and imports to existing Missile data struct

**Changes**:
- Add #[require] attribute with Transform, Teleporter, ActorPortals, physics components
- Add imports for avian3d physics types and constants
- Preserve Copy trait (see Copy Trait Rationale)
- Preserve existing data fields and new() method
- NO Default implementation needed

**Files Modified**:
- `src/actor/missile.rs` (struct definition at lines 27-36, imports section)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 4 Implementation Details" section below

---

### Step 5: Add Spaceship Struct with #[require] ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Additive (SAFE)
**Build Status**: ✅ Compiles successfully

**Objective**: Add #[require] attributes and imports to Spaceship marker struct

**Changes**:
- Add #[require] attribute with Transform, Teleporter, ActorPortals, physics components
- Add imports for avian3d physics types and constants
- Uses LOCKED_AXES_SPACESHIP constant (different from missile/nateroid)
- NO Default implementation needed

**Files Modified**:
- `src/actor/spaceship.rs` (struct definition at lines 9-11, imports section)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 5 Implementation Details" section below

---

### Step 6: Add Nateroid Struct ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Additive (SAFE)
**Build Status**: ✅ Compiles successfully

**Objective**: Create Nateroid struct with #[require] in nateroid.rs

**Changes**:
- Create new Nateroid marker struct
- Add #[require] attribute with Transform, Teleporter, ActorPortals, physics components
- Add imports for avian3d physics types and constants
- NO Default implementation needed

**Files Modified**:
- `src/actor/nateroid.rs` (new struct definition after line 1)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 6 Implementation Details" section below

---

### Step 7: Atomic Import Swap ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Breaking (ATOMIC GROUP)
**Build Status**: ✅ Compiles successfully
**Dependencies**: Requires Steps 4, 5, 6

**Objective**: Remove old struct definitions and add imports from modules

**⚠️ CRITICAL**: This step MUST be performed as a single atomic edit. Do NOT attempt to compile between removing struct definitions and adding imports. The code at line 59 (propagate_render_layers_on_spawn query) and lines 529-591 (spawn_actor match statement) both reference the Missile, Nateroid, and Spaceship types.

**Changes**:
- Remove lines 368-404 (old Missile, Nateroid, Spaceship struct definitions)
- Add imports: `use crate::actor::missile::Missile;`
- Add imports: `use crate::actor::nateroid::Nateroid;`
- Add imports: `use crate::actor::spaceship::Spaceship;`
- These actions MUST happen together in one edit

**Files Modified**:
- `src/actor/actor_spawner.rs` (struct definitions removal, imports addition)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 7 Implementation Details" section below

---

### Step 8: Update Missile Spawning ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Refactoring (SAFE)
**Build Status**: ✅ Compiles successfully
**Dependencies**: Requires Step 3 (public helpers), Step 7 (imports)

**Objective**: Refactor fire_missile to spawn directly instead of using spawn_actor

**Changes**:
- Replace spawn_actor call with direct commands.spawn()
- Call calculate_spawn_transform and apply_rotations directly
- Calculate velocity using config.velocity_behavior
- Spawn with full component list including Transform override
- Add necessary imports (RenderLayers, ActorKind, CollisionDamage, Health, apply_rotations, RenderLayer)

**Files Modified**:
- `src/actor/missile.rs` (lines 110-118 fire_missile function, imports section)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 8 Implementation Details" section below

---

### Step 9: Remove Redundant Spaceship Insert ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Cleanup/Removal (SAFE)
**Build Status**: ✅ Compiles successfully
**Dependencies**: Requires Step 7

**Objective**: Remove redundant .insert(Spaceship) call

**Changes**:
- Remove `.insert(Spaceship)` from line 35
- Spaceship is now spawned via spawn_actor (which uses the imported marker with #[require])
- This insert was redundant

**Files Modified**:
- `src/actor/spaceship.rs` (line 35)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 9 Implementation Details" section below

---

### Step 10: Verify Copy-Independent Code ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Verification Only
**Build Status**: ✅ Already compiles

**Objective**: Confirm despawn.rs uses Copy-independent reference pattern

**Changes**:
- Verify despawn_missiles uses `for (entity, missile)` pattern (not `&missile`)
- This was already refactored during gap analysis
- No code changes needed - verification only

**Files Modified**:
- `src/despawn.rs` (verification only, line 22)

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 10 Implementation Details" section below

---

### Step 11: Add Public Exports ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Additive (SAFE)
**Build Status**: ✅ Compiles successfully

**Objective**: Add pub use statements to make types available at module level

**Changes**:
- Add `pub use crate::actor::missile::Missile;`
- Add `pub use crate::actor::nateroid::Nateroid;`
- Add `pub use crate::actor::spaceship::Spaceship;`
- Makes consolidated types available at the actor module level

**Files Modified**:
- `src/actor/mod.rs`

**Build Command**:
```bash
cargo check
```

**Details**: See "Step 11 Implementation Details" section below

---

### Step 12: Final Validation ✅ COMPLETED
**Status**: ✅ COMPLETED
**Change Type**: Verification
**Build Status**: ✅ Complete

**Objective**: Run full test suite and verify all functionality

**Verification Tasks**:
- [ ] `cargo build` succeeds
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo +nightly fmt` completes
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
- [ ] Missile inspector shows data fields correctly
- [ ] Spaceship inspector shows components
- [ ] Nateroid inspector shows components
- [ ] Reflection system can access all components

**Build Command**:
```bash
cargo build && cargo clippy && cargo +nightly fmt
```

**Details**: See "Step 12 Implementation Details" section below

---

## IMPLEMENTATION DETAILS

### Step 1 Implementation Details

**Location**: `src/actor/actor_spawner.rs` lines 354-367

**Current Code**:
```rust
fn zero_gravity() -> GravityScale {
    GravityScale(0.0)
}

fn locked_axes_2d() -> LockedAxes {
    LockedAxes::new().lock_translation_z()
}

fn locked_axes_spaceship() -> LockedAxes {
    LockedAxes::new()
        .lock_rotation_x()
        .lock_rotation_y()
        .lock_translation_z()
}
```

**Replace With**:
```rust
// Replace private helper functions with public constants
pub const ZERO_GRAVITY: GravityScale = GravityScale(0.0);
pub const LOCKED_AXES_2D: LockedAxes = LockedAxes::new().lock_translation_z();
pub const LOCKED_AXES_SPACESHIP: LockedAxes = LockedAxes::new()
    .lock_rotation_x()
    .lock_rotation_y()
    .lock_translation_z();
```

**Rationale**: Converting to constants eliminates code duplication across missile.rs, spaceship.rs, and nateroid.rs while following DRY principles. The constants are compile-time evaluated using avian3d 0.4.1's const fn support, providing a single source of truth for physics configuration.

**Verification**: After this change, run `cargo check` - should compile successfully. The existing structs still use the old function calls (will be updated in Step 2).

---

### Step 2 Implementation Details

**Location**: `src/actor/actor_spawner.rs` lines 368-404 (three struct definitions)

**⚠️ Required Before Removing Helper Functions**: The existing struct definitions currently reference the helper functions in their #[require] attributes. These must be updated to use the new constants before the functions can be safely removed.

**Update Missile Struct** (lines 368-378):

Current:
```rust
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = zero_gravity(),
    LockedAxes = locked_axes_2d()
)]
pub struct Missile;
```

Change to:
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

**Update Nateroid Struct** (lines 380-391):

Current:
```rust
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = zero_gravity(),
    LockedAxes = locked_axes_2d()
)]
pub struct Nateroid;
```

Change to:
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

**Update Spaceship Struct** (lines 393-404):

Current:
```rust
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(
    Transform,
    Teleporter,
    ActorPortals,
    CollisionEventsEnabled,
    RigidBody::Dynamic,
    GravityScale = zero_gravity(),
    LockedAxes = locked_axes_spaceship()
)]
pub struct Spaceship;
```

Change to:
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

**Verification**: Run `cargo check` - should compile successfully. All #[require] attributes now use constants instead of function calls. The old helper functions (zero_gravity, locked_axes_2d, locked_axes_spaceship) can now be safely removed in a future cleanup step.

---

### Step 3 Implementation Details

**Location**: `src/actor/actor_spawner.rs`

**Change apply_rotations to public** (line 282):

Find:
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

**Change ActorConfig::calculate_spawn_transform to public** (~line 490 in ActorConfig impl block):

Find:
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

**Rationale**: Step 8 refactors missile spawning to call these functions directly instead of using spawn_actor. These functions must be public for missile.rs to access them.

---

### Step 4 Implementation Details

**Location**: `src/actor/missile.rs`

**Current Struct** (lines 27-36):
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

**New Struct**:
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

**Add Imports** (after line 1):
```rust
use avian3d::prelude::*;
use crate::actor::actor_spawner::{LOCKED_AXES_2D, ZERO_GRAVITY};
use crate::playfield::ActorPortals;
```

**Import Notes**:
- `avian3d::prelude::*` provides physics types: RigidBody, GravityScale, LockedAxes, CollisionEventsEnabled
- `LOCKED_AXES_2D`, `ZERO_GRAVITY` are physics constants from actor_spawner.rs (defined in Step 1)
- `ActorPortals` is required by the #[require] attribute
- `Teleporter` is already imported at line 5
- `bevy::prelude::*` (already imported) provides Transform and other common types

**Copy Trait Rationale**:

The merged Missile struct retains the Copy trait from the original data struct for these reasons:

1. **All fields are Copy-compatible**: `f32` and `Option<Vec3>` both implement Copy
2. **No conflict with #[require]**: The Copy trait is compatible with required components - it only affects how the struct itself is passed, not its component behavior
3. **Performance benefit**: Small structs (40 bytes) benefit from Copy semantics in queries where components are accessed by value
4. **Future flexibility**: While no current code requires Copy (despawn.rs:22 was refactored to use reference pattern `for (entity, missile)` instead of `for (entity, &missile)`), keeping Copy maintains the option for future optimizations

**Note**: The current marker struct in actor_spawner.rs does NOT have Copy, so adding it to the merged struct is a slight enhancement. The codebase is Copy-independent, so removing this trait in the future would not break any existing code.

**Transform in #[require] - Why Keep It When We Override?**

The `#[require(Transform)]` attribute provides a default `Transform::default()` that is **always overridden** during spawning (Step 8 calculates a custom transform). Despite this, we keep Transform in the requirement list because:

1. **Safety net**: Ensures Transform exists even if Missile is spawned via alternative code paths (testing, debugging, future extensions)
2. **Documentation**: Makes it explicit that Transform is semantically required for Missile to function
3. **Consistency**: Other required components (Teleporter, ActorPortals, etc.) DO use their defaults - keeping Transform makes the list complete
4. **Type guarantees**: Queries can rely on Transform always being present with `&Missile`

The alternative (manually adding Transform in every spawn call) is more error-prone than accepting the negligible overhead of creating and immediately overwriting a default Transform.

**Key Decision: Missile Data Fields (No Default Needed)**

The Missile struct has data fields that need custom initialization via `Missile::new(total_distance)`. Despite having #[require] attributes, the Missile component itself does NOT need Default. Only the REQUIRED components (Transform, Teleporter, etc.) need Default, not the component doing the requiring. This is confirmed by Bevy's own examples (e.g., breakout.rs where Wall has #[require] but no Default).

---

### Step 5 Implementation Details

**Location**: `src/actor/spaceship.rs`

**Current Struct** (lines 9-11):
```rust
#[derive(Reflect, Component, Debug)]
#[reflect(Component)]
pub struct Spaceship;
```

**Add Imports** (after line 1):
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

**New Struct Definition**:
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

**Changes**:
- Add #[require] attributes from actor_spawner.rs:397-408 (NO Default derive needed)
- Add imports for:
  - `avian3d::prelude::*` (provides RigidBody, GravityScale, LockedAxes, CollisionEventsEnabled)
  - `crate::actor::Teleporter` (required component)
  - `crate::playfield::ActorPortals` (required component)
  - `crate::actor::actor_spawner::{LOCKED_AXES_SPACESHIP, ZERO_GRAVITY}` (physics constants)

**Note**: Uses LOCKED_AXES_SPACESHIP constant (different from missile/nateroid which use LOCKED_AXES_2D).

---

### Step 6 Implementation Details

**Location**: `src/actor/nateroid.rs` (currently has no struct definition)

**Add Imports** (after line 1):
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

**Add Struct Definition** (after imports):
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

**Changes**:
- Move struct from actor_spawner.rs:384-395 (NO Default derive needed)
- Add imports for:
  - `avian3d::prelude::*` (provides RigidBody, GravityScale, LockedAxes, CollisionEventsEnabled)
  - `crate::actor::Teleporter` (required component)
  - `crate::playfield::ActorPortals` (required component)
  - `crate::actor::actor_spawner::{LOCKED_AXES_2D, ZERO_GRAVITY}` (physics constants)

**No Call Site Changes Needed**: The nateroid spawning code (line 42) already works correctly - it calls `spawn_actor` which will now use the imported Nateroid type.

---

### Step 7 Implementation Details

**Location**: `src/actor/actor_spawner.rs`

**⚠️ CRITICAL: ATOMIC OPERATION REQUIRED**

Steps 7a and 7b MUST be performed together as a single atomic change. **Do not attempt to compile between removing struct definitions (7a) and adding imports (7b).** The code at line 59 (`propagate_render_layers_on_spawn` query) and lines 529-591 (`spawn_actor` match statement) both reference the `Missile`, `Nateroid`, and `Spaceship` types. Compiling after 7a but before 7b will produce compilation errors.

**Correct Approach**: Remove definitions and add imports in the same Edit tool call.

**Step 7a: Remove Struct Definitions** (lines 368-404):

Delete these three struct definitions:
```rust
#[derive(Component, Default, Reflect)]
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
pub struct Missile;

#[derive(Component, Default, Reflect)]
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

#[derive(Component, Default, Reflect)]
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

**Note**: The current marker structs incorrectly derive `Default`. Components with `#[require]` attributes do NOT need `Default` themselves - only the required components need it. The new consolidated structs in their respective modules correctly omit `Default`.

**Step 7b: Add Imports** (to imports section):

Add these three imports:
```rust
use crate::actor::missile::Missile;
use crate::actor::nateroid::Nateroid;
use crate::actor::spaceship::Spaceship;
```

**Verification**: After completing both 7a and 7b together as a single edit, run `cargo check` to verify compilation succeeds. The query at line 59 and spawn_actor match at lines 529-591 will now use the imported types.

**No Changes Needed**:
- Line 59 query: `q_parents: Query<&RenderLayers, Or<(With<Missile>, With<Nateroid>, With<Spaceship>)>>` - works with imported types
- Lines 529-591 spawn_actor match: Uses imported types, functionality remains the same

---

### Step 8 Implementation Details

**Location**: `src/actor/missile.rs` lines 110-118

**Current Code**:
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

**Replace With**:
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

**Add Necessary Imports** (to imports section):
```rust
use bevy::camera::visibility::RenderLayers;
use crate::actor::actor_spawner::{ActorKind, CollisionDamage, Health, apply_rotations};
use crate::camera::RenderLayer;
```

**Import Coverage Analysis**:

The new spawning code uses many types. Here's where they come from:
- **From bevy::prelude::*** (already imported line 1): `Transform`, `Name`, `Commands`, `Entity`, `SceneRoot`
- **From avian3d::prelude::*** (added in Step 4): `Aabb`, `Collider`, `Mass`, `Restitution`, `LinearVelocity`, `AngularVelocity`, `CollisionLayers`
- **From bevy::camera::visibility**: `RenderLayers` (NEW - added above)
- **From crate::actor::actor_spawner**: `ActorKind`, `CollisionDamage`, `Health`, `apply_rotations` (NEW - added above)
- **From crate::camera**: `RenderLayer` (NEW - added above)
- **From crate::actor::Teleporter**: Already imported at line 5
- **From crate::playfield::ActorPortals**: Added in Step 4

**Types NOT Needing Import**:
- `VelocityBehavior`: Accessed via field access (`config.velocity_behavior.calculate_velocity()`), not direct type usage. No import needed.
- `ActorConfig`: Accessed via `missile_config.0` field. Type is already accessible through `MissileConfig`.
- `SpawnPosition`, `RotationBehavior`, etc.: Only used within ActorConfig, not directly referenced in missile.rs.

**Note**: The `apply_rotations` and `calculate_spawn_transform` functions are made public in Step 3.

**Why This Refactor?**

This refactor solves a circular dependency problem:
- Moving Missile struct to missile.rs creates a circular dependency if missile.rs needs to call spawn_actor (which is in actor_spawner.rs)
- actor_spawner.rs imports Missile from missile.rs
- If missile.rs imports spawn_actor from actor_spawner.rs, we have a cycle

By having missile.rs spawn directly using the helper functions, we break the cycle. This is architecturally sound because:
- Missile is the only actor that needs custom spawning logic (due to its data fields)
- Spaceship and Nateroid can continue using spawn_actor
- The spawning logic is still shared via public helper functions

---

### Step 9 Implementation Details

**Location**: `src/actor/spaceship.rs` line 35

**Current Code**:
```rust
spawn_actor(&mut commands, &spaceship_config.0, None, None)
    .insert(SpaceshipControl::generate_input_map())
    .insert(Spaceship);
```

**Replace With**:
```rust
spawn_actor(&mut commands, &spaceship_config.0, None, None)
    .insert(SpaceshipControl::generate_input_map());
```

**Remove** the `.insert(Spaceship)` line since spawn_actor already spawns with the Spaceship marker (which now has #[require] attributes).

**Rationale**: After Step 7, spawn_actor uses the imported Spaceship type from spaceship.rs, which has #[require] attributes. The Spaceship component is automatically added during spawn, making the explicit .insert(Spaceship) redundant.

---

### Step 10 Implementation Details

**Location**: `src/despawn.rs` line 22

**Verification Only** - No changes needed.

**Current Code** (should already be in place from gap analysis):
```rust
fn despawn_missiles(mut commands: Commands, query: Query<(Entity, &Missile)>) {
    for (entity, missile) in query.iter() {
        if missile.traveled_distance >= missile.total_distance {
            despawn(&mut commands, entity);
        }
    }
}
```

**What to Verify**:
- The iteration pattern is `for (entity, missile)` NOT `for (entity, &missile)`
- This uses a reference pattern instead of copying via the destructuring pattern
- This makes the code Copy-independent

**Background**: During gap analysis (Gap 7), this code was refactored to eliminate the Copy trait dependency. The original pattern was `for (entity, &missile)` which required Missile to implement Copy. The new pattern works regardless of Copy trait.

**Why This Matters**:
- The Missile struct currently has Copy, so both patterns would work
- However, the reference pattern is more idiomatic and future-proof
- If someone later adds a non-Copy field to Missile, the code continues to work
- The compiler already provides complete protection (would fail to compile if Copy was required but missing)

**Verification Command**:
```bash
cargo check
```

Should compile successfully with no warnings about this pattern.

---

### Step 11 Implementation Details

**Location**: `src/actor/mod.rs`

**Add Public Exports**:
```rust
pub use crate::actor::missile::Missile;
pub use crate::actor::nateroid::Nateroid;
pub use crate::actor::spaceship::Spaceship;
```

**Rationale**: This makes the consolidated types available at the module level, allowing other modules to import them as `use crate::actor::Missile` instead of needing the full path `use crate::actor::missile::Missile`.

**Benefits**:
- Cleaner imports for external modules
- Maintains module encapsulation
- Standard Rust pattern for re-exporting types

---

### Step 12 Implementation Details

**Comprehensive Testing Checklist**:

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

**Build Command**:
```bash
cargo build && cargo clippy && cargo +nightly fmt
```

**If Any Tests Fail**:
- Identify which component is missing or incorrect
- Verify #[require] attributes are correctly applied
- Check that imports are complete
- Ensure Transform overrides are working
- Verify physics constants are correct

---

## PROBLEM ANALYSIS

### Current State

We currently have **duplicate actor marker structs** defined in multiple locations:

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

---

## GOAL

Consolidate to **ONE struct per actor type** in their **respective modules**, maintaining all functionality.

---

## SOLUTION DESIGN

### Approach

1. **Move all marker structs** from `actor_spawner.rs` to their logical modules
2. **Merge data fields** (for `Missile`) into the marker struct
3. **Keep `#[require]` attributes** on the moved structs
4. **Update actor_spawner.rs** to import instead of define
5. **Remove redundant `.insert()` calls**

---

## RISKS AND MITIGATIONS

### Risk 1: Breaking existing queries
**Mitigation:** Search for all `With<Missile>`, `With<Spaceship>`, `With<Nateroid>` queries - should work identically

### Risk 2: Missile data lost during spawn
**Mitigation:** Careful testing of missile spawning and tracking logic

### Risk 3: Required components not applying
**Mitigation:** Verify in inspector that all required components are present

### Risk 4: Compilation errors from duplicate types
**Mitigation:** Step 7 removes old definitions and adds imports atomically to avoid conflicts

### Risk 5: Circular dependency in imports
**Mitigation:** Step 8 refactors missile spawning to call helper functions directly instead of spawn_actor

---

## SUCCESS CRITERIA

This plan consolidates duplicate actor marker structs into single, canonical definitions in their logical modules. It maintains all current functionality while improving code organization, maintainability, and clarity.

**Key Changes:**
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
- Follows DRY principles

---

## DESIGN REVIEW SKIP NOTES

This plan has undergone comprehensive design review and gap analysis:

**Design Review Findings** (6 total):
- ✅ DESIGN-1 (Code duplication) - FIXED: Converted helper functions to public constants
- ✅ DESIGN-2 (Missing imports) - FIXED: Added complete import specifications
- ✅ DESIGN-3 (Copy trait rationale) - FIXED: Added comprehensive rationale section
- ✅ IMPLEMENTATION-1 (Inline spawning) - ACCEPTED: Solves circular dependency
- ✅ IMPLEMENTATION-2 (Atomic operation) - FIXED: Added prominent warning
- ✅ SIMPLIFICATION-1 (Default implementation) - FIXED: Removed unnecessary Default

**Gap Analysis** (8 gaps found, all addressed):
- Gap 1: Missing constant definitions - CONFIRMED CORRECT (avian3d 0.4.1 has const fn)
- Gap 2: Missing visibility change (apply_rotations) - FIXED: Added Step 4e
- Gap 3: Missing visibility change (calculate_spawn_transform) - FIXED: Added Step 4e
- Gap 4: Missing import requirements - FIXED: Added complete import lists
- Gap 5: Missing pub use exports - CONFIRMED CORRECT (field access doesn't need import)
- Gap 6: Incomplete refactoring details - FIXED: Added Transform override rationale
- Gap 7: Copy trait compatibility - CODE REFACTORED: despawn.rs now Copy-independent
- Gap 8: Incomplete require attribute changes - FIXED: Added Step 2 (interim update)

**Implementation Completeness**: ✅ VERIFIED
- All technical details preserved
- All code examples complete
- All edge cases documented
- All dependencies identified
- All build commands specified
- All verification steps included

The plan is comprehensive, implementation-ready, and safe to execute step-by-step.
