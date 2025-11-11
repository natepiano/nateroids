agent: You’re describing a “frame / zoom-to-fit at fixed FOV” (also called “reframe to bounds” or “dolly-to-fit”). Conceptually it’s not just pan+zoom; it’s: keep the camera’s yaw/pitch and perspective (FOV) fixed, then dolly straight along its view direction until the chosen object’s bounds exactly fill the viewport.

Why Claude likely left extra space
Most “fit” snippets use the object’s center or a bounding sphere, which overestimates for wide/aspect-skewed shapes and/or doesn’t account for camera yaw/pitch. To truly fill the frame you must test the projected extents of the object’s AABB in camera space against both the horizontal and vertical FOV and solve for the minimum dolly distance that satisfies all corners.

⸻

How to implement (Bevy + bevy_panorbit_camera)

Goal: Given a target Aabb (or compute from meshes), a camera with fixed PerspectiveProjection { fov }, current yaw, pitch, and some target point, compute the radius (distance) that makes the AABB just fit. Then set PanOrbitCamera’s yaw/pitch/target and update its radius.

Math sketch
	1.	Build the camera view matrix from the desired yaw/pitch about target.
	2.	Transform all 8 corners of the target AABB into camera space (view space).
	3.	Let vfov = projection.fov (vertical FOV).
Compute hfov = 2 * atan(tan(vfov/2) * aspect) where aspect = viewport_width / viewport_height.
Define tan_v = tan(vfov/2) and tan_h = tan(hfov/2).
	4.	For each camera-space corner p=(x,y,z) with the camera looking down -Z (Bevy default), the fits-inside inequalities are:
	•	|x| ≤ -z * tan_h
	•	|y| ≤ -z * tan_v
	5.	You’re allowed to move the camera straight back along its forward axis by distance t (a pure dolly), which adds t to (-z) for every point. Solve the smallest t ≥ 0 that satisfies both inequalities for all corners:
	•	From horizontal: t ≥ (|x|/tan_h) - (-z)
	•	From vertical:   t ≥ (|y|/tan_v) - (-z)
Take the max over all corners and both axes. Add a tiny epsilon for safety.

That t is the extra distance you need. Your new orbit radius = current_radius + t (or just set it to t if you started with the eye sitting at the target surface).

Bevy-style code (0.16/0.17-ish API names)

