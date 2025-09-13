//! Enhanced selection module for collider creation
//!
//! This module provides comprehensive selection functionality for colliders,
//! integrating with both EditorSelection and physics picking systems.
//! It follows the design patterns from picking and selection modules.
//!
//! This implementation supports multiple selection modes:
//! - Single selection: Click to select one collider
//! - Multi selection: Shift+Click to add/remove from selection
//! - Toggle selection: Ctrl+Click to toggle individual selections
//!
//! This implementation is focused on 2D functionality only.

use crate::transform_gizmos::InternalGizmoCamera;
use crate::{ColliderType, ControlPointEntity, TransformGizmoSettings};
use avian2d::math::{AdjustPrecision, AsF32};
use avian2d::prelude::*;
use avian2d::schedule::PhysicsSchedulePlugin;
use bevy::app::App;
use bevy::picking::{
    PickSet,
    backend::{HitData, PointerHits},
};
use bevy::prelude::*;

use super::{ColliderCreationState, ToolMode};
use crate::selection::{EditorSelection, Selectable};
use crate::utils::DragCancelClick;
use bevy_egui::input::{EguiWantsInput, egui_wants_any_input};

/// Ensure collider picking hits are processed after gizmo mesh picking hits
const COLLIDER_POINTER_HITS_ORDER_OFFSET: f32 = 10_000.0;

/// Enhanced collider selection plugin that integrates with physics picking
#[derive(Default)]
pub struct ColliderSelectionPlugin;

impl ColliderSelectionPlugin {
    /// Check that all required dependencies are registered
    fn check_dependencies(&self, app: &App) {
        // Check for bevy_picking plugins
        if !app.is_plugin_added::<bevy::picking::PickingPlugin>() {
            panic!(
                "ColliderSelectionPlugin requires bevy::picking::PickingPlugin to be registered.\n\
                 Please add PickingPlugins to your app before adding ColliderSelectionPlugin:\n\
                 app.add_plugins(PickingPlugins::default());"
            );
        }

        // Check if avian2d physics plugins are registered by checking for a core physics plugin
        if !app.is_plugin_added::<PhysicsSchedulePlugin>() {
            panic!(
                "ColliderSelectionPlugin requires avian2d physics plugins to be registered.\n\
                 Please add PhysicsPlugins to your app before adding ColliderSelectionPlugin:\n\
                 app.add_plugins(PhysicsPlugins::default());"
            );
        }

        // Check for SelectionPlugin (provides EditorSelection resource)
        // Note: SelectionPlugin will check its own dependencies (CoreUtilsPlugin)
        if !app.is_plugin_added::<crate::selection::SelectionPlugin>() {
            panic!(
                "ColliderSelectionPlugin requires SelectionPlugin to be registered.\n\
                 This plugin provides the EditorSelection resource.\n\
                 Please add SelectionPlugin to your app before adding ColliderSelectionPlugin:\n\
                 app.add_plugins(SelectionPlugin);"
            );
        }

        // Check for InteractionStandardsPlugin (provides interaction standards)
        if !app.is_plugin_added::<crate::interaction_standards::InteractionStandardsPlugin>() {
            panic!(
                "ColliderSelectionPlugin requires InteractionStandardsPlugin to be registered.\n\
                 This plugin provides interaction standards and visual feedback.\n\
                 Please add InteractionStandardsPlugin to your app before adding ColliderSelectionPlugin:\n\
                 app.add_plugins(InteractionStandardsPlugin);"
            );
        }

        info!("ColliderSelectionPlugin: All plugin dependencies verified");
    }
}

impl Plugin for ColliderSelectionPlugin {
    fn build(&self, app: &mut App) {
        // Check required dependencies
        self.check_dependencies(app);

        app.init_resource::<ColliderSelectionSettings>()
            // Add mode transition systems
            .add_systems(OnEnter(ToolMode::Select), on_enter_select_mode)
            .add_systems(OnExit(ToolMode::Select), on_exit_select_mode)
            // Core selection systems
            .add_systems(
                PreUpdate,
                update_collider_selection_hits.in_set(PickSet::Backend),
            )
            // Keyboard shortcut systems (run in all modes)
            .add_systems(
                Update,
                (handle_selection_input, handle_selection_mode_input)
                    .run_if(in_state(ToolMode::Select).and(not(egui_wants_any_input))),
            )
            // Observers for all modes
            .add_observer(handle_collider_selection)
            .register_type::<ColliderSelectionSettings>()
            .register_type::<ColliderSelectionFilter>();
    }
}

