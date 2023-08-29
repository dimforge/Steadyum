use crate::selection::Selection;
use bevy::prelude::*;

pub fn handle_keyboard_inputs(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    selection: Query<(Entity, &Selection)>,
) {
    for (entity, selection) in selection.iter() {
        if selection.selected() {
            if keys.just_released(KeyCode::Delete) {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
