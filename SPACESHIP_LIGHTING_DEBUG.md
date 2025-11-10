# Spaceship Lighting Issue - RESOLVED

## Solution
Inverted hierarchy: SpaceshipSpawnBuffer as parent, SceneRoot as unnamed child.

## Final Structure
**Parent (SpaceshipSpawnBuffer + Spaceship marker)**:
- Has Spaceship marker → gets physics components via #[require]
- Has full transform (position, rotation, scale)
- Has spawn buffer AABB
- This entity moves and has physics

**Child (unnamed scene entity)**:
- Has SceneRoot with GLTF model
- Has Transform::IDENTITY (renders at parent's origin)
- Just handles rendering, no game logic

## Why This Works
The smaller entity (scene) being child of larger entity (spawn buffer) allows directional lights on Game layer to properly illuminate the scene. When reversed, Transform on child somehow interferes with parent SceneRoot lighting.