```rust
use bevy::prelude::*;
use bevy::render::primitives::Aabb;

// Inputs you control:
struct FrameParams {
    yaw: f32,            // radians
    pitch: f32,          // radians
    target: Vec3,        // orbit target point (object center or chosen focus)
    aabb: Aabb,          // world-space AABB of your object
    margin: f32,         // e.g., 1.02 for 2% padding
}

fn dolly_to_fit_fixed_fov(
    camera_transform: &GlobalTransform,          // your camera rig transform (only used for up vector basis if you like)
    perspective: &bevy::render::camera::PerspectiveProjection,
    viewport_size: UVec2,                        // physical target size (pixels) for correct aspect
    params: &FrameParams,
) -> f32 /*radius*/ {
    // 1) Build the desired camera *orientation* from yaw/pitch around +Y as up (typical PanOrbit)
    let (sy, cy) = params.yaw.sin_cos();
    let (sp, cp) = params.pitch.sin_cos();

    // Camera forward in world (looking direction). Convention: look along -Z in view.
    let forward = Vec3::new(cy*cp, sp, sy*cp).normalize();   // points *toward* the subject
    let right   = forward.cross(Vec3::Y).normalize();
    let up      = right.cross(forward).normalize();

    // View matrix from this yaw/pitch, *placed at the target* for now (eye position offset handled by 't')
    let view_rot = Mat3::from_cols(right, up, -forward);     // columns are camera basis (right, up, -forward)
    let view = Mat4::from_mat3(view_rot).inverse();          // rotation-only inverse

    // 2) Get AABB corners in world, then move into camera space *relative to target*
    let world_center = params.aabb.center.into();
    let half = params.aabb.half_extents.into();
    let corners = [
        Vec3::new(-half.x, -half.y, -half.z), Vec3::new( half.x, -half.y, -half.z),
        Vec3::new(-half.x,  half.y, -half.z), Vec3::new( half.x,  half.y, -half.z),
        Vec3::new(-half.x, -half.y,  half.z), Vec3::new( half.x, -half.y,  half.z),
        Vec3::new(-half.x,  half.y,  half.z), Vec3::new( half.x,  half.y,  half.z),
    ].map(|local| world_center + local);

    // shift so that target is origin, then rotate into camera basis
    let corners_cam: Vec<Vec3> = corners.iter()
        .map(|&w| view_rot * (w - params.target))
        .collect();

    // 3) FOVs and tangents
    let aspect = viewport_size.x as f32 / viewport_size.y as f32;
    let tan_v = (perspective.fov * 0.5).tan();
    let tan_h = ( (perspective.fov * 0.5).tan() * aspect ).atan().tan(); // equals tan_v * aspect but kept explicit

    // 4) Compute minimal extra dolly distance t
    let mut t_needed = 0.0_f32;
    for p in corners_cam.iter() {
        // In view space, camera looks toward -Z, so depth d = -z must be positive
        let d = (-p.z).max(1e-6);
        let need_h = (p.x.abs() / tan_h) - d;
        let need_v = (p.y.abs() / tan_v) - d;
        t_needed = t_needed.max(need_h.max(need_v));
    }

    let t = (t_needed * params.margin).max(0.0);
    t
}

// Then position your orbit camera:
fn apply_orbit(camera_transform: &mut Transform, params: &FrameParams, radius: f32) {
    let (sy, cy) = params.yaw.sin_cos();
    let (sp, cp) = params.pitch.sin_cos();
    let forward = Vec3::new(cy*cp, sp, sy*cp).normalize();

    let eye = params.target - forward * radius;
    camera_transform.translation = eye;
    camera_transform.look_at(params.target, Vec3::Y);
}
```

Hooking into bevy_panorbit_camera
	•	Set PanOrbitCamera { yaw, pitch, target, .. } to your desired angles/target.
	•	Compute radius = dolly_to_fit_fixed_fov(...).
	•	Update the component’s radius (or set transform directly if you manage the rig yourself).
	•	Important: use the actual viewport size (Camera::physical_target_size() in 0.16+) to get the right aspect, especially in a resized window or UI panel.

⸻

Tips & pitfalls
	•	Use the object’s full world-space AABB. If you only use its center + bounding sphere, you’ll get the “extra space” artifact on elongated objects.
	•	Account for near plane. If radius puts any corner in front of the near plane, bump t by a small epsilon.
	•	Margins. Add a small margin (e.g., 1–3%) so the object doesn’t kiss the edges due to float error.
	•	Consistent conventions. The math above assumes the camera looks down −Z in view space (Bevy default).
	•	Orthographic is different. For ortho, you’d scale orthographic_projection.scale (or extents) instead of dollying.

⸻

What to call it (search terms / naming in your code)
	•	“Frame / Frame Selected / Frame All” (common DCC term)
	•	“Zoom to Fit (fixed FOV)” or “Dolly-to-Fit”
	•	“Reframe to Bounds”
	•	“Screen-space crop fit” (your mental model)

If you implement the exact inequalities above, your result will behave like “drag the photo under a fixed crop and stop exactly when the whole subject is inside the crop,” which is the feel you’re after.


user: Okay it makes sense what you’re saying but will this worked on any arbitrarily panned and rotated and zoomed transform?

agent: Yes — if you interpret it the right way, this works for any arbitrarily rotated, panned, or zoomed camera pose — because the method doesn’t depend on where the camera currently is.

It only depends on:
	1.	What orientation you want (yaw/pitch).
	2.	What point you’re orbiting / framing (target).
	3.	What object’s bounds need to fit (AABB).
	4.	The perspective projection (FOV + aspect).

Then, from those, it computes the only correct distance along the camera view direction that makes the object exactly fill the frame.

⸻

Key idea

We reconstruct the camera orientation first, then solve for the correct radius.
That means your current camera position doesn’t matter at all.

