//! Joint selection and deletion tools
//!
//! Provides functionality for selecting and deleting joints in the editor.

use super::debug_render::joint::{JointVisualization, JointVisualizationOf, joint_relationships};
use super::utils::calculate_anchor_world_position_from_anchor;
use crate::debug_render::anchor::AnchorPoint;
use crate::selection::{EditorSelection, Selectable};
use avian2d::prelude::*;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_input;

/// Plugin for joint selection and deletion
#[derive(Default)]
pub struct JointSelectionPlugin;

impl Plugin for JointSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_joint_selection_input,
                update_joint_selection_visual,
                handle_joint_deletion,
            )
                .run_if(
                    in_state(crate::collider_tools::ToolMode::Joint).and(not(egui_wants_any_input)),
                ),
        );
    }
}

/// Handle mouse input for joint selection
fn handle_joint_selection_input(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<EditorSelection>,
    joint_query: Query<(Entity, &JointVisualization), With<Selectable>>,
    anchor_query: Query<&AnchorPoint>,
    collider_query: Query<&GlobalTransform, With<Collider>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
    window_query: Query<&Window>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }

    // Get mouse world position
    let Some(mouse_world_pos) = get_mouse_world_position(&camera_query, &window_query) else {
        return;
    };

    let shift_pressed =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let ctrl_pressed = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    // Find the closest joint to the mouse position
    let mut closest_joint: Option<(Entity, f32)> = None;
    const SELECTION_THRESHOLD: f32 = 20.0; // pixels

    for (entity, joint_vis) in joint_query.iter() {
        // Get positions for both endpoints, handling both anchors and origins
        let pos_a = get_joint_endpoint_position(joint_vis.anchor_a, &anchor_query, &collider_query);
        let pos_b = get_joint_endpoint_position(joint_vis.anchor_b, &anchor_query, &collider_query);

        if let (Some(pos_a), Some(pos_b)) = (pos_a, pos_b) {
            // Calculate distance from mouse to joint line
            let distance = distance_point_to_line_segment(mouse_world_pos, pos_a, pos_b);

            if distance < SELECTION_THRESHOLD {
                if let Some((_, current_distance)) = closest_joint {
                    if distance < current_distance {
                        closest_joint = Some((entity, distance));
                    }
                } else {
                    closest_joint = Some((entity, distance));
                }
            }
        }
    }

    // Handle selection based on input modifiers
    if let Some((joint_entity, _)) = closest_joint {
        if shift_pressed {
            // Add to selection
            selection.add(joint_entity);
        } else if ctrl_pressed {
            // Toggle selection
            selection.toggle(joint_entity);
        } else {
            // Replace selection
            selection.clear();
            selection.add(joint_entity);
        }
    } else if !shift_pressed && !ctrl_pressed {
        // Clear selection if clicking on empty space
        selection.clear();
    }
}

/// Get the world position of a joint endpoint (either anchor or collider origin)
fn get_joint_endpoint_position(
    endpoint_entity: Entity,
    anchor_query: &Query<&AnchorPoint>,
    collider_query: &Query<&GlobalTransform, With<Collider>>,
) -> Option<Vec2> {
    // First check if this endpoint is an anchor
    if let Ok(anchor) = anchor_query.get(endpoint_entity) {
        // Calculate the actual anchor position using the same method as anchor.rs
        if let Ok(collider_transform) = collider_query.get(anchor.parent_entity) {
            Some(calculate_anchor_world_position_from_anchor(
                anchor,
                &collider_transform,
            ))
        } else {
            None
        }
    } else {
        // If not an anchor, it should be a collider origin
        if let Ok(transform) = collider_query.get(endpoint_entity) {
            Some(transform.translation().truncate())
        } else {
            None
        }
    }
}

/// Update visual state of joints based on selection
fn update_joint_selection_visual(
    mut joint_query: Query<(Entity, &mut JointVisualization)>,
    selection: Res<EditorSelection>,
) {
    for (entity, mut joint_vis) in joint_query.iter_mut() {
        joint_vis.selected = selection.contains(entity);
    }
}

/// Handle joint deletion via keyboard input
fn handle_joint_deletion(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut selection: ResMut<EditorSelection>,
    joint_query: Query<(Entity, &JointVisualization)>,
    mut anchor_query: Query<&mut super::debug_render::anchor::AnchorPoint>,
    joint_viz_query: Query<&JointVisualizationOf>,
) {
    // Check for delete key press
    if !keyboard_input.just_pressed(KeyCode::Delete)
        && !keyboard_input.just_pressed(KeyCode::Backspace)
    {
        return;
    }

    let selected_entities: Vec<Entity> = selection.iter().collect();

    for entity in selected_entities {
        if let Ok((_, joint_vis)) = joint_query.get(entity) {
            // Mark anchors as no longer in joint (only if they are actually anchors)
            if let Ok(mut anchor_a) = anchor_query.get_mut(joint_vis.anchor_a) {
                anchor_a.in_joint = false;
            }
            if let Ok(mut anchor_b) = anchor_query.get_mut(joint_vis.anchor_b) {
                anchor_b.in_joint = false;
            }

            // Remove the physics joint using the relationship
            if let Some(physics_joint_entity) =
                joint_relationships::get_joint_for_visualization(entity, &joint_viz_query)
            {
                commands.entity(physics_joint_entity).despawn();
                info!("Deleted physics joint: {:?}", physics_joint_entity);
            }

            // Remove the joint visualization entity - the observer will handle cleanup
            commands.entity(entity).despawn();

            info!("Deleted joint visualization: {:?}", entity);
        }
    }

    // Clear selection after deletion
    selection.clear();
}

/// Calculate distance from a point to a line segment
fn distance_point_to_line_segment(point: Vec2, line_start: Vec2, line_end: Vec2) -> f32 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;

    let line_len_sq = line_vec.length_squared();

    if line_len_sq == 0.0 {
        // Line is actually a point
        return point_vec.length();
    }

    let t = (point_vec.dot(line_vec) / line_len_sq).clamp(0.0, 1.0);
    let projection = line_start + t * line_vec;

    (point - projection).length()
}

/// Get mouse world position (utility function)
fn get_mouse_world_position(
    camera_query: &Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
    window_query: &Query<&Window>,
) -> Option<Vec2> {
    let (camera, camera_transform) = camera_query.single().ok()?;
    let window = window_query.single().ok()?;

    let cursor_position = window.cursor_position()?;
    let world_position = camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()?;

    Some(world_position)
}
