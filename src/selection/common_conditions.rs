use bevy::prelude::*;

use super::EditorSelection;

/// True if the primary [`EditorSelection`] changed.
pub fn primary_selection_changed(
    mut cache: Local<Option<Entity>>,
    selection: Res<EditorSelection>,
) -> bool {
    let changed = *cache != selection.primary();
    *cache = selection.primary();
    changed
}
