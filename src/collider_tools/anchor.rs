//! Anchor creation and manipulation tools
//!
//! Provides functionality for creating, dragging, and managing anchor points on colliders.
//!
//! ## Important Concepts
//!
//! ### Local Anchor Position
//! Anchor points are defined by their `local_anchor_position`, which is relative to the
//! collider's transform origin (transform.position), NOT the center of mass.
//!
//! ### Origin vs Center of Mass
//! - **Transform Origin**: The position of the collider entity (transform.position).
//!   This is what users see and can manipulate directly.
//! - **Center of Mass**: Automatically computed by Avian based on collider shape.
//!   Users cannot directly manipulate center of mass.
//!
//! When creating joints, Avian automatically handles the conversion from local anchor
//! positions (relative to origin) to constraint anchor positions (relative to center of mass).
//!
//! ## Built-in Relationship System
//! This module uses Bevy's built-in parent-child relationship system:
//! - `ChildOf(Entity)` - indicates which collider this anchor belongs to
//! - `Children` - automatically maintained list of child entities (anchors) for a collider

use super::debug_render::anchor::AnchorPoint;
use super::debug_render::joint::{AnchorUsedBy, JointConfig, VisualizedBy};
use super::joint::{delete_joints_for_anchor, regenerate_joints_for_anchor_with_viz};
use super::utils::{
    calculate_anchor_world_position_from_anchor, create_anchor_at_position,
    get_mouse_world_position,
};
use crate::collider_tools::utils::calculate_snapped_position;
use crate::selection::EditorSelection;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_input;

// Re-use visualization functions from the visualization module
use super::EditorGizmoConfigGroup;
use super::visualization::draw_preview_anchor;

/// State for anchor creation and manipulation
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource, Default)]
pub struct AnchorCreationState {
    /// Whether Ctrl key is pressed for precise placement
    pub ctrl_pressed: bool,
    /// Whether Shift key is pressed for vertex snapping mode
    pub shift_pressed: bool,
    /// Whether preview mode is active (C key pressed)
    pub preview_mode: bool,
    /// Current preview anchor position
    pub preview_position: Option<Vec2>,
    /// Associated collider entity for preview
    pub preview_collider: Option<Entity>,
    /// Currently selected anchor for editing
    pub selected_anchor: Option<Entity>,
    /// Mouse position in world space
    pub mouse_position: Option<Vec2>,
    /// Whether anchor is currently being dragged
    pub is_dragging: bool,
}

/// Component for connection detection animation
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ConnectionAnimation {
    pub start_time: f32,
    pub duration: f32,
    pub target_entity: Entity,
    pub is_anchor: bool,
}

impl ConnectionAnimation {
    pub fn new(target_entity: Entity, is_anchor: bool) -> Self {
        Self {
            start_time: 0.0,
            duration: 1.0, // 1 second animation
            target_entity,
            is_anchor,
        }
    }
}

/// Plugin for anchor creation and manipulation
#[derive(Default)]
pub struct AnchorCreationPlugin;

impl Plugin for AnchorCreationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnchorCreationState>()
            .add_observer(on_anchor_removed)
            .add_systems(
                OnEnter(crate::collider_tools::ToolMode::Anchor),
                on_enter_anchor_mode,
            )
            .add_systems(
                OnExit(crate::collider_tools::ToolMode::Anchor),
                on_exit_anchor_mode,
            )
            .add_systems(
                Update,
                (
                    update_mouse_position,
                    update_anchor_preview,
                    update_anchor_preview_visualization::<EditorGizmoConfigGroup>,
                    sync_anchor_selection_with_editor_selection,
                    handle_anchor_mode_input,
                    handle_anchor_dragging,
                )
                    .run_if(
                        in_state(crate::collider_tools::ToolMode::Anchor)
                            .and(not(egui_wants_any_input)),
                    ),
            );
    }
}

