use bevy::prelude::*;

use super::Nateroid;
use super::spaceship::Spaceship;

pub struct SpaceshipDiagnosticsPlugin;

impl Plugin for SpaceshipDiagnosticsPlugin {
    fn build(&self, _app: &mut App) {}
}

fn debug_spaceship_visibility(
    spaceship_query: Query<
        (
            Entity,
            &Transform,
            &InheritedVisibility,
            &ViewVisibility,
            &Visibility,
            Option<&Children>,
        ),
        With<Spaceship>,
    >,
    nateroid_query: Query<
        (Entity, &InheritedVisibility, &ViewVisibility, &Visibility),
        With<Nateroid>,
    >,
    children_query: Query<(Entity, &InheritedVisibility, &ViewVisibility, &Visibility)>,
) {
    for (entity, transform, inherited_vis, view_vis, vis, children) in spaceship_query.iter() {
        error!(
            "Spaceship Entity {entity:?} at {:.2?}:\n  \
             Visibility: {vis:?}\n  \
             InheritedVisibility: {inherited_vis:?}\n  \
             ViewVisibility: {view_vis:?}",
            transform.translation
        );

        if let Some(children) = children {
            for child_entity in children.iter() {
                if let Ok((child_e, child_inherited, child_view, child_vis)) =
                    children_query.get(child_entity)
                {
                    error!(
                        "  Child {child_e:?}:\n    \
                         Visibility: {child_vis:?}\n    \
                         InheritedVisibility: {child_inherited:?}\n    \
                         ViewVisibility: {child_view:?}"
                    );
                }
            }
        }
    }

    // Compare with a nateroid
    if let Some((entity, inherited_vis, view_vis, vis)) = nateroid_query.iter().next() {
        error!(
            "Nateroid Entity {entity:?} (for comparison):\n  \
             Visibility: {vis:?}\n  \
             InheritedVisibility: {inherited_vis:?}\n  \
             ViewVisibility: {view_vis:?}"
        );
    }
}