It works even if your current camera is:
	•	upside down,
	•	miles away,
	•	zoomed incorrectly,
	•	or looking at something else.

Because we’re computing the target pose directly from the geometry.

⸻

Why this works regardless of the starting transform

A perspective camera can be uniquely defined by:

Parameter
Meaning
FOV (vertical)
Controls perspective / how “wide” you see
Aspect ratio
Ties horizontal coverage to vertical
Orientation (rotation)
Where the camera is pointed
Distance to target (radius)
How far along the camera’s view direction you are

Once we fix:
	•	Orientation (yaw/pitch)
	•	Target point
	•	FOV / aspect

There is exactly one camera distance that causes the object to fit.

We solve for that distance, rather than trying to adjust incrementally.

This is why it works no matter how the camera is currently positioned.

⸻

The mental picture (your “static photograph crop” metaphor)

Imagine you have a giant photograph hanging in space.
Your camera’s FOV is fixed — so the size of the crop window never changes.

You are:
	•	Rotating that crop window (yaw/pitch).
	•	Sliding the photo around so the subject is centered (target).
	•	Then sliding the camera forward/backwards so the subject exactly fills the crop.

This is exactly what the math above does.

And yes — this is not the same as:
	•	zoom += wheel_delta
	•	orbit.radius *= scale
	•	pan += mouse_drag

Those modify the frame window, whereas you want to move the underlying image.
You are correct that this is fundamentally different.

⸻

So the answer to your question:

“Will this work on any arbitrarily panned and rotated and zoomed transform?”

Yes. Because we never use the current transform.
We calculate the correct transform fresh, from geometry and FOV.

The incoming camera pose doesn’t matter at all.

user: I honestly feel as if I’ve seen a similar version of the math you’re describing as one of our experiments but it still left viewport space unfilled. Hard for you to speculate why I know

agent:

Totally fair. And yes, you definitely have seen something like this math before.
And yes, it’s very common for it to still leave empty space.
There are only three reasons that happens. If even one is off, the framing will look almost correct but leave slack around the edges.

Let’s go through them clearly and simply.

⸻

1. Bounding sphere instead of bounding box

If the math used radius = object_bounding_sphere, that always overestimates.

Bounding sphere fits in all directions, so it guarantees the object is fully visible, but that guarantee comes at the cost of unused space on the long dimension.

This is the most common failure.

Symptom
	•	Objects that are flat or wide leave empty space top or bottom.
	•	Tall skinny objects leave empty space left or right.

Fix

Must use all 8 corners of the AABB in camera space.

⸻

2. Horizontal vs Vertical FOV mismatch

If the code uses vertical FOV only, but compares against horizontal extents, it will be wrong unless the viewport is square.

This is the notorious aspect coupling mistake.

Symptom
	•	Looks good at some window sizes.
	•	Looks wrong after resizing or in widescreen.

Fix

Use both:
```rust
tan_v = tan(vertical_fov / 2)
tan_h = tan_v * aspect_ratio
```

If you skip tan_h, you get unused space.

⸻

3. Camera-space depth sign convention

If the projection math assumes:

```rust
camera looks down -Z
```

but the code treats z as positive depth, the fit equations flip sign and give the illusion of being correct… but solve for a distance farther than needed.

This is extremely common across examples copied from Unity, ThreeJS, Blender, and OpenGL tutorials, because they use different conventions.

Symptom
	•	The object always fits, but never tightly.
	•	Increasing pitch makes the problem worse or better in weird ways.

Fix

Make sure you’re using:

```rust
depth d = -p.z   (must be positive)
|x| <= d * tan_h
|y| <= d * tan_v
```

So if your previous attempt looked almost right…

It was almost certainly one of these:

What went wrong
Result
Used bounding sphere
Loose frame on elongated models
Forgot to compute tan_h
Wrong fit on most aspect ratios
Used p.z instead of -p.z
Always slightly zoomed out


Gotcha—you’re describing a classic “frame to fit” (a.k.a. frame selection / view framing / zoom-to-fit) effect:
	•	Keep the camera’s perspective/FOV unchanged (so the look “feels” the same).
	•	Don’t dolly-zoom or change FOV.
	•	Move the camera along its current view direction so the chosen object (or its bounds) exactly fills the viewport—no wasted letterbox space—for whatever yaw/pitch/pan you’re at.

