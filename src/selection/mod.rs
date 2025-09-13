//! Editor selection module.

pub mod common_conditions;

use bevy::{
    ecs::entity::{Entities, EntitySet, EntitySetIterator, FromEntitySetIterator, UniqueEntityVec},
    prelude::*,
};
use bevy_egui::input::EguiWantsInput;

use crate::collider_tools::ControlPointEntity;
use crate::utils::DragCancelClick;

/// Editor selection plugin.
#[derive(Default)]
pub struct SelectionPlugin;

impl SelectionPlugin {
    /// Check that all required dependencies are registered
    fn check_dependencies(&self, app: &App) {
        // Check for CoreUtilsPlugin (provides DragCancelClick)
        if !app.is_plugin_added::<crate::utils::CoreUtilsPlugin>() {
            panic!(
                "SelectionPlugin requires CoreUtilsPlugin to be registered.\n\
                 This plugin provides the DragCancelClick event used by selection systems.\n\
                 Please add CoreUtilsPlugin to your app before adding SelectionPlugin:\n\
                 app.add_plugins(CoreUtilsPlugin);"
            );
        }

        info!("SelectionPlugin: All plugin dependencies verified");
    }
}

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        // Check required dependencies
        self.check_dependencies(app);

        app.init_resource::<EditorSelection>()
            .add_systems(
                PostUpdate,
                (
                    remove_entity_from_selection_if_despawned,
                    log_selection_changes,
                ),
            )
            .add_observer(selection_handler);
    }
}

fn selection_handler(
    mut trigger: Trigger<Pointer<DragCancelClick>>,
    selectable_query: Query<(), With<Selectable>>,
    camera_query: Query<(), With<Camera>>,
    control_point_query: Query<(), With<ControlPointEntity>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<EditorSelection>,
    current_mode: Res<State<crate::collider_tools::ToolMode>>,
    egui_wants_input_resource: Res<EguiWantsInput>,
) {
    // Debug: Log all selection attempts
    info!(
        "Selection handler triggered for entity: {:?}",
        trigger.target()
    );
    info!(
        "Entity is selectable: {}",
        selectable_query.contains(trigger.target())
    );
    info!(
        "Entity is control point: {}",
        control_point_query.contains(trigger.target())
    );
    info!(
        "Entity is camera: {}",
        camera_query.contains(trigger.target())
    );
    info!("Current mode: {:?}", current_mode.get());
    if egui_wants_input_resource.wants_any_input() {
        return;
    }

    if trigger.button != PointerButton::Primary {
        return;
    }

    let target = trigger.target();

    // Ignore control point entities to prevent selection conflicts
    if control_point_query.contains(target) {
        trigger.propagate(false); // Prevent event propagation
        return;
    }

    if selectable_query.contains(target) {
        trigger.propagate(false);
        let shift = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
        if shift {
            info!("Shift pressed, toggling selection for entity {:?}", target);
            selection.toggle(target);
        } else {
            info!("Setting selection to entity {:?}", target);
            selection.set(target);
        }
    } else if camera_query.contains(target) {
        // Clicked on camera (background) - clear selection if not holding shift
        // Don't clear selection in Edit mode to avoid interrupting editing operations
        let shift_pressed = keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        if !shift_pressed && *current_mode.get() != crate::collider_tools::ToolMode::Edit {
            info!("Clicked on camera background, clearing selection");
            selection.clear();
        } else if *current_mode.get() == crate::collider_tools::ToolMode::Edit {
            info!("In Edit mode, preserving selection on background click");
        }
    } else {
        info!("Target entity {:?} is not selectable or camera", target);
    }
}

/// The currently selected entities in the scene.
#[derive(Resource, Default)]
pub struct EditorSelection(UniqueEntityVec);

impl EditorSelection {
    /// Toggle selection for an entity.
    pub fn toggle(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        if !self.remove(entity) {
            // SAFETY: The preceding call to self.remove ensures the entity is not present.
            #[expect(unsafe_code)]
            unsafe {
                self.0.push(entity);
            }
        }
    }

    /// Set the selection to an entity, making it the primary selection.
    pub fn set(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        *self = EditorSelection::from_iter([entity]);
    }

    /// Add an entity to the selection, making it the primary selection.
    ///
    /// If the entity was already part of the selection it will be made the primary selection.
    pub fn add(&mut self, entity: Entity) {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        self.remove(entity);
        // SAFETY: The preceding call to self.remove ensures the entity is not present.
        #[expect(unsafe_code)]
        unsafe {
            self.0.push(entity);
        }
    }

    /// Remove an entity from the selection if present. Returns `true` if the entity was removed.
    pub fn remove(&mut self, entity: Entity) -> bool {
        debug_assert_ne!(entity, Entity::PLACEHOLDER);
        if let Some(position) = self.0.iter().position(|selected| *selected == entity) {
            self.0.remove(position);
            true
        } else {
            false
        }
    }

    /// Empty the selection.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Check whether the selection contains a given entity.
    pub fn contains(&self, entity: Entity) -> bool {
        self.0.contains(&entity)
    }

    /// The last entity in the selection.
    pub fn primary(&self) -> Option<Entity> {
        self.0.last().copied()
    }

    /// Returns an iterator over all entities in the selection in the order they were selected.
    pub fn iter(&self) -> impl EntitySetIterator<Item = Entity> {
        self.0.iter().copied()
    }

    /// Check if the selection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the number of selected entities.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl FromIterator<Entity> for EditorSelection {
    fn from_iter<T: IntoIterator<Item = Entity>>(iter: T) -> Self {
        EditorSelection(UniqueEntityVec::from_iter(iter))
    }
}

impl FromEntitySetIterator<Entity> for EditorSelection {
    fn from_entity_set_iter<T: EntitySet<Item = Entity>>(set_iter: T) -> Self {
        EditorSelection(UniqueEntityVec::from_entity_set_iter(set_iter))
    }
}

/// Marker component for selectable entities.
#[derive(Component, Default, Clone)]
pub struct Selectable;

/// This system removes entities from the [`EditorSelection`] when they are despawned.
pub fn remove_entity_from_selection_if_despawned(
    mut selection: ResMut<EditorSelection>,
    entities: &Entities,
) {
    // Avoid triggering change detection every frame.
    if selection.0.iter().any(|entity| !entities.contains(*entity)) {
        selection.0.retain(|entity| entities.contains(*entity));
    }
}

/// This system logs selection changes when they occur.
fn log_selection_changes(selection: Res<EditorSelection>) {
    if selection.is_changed() {
        let selected_entities: Vec<Entity> = selection.iter().collect();
        let primary_entity = selection.primary();

        if selected_entities.is_empty() {
            info!("Selection cleared");
        } else if selected_entities.len() == 1 {
            info!("Single entity selected: {:?}", selected_entities[0]);
        } else {
            info!(
                "Multiple entities selected: {:?}, primary: {:?}",
                selected_entities, primary_entity
            );
        }
    }
}