/// Observer that automatically cleans up joints when an anchor is removed
fn on_anchor_removed(
    trigger: Trigger<OnRemove, AnchorPoint>,
    mut commands: Commands,
    all_anchor_used_by_query: Query<(Entity, &AnchorUsedBy)>,
    visualized_by_query: Query<&VisualizedBy>,
) {
    let anchor_entity = trigger.target();
    info!(
        "AnchorPoint component removed from entity {:?}, cleaning up joints",
        anchor_entity
    );

    // Delete all joints associated with this anchor
    delete_joints_for_anchor(
        &mut commands,
        anchor_entity,
        &all_anchor_used_by_query,
        &visualized_by_query,
    );
}

pub(super) fn on_enter_anchor_mode(
    mut anchor_state: ResMut<AnchorCreationState>,
    _selection: Res<EditorSelection>,
    _commands: Commands,
) {
    info!(
        "Entered anchor mode - Left click to select anchors, C to enter preview mode then click to create (Ctrl for precise, Shift for vertex snapping), Delete to remove selected anchors"
    );

    // Clear state
    *anchor_state = AnchorCreationState::default();
}

/// Clean up anchor mode
pub(super) fn on_exit_anchor_mode(mut anchor_state: ResMut<AnchorCreationState>) {
    info!("Exited anchor mode");

    *anchor_state = AnchorCreationState::default();
}

/// Sync anchor selection state with the editor selection
/// This ensures that when anchors are selected through the Bevy picking system,
/// our anchor_state stays in sync for dragging operations
fn sync_anchor_selection_with_editor_selection(
    selection: Res<EditorSelection>,
    mut anchor_state: ResMut<AnchorCreationState>,
    anchor_query: Query<Entity, With<AnchorPoint>>,
) {
    // Debug: Log current state
    if selection.is_changed() {
        info!(
            "Selection changed! Primary: {:?}, All: {:?}",
            selection.primary(),
            selection.iter().collect::<Vec<_>>()
        );
        info!(
            "Current anchor_state.selected_anchor: {:?}",
            anchor_state.selected_anchor
        );

        if let Some(primary_selected) = selection.primary() {
            info!(
                "Checking if entity {:?} is an anchor: {}",
                primary_selected,
                anchor_query.contains(primary_selected)
            );

            // Check if the selected entity is an anchor
            if anchor_query.contains(primary_selected) {
                if anchor_state.selected_anchor != Some(primary_selected) {
                    anchor_state.selected_anchor = Some(primary_selected);
                    info!("Synced anchor selection: {:?}", primary_selected);
                }
            } else {
                // If a non-anchor entity is selected, clear anchor selection
                if anchor_state.selected_anchor.is_some() {
                    anchor_state.selected_anchor = None;
                    info!("Cleared anchor selection - non-anchor entity selected");
                }
            }
        } else {
            // No entity selected, clear anchor selection
            if anchor_state.selected_anchor.is_some() {
                anchor_state.selected_anchor = None;
                info!("Cleared anchor selection - no entity selected");
            }
        }
    }
}

