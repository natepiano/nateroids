mod drawing;
mod geometry;
mod intersection;

pub use drawing::PortalActorKind;
pub(super) use drawing::draw_portal;
pub(super) use geometry::calculate_portal_face_count;
