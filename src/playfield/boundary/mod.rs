mod constants;
mod gizmo;
mod portal_render;

use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use constants::BOUNDARY_EXTERIOR_LINE_WIDTH_MAX;
use constants::BOUNDARY_EXTERIOR_LINE_WIDTH_MIN;
use constants::BOUNDARY_EXTERIOR_SCALAR_MAX;
use constants::BOUNDARY_EXTERIOR_SCALAR_MIN;
use constants::BOUNDARY_GRID_LINE_WIDTH_MAX;
use constants::BOUNDARY_GRID_LINE_WIDTH_MIN;
use gizmo::BoundaryGizmo;
pub(crate) use gizmo::BoundaryVolume;
pub(crate) use gizmo::GridFlash;
use gizmo::GridFlashAnimation;
use gizmo::GridGizmo;
pub(crate) use portal_render::PortalActorKind;
pub(crate) use portal_render::calculate_portal_face_count;
pub(crate) use portal_render::draw_portal;

use super::constants::BOUNDARY_CELL_COUNT;
use super::constants::BOUNDARY_COLOR;
use super::constants::BOUNDARY_GRID_LINE_WIDTH;
use super::constants::BOUNDARY_OUTER_LINE_WIDTH;
use super::constants::BOUNDARY_SCALAR;
use super::constants::BOUNDARY_START_ALPHA;
use super::portals::Portal;
use super::portals::PortalGizmo;
use crate::input::InspectBoundarySwitch;
use crate::orientation::CameraOrientation;
use crate::state::GameState;
use crate::switches;
use crate::switches::Switch;

event!(BoundaryInspectorEvent);

pub(super) struct BoundaryPlugin;

impl Plugin for BoundaryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Boundary>()
            .init_gizmo_group::<GridGizmo>()
            .init_gizmo_group::<BoundaryGizmo>()
            .add_plugins(
                ResourceInspectorPlugin::<Boundary>::default()
                    .run_if(switches::is_switch_on(Switch::InspectBoundary)),
            )
            .add_systems(Startup, gizmo::spawn_boundary_volume)
            .add_systems(Update, gizmo::apply_boundary_settings)
            .add_systems(Update, gizmo::sync_boundary_volume)
            .add_systems(
                Update,
                gizmo::draw_boundary
                    .run_if(in_state(GameState::Splash).or_else(in_state(GameState::InGame))),
            )
            .add_systems(Update, gizmo::fade_boundary_in)
            .add_systems(
                Update,
                gizmo::detect_cell_count_change.run_if(in_state(GameState::InGame)),
            )
            .add_systems(
                Update,
                gizmo::animate_grid_flash
                    .run_if(resource_exists::<GridFlashAnimation>)
                    .after(gizmo::fade_boundary_in),
            )
            .add_observer(gizmo::start_boundary_fade)
            .add_observer(gizmo::on_grid_flash);
        bind_action_switch!(
            app,
            InspectBoundarySwitch,
            BoundaryInspectorEvent,
            Switch::InspectBoundary
        );
    }
}

/// Inspector-controlled `Boundary` resource for grid and exterior dimensions.
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub(crate) struct Boundary {
    pub(crate) cell_count:          UVec3,
    pub(crate) grid_color:          Color,
    pub(crate) outer_color:         Color,
    #[inspector(
        min = BOUNDARY_GRID_LINE_WIDTH_MIN,
        max = BOUNDARY_GRID_LINE_WIDTH_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) grid_line_width:     f32,
    #[inspector(
        min = BOUNDARY_EXTERIOR_LINE_WIDTH_MIN,
        max = BOUNDARY_EXTERIOR_LINE_WIDTH_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) exterior_line_width: f32,
    #[inspector(
        min = BOUNDARY_EXTERIOR_SCALAR_MIN,
        max = BOUNDARY_EXTERIOR_SCALAR_MAX,
        display = NumberDisplay::Slider
    )]
    pub(crate) exterior_scalar:     f32,
}

impl Boundary {
    pub(crate) fn longest_diagonal(&self) -> f32 {
        let boundary_scale = self.scale();
        let x = boundary_scale.x;
        let y = boundary_scale.y;
        let z = boundary_scale.z;
        // FMA optimization (faster + more precise): (x² + y² + z²).sqrt()
        z.mul_add(z, y.mul_add(y, x.mul_add(x, 0.0))).sqrt()
    }

    pub(crate) fn max_missile_distance(&self) -> f32 {
        let boundary_scale = self.scale();
        boundary_scale.x.max(boundary_scale.y).max(boundary_scale.z)
    }

    pub(crate) fn scale(&self) -> Vec3 { self.exterior_scalar * self.cell_count.as_vec3() }

    pub(super) fn draw_portal(
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        camera_orientation: &CameraOrientation,
        portal_actor_kind: PortalActorKind,
        transform: &Transform,
    ) {
        draw_portal(
            gizmos,
            portal,
            color,
            resolution,
            camera_orientation,
            portal_actor_kind,
            transform,
        );
    }

    pub(super) fn calculate_portal_face_count(portal: &Portal, transform: &Transform) -> usize {
        calculate_portal_face_count(portal, transform)
    }
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            cell_count:          BOUNDARY_CELL_COUNT,
            // Start with alpha 0 - will be faded in during splash screen
            grid_color:          BOUNDARY_COLOR.with_alpha(BOUNDARY_START_ALPHA),
            outer_color:         BOUNDARY_COLOR.with_alpha(BOUNDARY_START_ALPHA),
            grid_line_width:     BOUNDARY_GRID_LINE_WIDTH,
            exterior_line_width: BOUNDARY_OUTER_LINE_WIDTH,
            exterior_scalar:     BOUNDARY_SCALAR,
        }
    }
}