fn handle_anchor_mode_input(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut anchor_state: ResMut<AnchorCreationState>,
    mut commands: Commands,
    mut selection: ResMut<EditorSelection>,
    anchor_query: Query<(Entity, &AnchorPoint, &GlobalTransform)>,
    collider_query: Query<(Entity, &GlobalTransform, &Collider, Option<&Children>)>,
) {
    // Update key states
    let ctrl_pressed =
        key_input.pressed(KeyCode::ControlLeft) || key_input.pressed(KeyCode::ControlRight);
    let shift_pressed =
        key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);

    // Update state if key states changed
    if ctrl_pressed != anchor_state.ctrl_pressed {
        anchor_state.ctrl_pressed = ctrl_pressed;
        info!(
            "Ctrl {} - precise placement mode",
            if ctrl_pressed { "pressed" } else { "released" }
        );
    }
    if shift_pressed != anchor_state.shift_pressed {
        anchor_state.shift_pressed = shift_pressed;
        info!(
            "Shift {} - vertex snapping mode",
            if shift_pressed { "pressed" } else { "released" }
        );
    }

    // Get current mouse position
    let Some(mouse_pos) = anchor_state.mouse_position else {
        return;
    };

    // Handle anchor selection (similar to joint system)
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if anchor_state.preview_mode {
            if let Some((collider_entity, anchor_pos)) = anchor_state
                .preview_collider
                .zip(anchor_state.preview_position)
            {
                // Check for duplicates using the relationship system
                let duplicate_threshold = 1.0;
                let is_duplicate =
                    if let Ok((_, _, _, Some(children))) = collider_query.get(collider_entity) {
                        children.iter().any(|child_entity| {
                            if let Ok((_, anchor_point, _)) = anchor_query.get(child_entity) {
                                let calculated_pos = if let Ok((_, collider_transform, _, _)) =
                                    collider_query.get(anchor_point.parent_entity)
                                {
                                    calculate_anchor_world_position_from_anchor(
                                        anchor_point,
                                        &collider_transform,
                                    )
                                } else {
                                    return false;
                                };
                                calculated_pos.distance(anchor_pos) < duplicate_threshold
                            } else {
                                false
                            }
                        })
                    } else {
                        false
                    };

                if !is_duplicate {
                    if let Ok((_, collider_transform, _, _)) = collider_query.get(collider_entity) {
                        let anchor_entity = create_anchor_at_position(
                            &mut commands,
                            anchor_pos,
                            collider_entity,
                            Some(&collider_transform),
                        );

                        anchor_state.selected_anchor = Some(anchor_entity);

                        // Select the new anchor
                        selection.clear();
                        selection.add(anchor_entity);

                        info!(
                            "Created new anchor at {:?} on collider {:?}",
                            anchor_pos, collider_entity
                        );

                        // Exit preview mode after creation
                        anchor_state.preview_mode = false;
                        anchor_state.preview_position = None;
                        anchor_state.preview_collider = None;
                    }
                }
            } else {
                warn!("No collider selected for anchor creation");
            }
        } else {
            let selected_anchor = anchor_query
                .iter()
                .find_map(|(entity, anchor, _transform)| {
                    // Use the same calculation method as draw_anchor_gizmos for consistency
                    let calculated_anchor_pos = if let Ok((_, collider_transform, _, _)) =
                        collider_query.get(anchor.parent_entity)
                    {
                        calculate_anchor_world_position_from_anchor(anchor, &collider_transform)
                    } else {
                        return None;
                    };

                    // Use center point distance with a small threshold for precise selection
                    let distance = mouse_pos.distance(calculated_anchor_pos);
                    if distance <= 8.0 {
                        // Fixed selection threshold for anchor center
                        info!(
                            "Selected anchor at {:?} (distance: {})",
                            calculated_anchor_pos,
                            mouse_pos.distance(calculated_anchor_pos)
                        );
                        Some(entity)
                    } else {
                        None
                    }
                });

            if let Some(anchor_entity) = selected_anchor {
                // Select the anchor
                anchor_state.selected_anchor = Some(anchor_entity);
                selection.clear();
                selection.add(anchor_entity);
                info!("Selected anchor: {:?}", anchor_entity);
                return;
            } else {
                // Clear anchor selection if clicking elsewhere
                anchor_state.selected_anchor = None;
            }
        }
    }

    // Handle right click - toggle preview mode
    if mouse_button_input.just_pressed(MouseButton::Right) && !selection.is_empty() {
        // Toggle preview mode
        anchor_state.preview_mode = !anchor_state.preview_mode;

        if anchor_state.preview_mode {
            info!("Entered anchor preview mode - move mouse to preview, click to create");
        } else {
            info!("Exited anchor preview mode");
            // Clear preview when exiting preview mode
            anchor_state.preview_position = None;
            anchor_state.preview_collider = None;
        }
    }

    // Handle Delete key - remove selected anchor
    if key_input.just_pressed(KeyCode::Delete) {
        if let Some(anchor_entity) = anchor_state.selected_anchor {
            // The OnRemove observer will automatically handle joint cleanup
            commands.entity(anchor_entity).despawn();
            selection.remove(anchor_entity);
            anchor_state.selected_anchor = None;
            info!("Deleted anchor: {:?}", anchor_entity);
        }
    }

    // Handle Escape key - cancel selection
    if key_input.just_pressed(KeyCode::Escape) {
        anchor_state.preview_mode = false;

        if anchor_state.selected_anchor.is_some() {
            anchor_state.selected_anchor = None;
            info!("Cancelled anchor selection");
        }
    }
}