/// Settings for collider selection behavior
#[derive(Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct ColliderSelectionSettings {
    /// Enable hover effects for colliders
    pub enable_hover_effects: bool,
    /// Highlight color for hovered colliders
    pub hover_color: Color,
    /// Selection color for selected colliders
    pub selection_color: Color,
    /// Control point hover radius multiplier
    pub control_point_hover_radius: f32,
    /// Enable multi-selection with Shift key
    pub enable_multi_selection: bool,
}

impl Default for ColliderSelectionSettings {
    fn default() -> Self {
        Self {
            enable_hover_effects: true,
            hover_color: Color::srgba(1.0, 1.0, 0.0, 0.3), // Yellow with transparency
            selection_color: Color::srgb(0.2, 0.8, 1.0),   // Cyan
            control_point_hover_radius: 1.5,
            enable_multi_selection: true, // Enable multi-selection by default
        }
    }
}

/// Filter component for collider selection
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct ColliderSelectionFilter(pub SpatialQueryFilter);

impl ColliderSelectionFilter {
    /// Create a filter that only selects created colliders
    pub fn created_colliders_only() -> Self {
        Self(SpatialQueryFilter::default())
    }

    /// Create a filter with specific layer mask
    pub fn from_mask(mask: impl Into<LayerMask>) -> Self {
        Self(SpatialQueryFilter::default().with_mask(mask))
    }

    /// Add excluded entities to the filter
    pub fn with_excluded_entities(mut self, entities: impl IntoIterator<Item = Entity>) -> Self {
        self.0 = self.0.with_excluded_entities(entities);
        self
    }
}

/// Enhanced collider selection handler with physics picking integration
///
/// This observer handles DragCancelClick events for colliders, providing
/// sophisticated selection behavior similar to the main selection system.
fn handle_collider_selection(
    mut trigger: Trigger<Pointer<DragCancelClick>>,
    mut selection: ResMut<EditorSelection>,
    keyboard: Res<ButtonInput<KeyCode>>,
    selectable_query: Query<(), With<Selectable>>,
    created_collider_query: Query<(), With<ColliderType>>,
    camera_query: Query<(), With<Camera>>,
    control_point_query: Query<(), With<ControlPointEntity>>,
    settings: Res<ColliderSelectionSettings>,
    egui_wants_input_resource: Res<EguiWantsInput>,
) {
    if egui_wants_input_resource.wants_any_input() {
        return;
    }

    info!(
        "ColliderSelection: DragCancelClick event triggered on entity {:?} with button {:?}",
        trigger.target(),
        trigger.button
    );

    // Only handle primary button clicks
    if trigger.button != PointerButton::Primary {
        info!("ColliderSelection: Ignoring non-primary button click");
        return;
    }

    let target = trigger.target();

    // Ignore control point entities to prevent selection conflicts
    if control_point_query.contains(target) {
        info!(
            "ColliderSelection: Target entity {:?} is a control point, ignoring selection",
            target
        );
        trigger.propagate(false); // Prevent event propagation
        return;
    }

    // Check if target is a selectable created collider
    if selectable_query.contains(target) && created_collider_query.contains(target) {
        info!(
            "ColliderSelection: Target entity {:?} is a selectable collider, processing selection",
            target
        );
        trigger.propagate(false);

        // Handle multi-selection and toggle modifiers
        let shift_pressed = settings.enable_multi_selection
            && keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        if shift_pressed {
            // Shift+Click: add to selection (multi-selection)
            info!(
                "ColliderSelection: Shift pressed, adding entity {:?} to selection",
                target
            );
            selection.toggle(target);
        } else {
            // Regular click: set selection to this entity only
            info!(
                "ColliderSelection: Setting selection to entity {:?}",
                target
            );
            selection.set(target);
        }
    } else if camera_query.contains(target) {
        // Clicked on camera (background) - clear selection if not holding shift or ctrl
        // Don't clear selection in Edit mode to avoid interrupting editing operations
        let shift_pressed = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);

        if !shift_pressed {
            info!("ColliderSelection: Clicked on camera background, clearing selection");
            selection.clear();
        }
    } else {
        info!(
            "ColliderSelection: Target entity {:?} is not a selectable collider",
            target
        );
    }
}

