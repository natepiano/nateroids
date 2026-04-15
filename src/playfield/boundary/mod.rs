mod gizmo;
mod portal_render;
mod teleport;

use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
pub use gizmo::BoundaryVolume;

use super::constants::BOUNDARY_CELL_COUNT;
use super::constants::BOUNDARY_GRID_LINE_WIDTH;
use super::constants::BOUNDARY_OUTER_LINE_WIDTH;
use super::constants::BOUNDARY_SCALAR;
use super::portals::Portal;
use super::portals::PortalGizmo;
use super::types::BoundaryGizmo;
use super::types::GridFlashAnimation;
use super::types::GridGizmo;
use super::types::PortalActorKind;
use crate::input::InspectBoundarySwitch;
use crate::orientation::CameraOrientation;
use crate::state::GameState;
use crate::switches;
use crate::switches::Switch;
use crate::switches::Switches;

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
                    .run_if(in_state(GameState::Splash).or(in_state(GameState::InGame))),
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

/// defines
#[derive(Resource, Reflect, InspectorOptions, Clone, Debug)]
#[reflect(Resource, InspectorOptions)]
pub struct Boundary {
    pub cell_count:          UVec3,
    pub grid_color:          Color,
    pub outer_color:         Color,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub grid_line_width:     f32,
    #[inspector(min = 0.1, max = 40.0, display = NumberDisplay::Slider)]
    pub exterior_line_width: f32,
    #[inspector(min = 50., max = 300., display = NumberDisplay::Slider)]
    pub exterior_scalar:     f32,
}

impl Default for Boundary {
    fn default() -> Self {
        Self {
            cell_count:          BOUNDARY_CELL_COUNT,
            // Start with alpha 0 - will be faded in during splash screen
            grid_color:          Color::from(tailwind::BLUE_500).with_alpha(0.0),
            outer_color:         Color::from(tailwind::BLUE_500).with_alpha(0.0),
            grid_line_width:     BOUNDARY_GRID_LINE_WIDTH,
            exterior_line_width: BOUNDARY_OUTER_LINE_WIDTH,
            exterior_scalar:     BOUNDARY_SCALAR,
        }
    }
}

impl Boundary {
    pub fn longest_diagonal(&self) -> f32 {
        let boundary_scale = self.scale();
        let x = boundary_scale.x;
        let y = boundary_scale.y;
        let z = boundary_scale.z;
        // FMA optimization (faster + more precise): (x² + y² + z²).sqrt()
        z.mul_add(z, y.mul_add(y, x.mul_add(x, 0.0))).sqrt()
    }

    pub fn max_missile_distance(&self) -> f32 {
        let boundary_scale = self.scale();
        boundary_scale.x.max(boundary_scale.y).max(boundary_scale.z)
    }

    pub fn scale(&self) -> Vec3 { self.exterior_scalar * self.cell_count.as_vec3() }

    /// Returns the 8 corner points of the boundary as a fixed-size array
    pub fn corners(&self) -> [Vec3; 8] {
        let grid_size = self.scale();
        let half_size = grid_size / 2.0;
        [
            Vec3::new(-half_size.x, -half_size.y, -half_size.z),
            Vec3::new(half_size.x, -half_size.y, -half_size.z),
            Vec3::new(-half_size.x, half_size.y, -half_size.z),
            Vec3::new(half_size.x, half_size.y, -half_size.z),
            Vec3::new(-half_size.x, -half_size.y, half_size.z),
            Vec3::new(half_size.x, -half_size.y, half_size.z),
            Vec3::new(-half_size.x, half_size.y, half_size.z),
            Vec3::new(half_size.x, half_size.y, half_size.z),
        ]
    }

    pub(super) fn draw_portal(
        gizmos: &mut Gizmos<PortalGizmo>,
        portal: &Portal,
        color: Color,
        resolution: u32,
        orientation: &CameraOrientation,
        actor_kind: PortalActorKind,
        transform: &Transform,
    ) {
        portal_render::draw_portal(
            gizmos,
            portal,
            color,
            resolution,
            orientation,
            actor_kind,
            transform,
        );
    }

    pub(super) fn calculate_portal_face_count(portal: &Portal, transform: &Transform) -> usize {
        portal_render::calculate_portal_face_count(portal, transform)
    }
}
