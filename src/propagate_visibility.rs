use bevy::prelude::*;

// TODO: we can likely get rid of this once https://github.com/bevyengine/bevy/pull/2087 and
//       https://github.com/bevyengine/bevy/issues/838 are resolved.
// Propagate `Visible::is_visible` recursively from a component to its children.
pub fn visible_propagate_system(
    mut root_query: Query<(Entity, Option<&Children>, &Visibility), Without<Parent>>,
    mut visible_query: ParamSet<(
        Query<&mut Visibility, With<Parent>>,
        Query<Entity, Changed<Visibility>>,
    )>,
    children_query: Query<Option<&Children>, (With<Parent>, With<Visibility>)>,
) {
    for (entity, children, visible) in root_query.iter_mut() {
        let changed = visible_query.p1().get(entity).is_ok();

        if let Some(children) = children {
            for child in children.iter() {
                propagate_recursive(
                    *visible == Visibility::Visible,
                    &mut visible_query,
                    &children_query,
                    *child,
                    changed,
                );
            }
        }
    }
}

fn propagate_recursive(
    parent_is_visible: bool,
    visible_query: &mut ParamSet<(
        Query<&mut Visibility, With<Parent>>,
        Query<Entity, Changed<Visibility>>,
    )>,
    children_query: &Query<Option<&Children>, (With<Parent>, With<Visibility>)>,
    entity: Entity,
    mut changed: bool,
) {
    changed |= visible_query.p1().get(entity).is_ok();

    let visible = {
        if let Ok(mut visible) = visible_query.p0().get_mut(entity) {
            if changed {
                visible.is_visible = parent_is_visible;
            }

            visible.is_visible
        } else {
            return;
        }
    };

    if let Ok(Some(children)) = children_query.get(entity) {
        for child in children.iter() {
            propagate_recursive(visible, visible_query, children_query, *child, changed);
        }
    }
}