Below is a drop-in approach that works for any arbitrarily rotated and panned target, and plays nicely with bevy_panorbit_camera (0.17.2). It does tight framing by testing all 8 corners of the target’s world-space AABB in camera space, so elongated shapes fit snugly (this is the bit many “radius/FOV” formulas miss, which causes the extra empty space).

⸻

How it works (brief)
	1.	Pick the point you want to “orbit around” (usually the object’s center or some authored focus).
	2.	Get the 8 world corners of the object’s AABB (after all world transforms).
	3.	Transform those corners into camera space (using the camera’s current rotation & position).
	4.	With your fixed FOV and aspect, compute the minimum Z distance (along the camera’s forward axis) such that every corner would project within the view frustum:
	•	Let t_y = tan(FOV_y / 2) and t_x = t_y * aspect (assuming FOV is vertical).
	•	For each corner in camera space (x, y, z) with z > 0, that corner fits if:
	•	|x| <= z * t_x and |y| <= z * t_y.
	•	Rearranged, the distance needed to fit that corner is:
	•	z >= max( |x| / t_x, |y| / t_y ).
	•	Take the max over all corners as your required z_fit.
	5.	Place the camera so that its position is focus - forward * z_fit (keeping yaw/pitch/FOV unchanged).
	6.	If you’re using PanOrbitCamera, update its focus/target and radius to match so mouse wheel/orbit feels continuous.

This works for any arbitrary yaw/pitch/pan because we compute everything in the camera’s current space.

⸻

Why people see “extra space”

Most “quick” solutions:
	•	use a bounding sphere → wastes space on long/thin objects,
	•	fit using an axis-aligned AABB in world or object space (not camera space),
	•	use the wrong FOV (mix up vertical vs horizontal),
	•	or ignore aspect ratio.
Any of those leaves letterboxing.

⸻

Code

This example:
	•	finds an entity’s world-space AABB (using Aabb from bevy_render::primitives computed via meshes),
	•	computes the tight distance,
	•	and updates a camera (and its PanOrbitCamera component if present).