/// Update mouse position and tracking
fn update_mouse_position(
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
    mut anchor_state: ResMut<AnchorCreationState>,
) {
    let Ok(window) = windows.single() else {
        anchor_state.mouse_position = None;
        return;
    };

    let Ok((camera, camera_transform)) = camera_query.single() else {
        anchor_state.mouse_position = None;
        return;
    };

    anchor_state.mouse_position = get_mouse_world_position(window, camera, camera_transform);
}

fn update_anchor_preview(
    mut anchor_state: ResMut<AnchorCreationState>,
    collider_query: Query<(&GlobalTransform, &Collider, Option<&Children>)>,
    selection: Res<EditorSelection>,
) {
    // Only update preview when in preview mode
    if !anchor_state.preview_mode {
        anchor_state.preview_position = None;
        anchor_state.preview_collider = None;
        return;
    }

    let Some(mouse_pos) = anchor_state.mouse_position else {
        anchor_state.preview_position = None;
        anchor_state.preview_collider = None;
        return;
    };

    // Find a selected collider to create anchor on
    let target_collider = {
        let mut best_target = None;
        let mut min_distance = f32::MAX;

        for collider_entity in selection.iter() {
            if let Ok((collider_transform, collider, _)) = collider_query.get(collider_entity) {
                let target_pos = calculate_snapped_position(
                    mouse_pos,
                    collider_transform,
                    collider,
                    anchor_state.shift_pressed,
                    anchor_state.ctrl_pressed,
                );

                let distance = (target_pos - mouse_pos).length();
                if distance < min_distance {
                    min_distance = distance;
                    best_target = Some((target_pos, collider_entity));
                }
            }
        }
        best_target
    };

    // Update preview state
    match target_collider {
        Some((pos, collider)) => {
            anchor_state.preview_position = Some(pos);
            anchor_state.preview_collider = Some(collider);
        }
        None => {
            anchor_state.preview_position = None;
            anchor_state.preview_collider = None;
        }
    }
}

fn update_anchor_preview_visualization<Config: GizmoConfigGroup>(
    mut gizmos: Gizmos<Config>,
    anchor_state: Res<AnchorCreationState>,
    time: Res<Time>,
    collider_query: Query<&GlobalTransform, With<Collider>>,
    theme_colors: Res<crate::ui::theme_colors::EditorThemeColors>,
) {
    // Only show preview when in preview mode
    if anchor_state.preview_mode {
        if let Some(preview_pos) = anchor_state.preview_position {
            // Draw preview anchor with mode-specific color
            let preview_color = if anchor_state.ctrl_pressed {
                Color::srgba(0.0, 1.0, 0.0, 0.8) // Green for precise (Ctrl)
            } else if anchor_state.shift_pressed {
                Color::srgba(1.0, 0.0, 1.0, 0.8) // Magenta for vertex snapping (Shift)
            } else {
                Color::srgba(1.0, 1.0, 0.0, 0.8) // Yellow for free placement mode
            };

            // Draw preview anchor
            draw_preview_anchor(&mut gizmos, preview_pos, time.elapsed_secs(), &theme_colors);

            // Draw connection line to collider origin if we have a collider
            if let Some(collider_entity) = anchor_state.preview_collider {
                if let Ok(collider_transform) = collider_query.get(collider_entity) {
                    let collider_origin = collider_transform.translation().truncate();

                    // Draw dashed connection line
                    super::visualization::draw_dashed_line(
                        &mut gizmos,
                        collider_origin,
                        preview_pos,
                        preview_color,
                        time.elapsed_secs(),
                    );
                }
            }
        }
    }
}