/// Update collider selection hits using physics picking (2D only)
///
/// This system integrates with the physics picking backend to provide
/// accurate collision detection for collider selection in 2D space.
pub fn update_collider_selection_hits(
    picking_cameras: Query<
        (&Camera, Option<&ColliderSelectionFilter>),
        Without<InternalGizmoCamera>,
    >,
    ray_map: Res<bevy::picking::backend::ray::RayMap>,
    pickables: Query<&bevy::picking::Pickable>,
    created_colliders: Query<&ColliderType>,
    spatial_query: SpatialQuery,
    mut output_events: EventWriter<PointerHits>,
) {
    for (&ray_id, &ray) in ray_map.map.iter() {
        let Ok((camera, selection_filter)) = picking_cameras.get(ray_id.camera) else {
            continue;
        };

        if !camera.is_active {
            continue;
        }

        let default_filter = SpatialQueryFilter::DEFAULT;
        let filter = selection_filter
            .as_ref()
            .map(|f| &f.0)
            .unwrap_or(&default_filter);

        // 2D point intersection for collider selection
        let mut hits: Vec<(Entity, HitData)> = vec![];

        spatial_query.point_intersections_callback(
            ray.origin.truncate().adjust_precision(),
            filter,
            |entity| {
                // Check if entity is a created collider and pickable
                let is_created_collider = created_colliders.get(entity).is_ok();
                let is_pickable = pickables
                    .get(entity)
                    .map(|p| *p != bevy::picking::Pickable::IGNORE)
                    .unwrap_or(true);

                if is_created_collider && is_pickable {
                    hits.push((
                        entity,
                        HitData::new(ray_id.camera, 0.0, Some(ray.origin.f32()), None),
                    ));
                }

                true
            },
        );

        if !hits.is_empty() {
            // Ensure collider hits are processed AFTER gizmo mesh picking hits
            let order = camera.order as f32 - COLLIDER_POINTER_HITS_ORDER_OFFSET;
            output_events.write(PointerHits::new(ray_id.pointer, hits, order));
        }
    }
}

// ===== SELECTION MODE INPUT SYSTEMS =====

/// Unified selection input handler that supports keyboard shortcuts
/// Note: Individual collider selection is handled by the observer system
pub fn handle_selection_input(
    mut selection: ResMut<EditorSelection>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    collider_query: Query<Entity, (With<Selectable>, With<ColliderType>)>,
) {
    // Handle Ctrl+A to select all colliders
    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && keyboard_input.just_pressed(KeyCode::KeyA)
    {
        let all_colliders: Vec<Entity> = collider_query.iter().collect();
        let count = all_colliders.len();
        if !all_colliders.is_empty() {
            selection.clear();
            for entity in all_colliders {
                selection.add(entity);
            }
            info!("Selected all {} colliders", count);
        }
    }

    // Handle ESC to clear selection
    if keyboard_input.just_pressed(KeyCode::Escape) {
        selection.clear();
        info!("Selection cleared via ESC key");
    }
}

/// Handle selection mode input (keyboard shortcuts and deletion)
pub fn handle_selection_mode_input(
    mut commands: Commands,
    mut state: ResMut<ColliderCreationState>,
    mut selection: ResMut<EditorSelection>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Handle Delete key to remove selected colliders
    if keyboard_input.just_pressed(KeyCode::Delete)
        || keyboard_input.just_pressed(KeyCode::Backspace)
    {
        let selected_entities: Vec<Entity> = selection.iter().collect();
        let count = selected_entities.len();
        if !selected_entities.is_empty() {
            for entity in selected_entities {
                commands.entity(entity).despawn();
                state.created_colliders.retain(|&e| e != entity);
            }
            selection.clear();
            info!("Deleted {} selected colliders", count);
        }
    }
}

// State transition systems for CreationMode isolation
/// System called when entering Select mode
pub(super) fn on_enter_select_mode(
    mut state: ResMut<ColliderCreationState>,
    mut gizmo_settings: ResMut<TransformGizmoSettings>,
) {
    info!("Entering Select mode");
    // Clear any ongoing creation state
    state.preview_collider = None;
    state.triangle_creation_step = None;
    state.triangle_base_edge = None;
    // Enable gizmo settings and ensure gizmo is visible
    gizmo_settings.enabled = true;
}

/// System called when exiting Select mode
pub(super) fn on_exit_select_mode(
    mut gizmo_settings: ResMut<TransformGizmoSettings>,
    mut gizmo_query: Query<&mut Visibility, With<crate::TransformGizmo>>,
) {
    info!("Exiting Select mode");

    // Disable gizmo settings and hide the gizmo
    gizmo_settings.enabled = false;

    // Hide the gizmo when exiting select mode
    if let Ok(mut visibility) = gizmo_query.single_mut() {
        *visibility = Visibility::Hidden;
    }
}