```rust
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::mesh::MeshAabb; // if you use Bevy's extracted AABBs
use bevy::render::view::Projection;
use bevy_panorbit_camera::PanOrbitCamera; // 0.17.2

#[derive(Component)]
pub struct FrameThis(pub Entity); // attach to a camera, pointing to the target entity

pub struct ViewFramingPlugin;
impl Plugin for ViewFramingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, frame_to_fit_system);
    }
}

fn frame_to_fit_system(
    mut cams: Query<(
        Entity,
        &mut Transform,
        &GlobalTransform,
        &Projection,
        Option<&mut PanOrbitCamera>,
        &FrameThis
    )>,
    targets: Query<(&GlobalTransform, Option<&Aabb>, Option<&MeshAabb>)>,
) {
    for (
        cam_entity,
        mut cam_tf,
        cam_gxf,
        proj,
        pan_orbit_opt,
        FrameThis(target_ent),
    ) in cams.iter_mut()
    {
        let Ok((target_gxf, aabb_opt, mesh_aabb_opt)) = targets.get(*target_ent) else {
            continue;
        };

        // 1) Get a world-space AABB for the target.
        // Prefer `Aabb` if you’ve stored it on the entity; otherwise MeshAabb -> Aabb via transform.
        let world_aabb = if let Some(aabb) = aabb_opt {
            aabb_in_world(*aabb, *target_gxf)
        } else if let Some(mesh_aabb) = mesh_aabb_opt {
            aabb_in_world(mesh_aabb.aabb, *target_gxf)
        } else {
            // Fallback: if you don’t have AABBs, you’ll need to compute one from your mesh data.
            continue;
        };

        // 2) Choose a focus point: AABB center works well for most cases.
        let focus = world_aabb.center;

        // 3) Compute tight Z distance (camera space) for current camera orientation/FOV/aspect.
        let (fov_y, aspect) = match proj {
            Projection::Perspective(p) => (p.fov, p.aspect_ratio),
            _ => continue, // only handling perspective here
        };

        let cam_rot = cam_gxf.rotation();
        let cam_forward = cam_rot * -Vec3::Z; // Bevy cameras look toward -Z in view space
        let cam_right   = cam_rot *  Vec3::X;
        let cam_up      = cam_rot *  Vec3::Y;

        // Move the coordinate frame so that "focus" is the origin we want to frame around.
        // We'll compute corner positions relative to a camera sitting at `focus` looking in its current direction.
        let corners = aabb_corners(&world_aabb);
        let tight_z = required_distance_along_forward(&corners, focus, cam_right, cam_up, cam_forward, fov_y, aspect);

        // 4) Set camera position so that focus fills tightly.
        let new_cam_pos = focus - cam_forward * tight_z;
        cam_tf.translation = new_cam_pos;
        // keep orientation (yaw/pitch) unchanged; ensure it still looks at focus:
        cam_tf.look_at(focus, cam_up);

        // 5) If using PanOrbitCamera, sync its target & radius so controls remain continuous.
        if let Some(mut pan) = pan_orbit_opt {
            // Field names may differ slightly across minor versions; these are the common ones:
            pan.target = focus;
            pan.radius = tight_z;
        }

        // If you use a physics/camera smoothing plugin, you may need to bypass lerps for one frame.
    }
}

/// Transform an Aabb by a GlobalTransform into world space.
fn aabb_in_world(aabb: Aabb, gxf: GlobalTransform) -> Aabb {
    // Fast conservative transform of an AABB by an arbitrary rotation+scale:
    // Build OBB corners in local, transform to world, then recompute AABB around them.
    let corners = aabb_corners_local(aabb);
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for c in corners {
        let w = gxf.transform_point(c);
        min = min.min(w);
        max = max.max(w);
    }
    Aabb::from_min_max(min, max)
}

#[inline]
fn aabb_corners_local(aabb: Aabb) -> [Vec3; 8] {
    let min = aabb.min();
    let max = aabb.max();
    [
        Vec3::new(min.x, min.y, min.z),
        Vec3::new(max.x, min.y, min.z),
        Vec3::new(min.x, max.y, min.z),
        Vec3::new(max.x, max.y, min.z),
        Vec3::new(min.x, min.y, max.z),
        Vec3::new(max.x, min.y, max.z),
        Vec3::new(min.x, max.y, max.z),
        Vec3::new(max.x, max.y, max.z),
    ]
}

#[inline]
fn aabb_corners(aabb: &Aabb) -> [Vec3; 8] {
    aabb_corners_local(*aabb)
}

/// Compute the minimal distance (along camera forward) such that all corners fit the viewport,
/// **keeping FOV and aspect fixed**.
fn required_distance_along_forward(
    world_corners: &[Vec3; 8],
    focus: Vec3,
    cam_right: Vec3,
    cam_up: Vec3,
    cam_forward: Vec3,
    fov_y: f32,
    aspect: f32,
) -> f32 {
    // Vertical FOV is assumed.
    let t_y = (fov_y * 0.5).tan();
    let t_x = t_y * aspect;

    let mut needed = 0.0f32;

    for &corner_w in world_corners.iter() {
        // Express corner in a camera-centered basis whose origin is the focus point.
        let r = corner_w - focus;

        // Camera-space coordinates relative to a camera located AT the focus (just for measuring extents).
        let x = r.dot(cam_right);
        let y = r.dot(cam_up);
        let z = r.dot(cam_forward).max(0.0) + 1e-4; // ensure positive; add epsilon to avoid div-by-zero

        // Distance required so this corner fits both horizontally and vertically:
        // we need z' >= max(|x|/t_x, |y|/t_y). Since z' = z + d (extra camera pull-back),
        // but we place the camera at focus - forward * Z_total, equivalently the needed total distance is:
        let z_req = f32::max(x.abs() / t_x, y.abs() / t_y);
        needed = needed.max(z_req);
    }

    // Ensure we don't clip the near plane. If your camera near is, say, 0.1, you can add a margin here if desired.
    needed
}
```