/// Handle anchor dragging - move selected anchor with mouse
fn handle_anchor_dragging(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut anchor_state: ResMut<AnchorCreationState>,
    mut anchor_query: Query<(&mut AnchorPoint, &mut Transform)>,
    collider_query: Query<(&GlobalTransform, &Collider, Option<&Children>)>,
    mut selection: ResMut<EditorSelection>,
    mut commands: Commands,
    all_anchor_used_by_query: Query<(Entity, &AnchorUsedBy)>,
    joint_config_query: Query<&JointConfig>,
    visualized_by_query: Query<&VisualizedBy>,
) {
    let Some(anchor_entity) = anchor_state.selected_anchor else {
        return;
    };

    let Some(mouse_pos) = anchor_state.mouse_position else {
        return;
    };

    // Update key states for snapping during drag
    let ctrl_pressed =
        key_input.pressed(KeyCode::ControlLeft) || key_input.pressed(KeyCode::ControlRight);
    let shift_pressed =
        key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);

    // Update state if key states changed
    if ctrl_pressed != anchor_state.ctrl_pressed {
        anchor_state.ctrl_pressed = ctrl_pressed;
        info!(
            "Ctrl {} - precise placement mode",
            if ctrl_pressed { "pressed" } else { "released" }
        );
    }
    if shift_pressed != anchor_state.shift_pressed {
        anchor_state.shift_pressed = shift_pressed;
        info!(
            "Shift {} - vertex snapping mode",
            if shift_pressed { "pressed" } else { "released" }
        );
    }

    // Handle drag start - check if we should start dragging on the selected anchor
    if mouse_button_input.just_pressed(MouseButton::Left) && !anchor_state.is_dragging {
        // Check if we're clicking on the selected anchor
        let Ok((anchor, _)) = anchor_query.get(anchor_entity) else {
            return;
        };

        // Calculate the correct anchor position using the utility function
        let calculated_anchor_pos =
            if let Ok((collider_transform, _, _)) = collider_query.get(anchor.parent_entity) {
                calculate_anchor_world_position_from_anchor(&anchor, &collider_transform)
            } else {
                return;
            };

        // Use center point distance for consistent drag start with calculated position
        let distance = mouse_pos.distance(calculated_anchor_pos);
        if distance <= 8.0 {
            // Fixed threshold for anchor center
            anchor_state.is_dragging = true;
            info!("Started anchor drag - physics paused");
        }
    }

    // Handle drag end
    if mouse_button_input.just_released(MouseButton::Left) && anchor_state.is_dragging {
        anchor_state.is_dragging = false;
        info!("Ended anchor drag - physics resumed");
        return;
    }

    // Only drag when left mouse button is held down AND we're in dragging state
    if !mouse_button_input.pressed(MouseButton::Left) || !anchor_state.is_dragging {
        return;
    }

    let Ok((mut anchor, mut transform)) = anchor_query.get_mut(anchor_entity) else {
        warn!(
            "Failed to get anchor components for dragging: {:?}",
            anchor_entity
        );
        anchor_state.selected_anchor = None;
        return;
    };

    // Get the parent collider's transform
    if let Ok((collider_transform, collider, _)) = collider_query.get(anchor.parent_entity) {
        // Calculate the target position with snapping
        let target_pos = calculate_snapped_position(
            mouse_pos,
            collider_transform,
            collider,
            anchor_state.shift_pressed,
            anchor_state.ctrl_pressed,
        );

        // Calculate local anchor position relative to collider origin
        let collider_pos = collider_transform.translation().xy();
        let local_anchor_pos = target_pos - collider_pos;

        // Update the local anchor position
        anchor.local_anchor_position = local_anchor_pos;

        // Update the world transform of the anchor using the correct calculation
        let rotated_anchor = collider_transform.rotation() * local_anchor_pos.extend(0.0);
        let world_pos = collider_pos + rotated_anchor.xy();
        transform.translation = world_pos.extend(transform.translation.z);

        // Regenerate all joints associated with this anchor
        // Create a temporary query for joint regeneration
        let anchor_query_for_joints = anchor_query.into_readonly();
        regenerate_joints_for_anchor_with_viz(
            &mut commands,
            anchor_entity,
            &anchor_query_for_joints,
            &all_anchor_used_by_query,
            &joint_config_query,
            &visualized_by_query,
        );

        // Update selection to highlight the dragged anchor
        selection.clear();
        selection.add(anchor_entity);

        // Debug logging for dragging start
        if mouse_button_input.just_pressed(MouseButton::Left) {
            info!(
                "Started dragging anchor {:?} to local position {:?}",
                anchor_entity, local_anchor_pos
            );
        }
    } else {
        warn!(
            "Failed to get parent collider transform for anchor: {:?}",
            anchor_entity
        );
        anchor_state.selected_anchor = None;
    }
}
