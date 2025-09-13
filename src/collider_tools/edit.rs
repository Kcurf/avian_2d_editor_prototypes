use crate::{DragCancelClick, EditorSelection};

use super::{ColliderCreationState, ColliderType, utils::*, visualization::*};

use super::ColliderData;
use avian2d::parry::shape::TypedShape;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_input;

use super::utils::{
    update_dynamic_capsule_control_point, update_dynamic_circle_control_point,
    update_dynamic_rectangle_control_point,
};

/// Edit state for collider modification
#[derive(Clone, Resource)]
pub struct ColliderEditState {
    /// Control points for the selected collider
    pub control_points: Vec<ControlPoint>,
    /// Currently dragged control point
    pub dragging_point: Option<usize>,
    /// Original collider data before editing (for undo)
    pub original_collider_data: Option<ColliderData>,
    /// Edit history for undo/redo functionality
    pub edit_history: EditHistory,
    /// Last selected entity to track selection changes
    pub last_selected_entity: Option<Entity>,
    /// Last valid rectangle size to prevent degeneration
    pub last_rectangle_size: Option<Vec2>,
    /// Entity being edited (preserved during drag when selection is cleared)
    pub editing_entity: Option<Entity>,
}

impl Default for ColliderEditState {
    fn default() -> Self {
        Self {
            control_points: Vec::new(),
            dragging_point: None,
            original_collider_data: None,
            edit_history: EditHistory::default(),
            last_selected_entity: None,
            last_rectangle_size: None,
            editing_entity: None,
        }
    }
}

/// Control point for collider editing
#[derive(Clone, Debug)]
pub struct ControlPoint {
    /// World position of the control point
    pub position: Vec2,
    /// Type of control point (corner, edge midpoint, radius control, etc.)
    pub point_type: ControlPointType,
    /// Index in the collider's vertex array (if applicable)
    pub vertex_index: Option<usize>,
}

/// Types of control points for different editing operations
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlPointType {
    /// Corner vertex that can be moved to reshape the collider
    Vertex,
    /// Edge midpoint for adding new vertices or adjusting curves
    EdgeMidpoint,
    /// Radius control for circles and capsules
    RadiusControl,
    /// Length control for capsules
    LengthControl,
}

/// Component for control point entities in edit mode
#[derive(Component)]
pub struct ControlPointMarker {
    /// Index of this control point
    pub index: usize,
    /// Type of control point
    pub point_type: ControlPointType,
}

/// Marker component to identify control point entities and prevent selection conflicts
/// IMPORTANT: Any entity with this component should NOT be processed by selection systems
#[derive(Component, Debug)]
pub struct ControlPointEntity;

/// Edit history for undo/redo functionality
#[derive(Clone)]
pub struct EditHistory {
    /// Stack of previous states
    pub undo_stack: Vec<ColliderData>,
    /// Stack of undone states (for redo)
    pub redo_stack: Vec<ColliderData>,
    /// Maximum number of undo steps to keep
    pub max_history: usize,
}

impl Default for EditHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

// Selection functions have been moved to the selection module

impl EditHistory {
    /// Push a new state to the undo stack
    pub fn push_state(&mut self, state: ColliderData) {
        self.undo_stack.push(state);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
        // Clear redo stack when new action is performed
        self.redo_stack.clear();
    }

    /// Pop the last state from undo stack
    pub fn undo(&mut self) -> Option<ColliderData> {
        self.undo_stack.pop()
    }

    /// Push a state to redo stack
    pub fn push_redo(&mut self, state: ColliderData) {
        self.redo_stack.push(state);
    }

    /// Pop a state from redo stack
    pub fn redo(&mut self) -> Option<ColliderData> {
        self.redo_stack.pop()
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

// ===== EDIT MODE SYSTEMS =====

/// Unified control point interaction system
///
/// This system handles all control point interactions including selection,
/// dragging, and state management in a single, unified system to avoid
/// conflicts between multiple interaction systems.
pub fn handle_control_point_interaction(
    mut commands: Commands,
    mut edit_state: ResMut<ColliderEditState>,
    selection: Res<EditorSelection>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
    mut collider_query: Query<(&mut Transform, &mut Collider, &ColliderType)>,
) {
    // Get cursor position once at the beginning
    let cursor_pos = if let (Ok(window), Ok((camera, camera_transform))) =
        (windows.single(), camera_query.single())
    {
        get_cursor_world_position(&window, &camera, &camera_transform)
    } else {
        None
    };

    // Update dynamic control point position for all supported collider types
    if let Some(selected_entity) = selection.primary() {
        if let Ok((transform, collider, collider_type)) = collider_query.get(selected_entity) {
            match collider_type {
                ColliderType::Rectangle => {
                    update_dynamic_rectangle_control_point(
                        cursor_pos,
                        collider,
                        transform,
                        &mut edit_state,
                    );
                }
                ColliderType::Circle => {
                    update_dynamic_circle_control_point(
                        cursor_pos,
                        collider,
                        transform,
                        &mut edit_state,
                    );
                }
                ColliderType::Capsule => {
                    update_dynamic_capsule_control_point(
                        cursor_pos,
                        collider,
                        transform,
                        &mut edit_state,
                    );
                }
                _ => {
                    // Other collider types use their existing control point systems
                }
            }
        }
    }

    // Handle mouse press for control point selection
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = cursor_pos {
            let mut clicked_point = None;
            let mut min_distance = f32::INFINITY;

            // Generate control points if they don't exist
            if edit_state.control_points.is_empty() {
                if let Some(entity) = selection.primary() {
                    if let Ok((transform, collider, created_collider)) = collider_query.get(entity)
                    {
                        generate_control_points(
                            &mut edit_state,
                            transform,
                            collider,
                            created_collider,
                        );
                    }
                }
            }

            // Check for control point clicks
            for (index, control_point) in edit_state.control_points.iter().enumerate() {
                let distance_squared = control_point.position.distance_squared(cursor_pos);
                let radius = match control_point.point_type {
                    ControlPointType::Vertex => 8.0,
                    ControlPointType::RadiusControl | ControlPointType::LengthControl => 8.0,
                    _ => 6.0,
                };
                let threshold_squared = (radius + 2.0) * (radius + 2.0);

                if distance_squared <= threshold_squared && distance_squared < min_distance {
                    clicked_point = Some(index);
                    min_distance = distance_squared;
                }
            }

            if let Some(point_index) = clicked_point {
                // Start dragging the control point
                edit_state.dragging_point = Some(point_index);

                // Save the entity being edited and current state to history
                if let Some(entity) = selection.primary() {
                    edit_state.editing_entity = Some(entity);
                    if let Ok((transform, collider, collider_type)) = collider_query.get(entity) {
                        let current_state = ColliderData {
                            transform: *transform,
                            collider: collider.clone(),
                            collider_type: *collider_type,
                        };
                        edit_state.edit_history.push_state(current_state);
                    }
                }
            }
        }
    }

    // Handle mouse drag for control point movement
    if mouse_button.pressed(MouseButton::Left) {
        if let (Some(cursor_pos), Some(dragging_index)) = (cursor_pos, edit_state.dragging_point) {
            if let Some(control_point) = edit_state.control_points.get_mut(dragging_index) {
                control_point.position = cursor_pos;

                // Apply changes to the collider using the preserved editing entity
                if let Some(entity) = edit_state.editing_entity {
                    if let Ok((mut transform, mut collider, collider_type)) =
                        collider_query.get_mut(entity)
                    {
                        apply_control_point_changes(
                            &mut edit_state,
                            &mut transform,
                            &mut collider,
                            collider_type,
                        );
                    }
                }
            }
        }
    }

    // Handle mouse release to stop dragging
    if mouse_button.just_released(MouseButton::Left) {
        if edit_state.dragging_point.is_some() {
            edit_state.dragging_point = None;

            // Save state for undo after dragging is complete
            if let Some(entity) = edit_state.editing_entity {
                if let Ok((transform, collider, collider_type)) = collider_query.get(entity) {
                    let new_data = ColliderData {
                        transform: *transform,
                        collider: collider.clone(),
                        collider_type: *collider_type,
                    };
                    edit_state.edit_history.push_state(new_data);
                }
                // Clear the editing entity after drag is complete
                edit_state.editing_entity = None;
            }
        }
    }

    // Handle escape key to cancel drag
    if keyboard.just_pressed(KeyCode::Escape) {
        if edit_state.dragging_point.is_some() {
            info!("Edit mode: Escape pressed, canceling drag operation");
            edit_state.dragging_point = None;

            // Restore original state if we were dragging
            if let Some(selected_entity) = selection.primary() {
                if let Ok((transform, collider, collider_type)) =
                    collider_query.get(selected_entity)
                {
                    // Regenerate control points to reset their positions
                    generate_control_points(&mut edit_state, transform, collider, collider_type);
                }
            }
        }
    }

    // Handle undo/redo
    if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
        if let Some(selected_entity) = selection.primary() {
            if let Ok((transform, collider, created_collider)) = collider_query.get(selected_entity)
            {
                if keyboard.just_pressed(KeyCode::KeyZ) {
                    if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
                    {
                        // Redo with Ctrl+Shift+Z
                        info!("Edit mode: Performing redo operation");
                        handle_redo(
                            &mut commands,
                            &mut edit_state,
                            selected_entity,
                            transform,
                            collider,
                            created_collider,
                        );
                    } else {
                        // Undo with Ctrl+Z
                        info!("Edit mode: Performing undo operation");
                        handle_undo(
                            &mut commands,
                            &mut edit_state,
                            selected_entity,
                            transform,
                            collider,
                            created_collider,
                        );
                    }
                } else if keyboard.just_pressed(KeyCode::KeyY) {
                    // Redo with Ctrl+Y
                    info!("Edit mode: Performing redo operation (Ctrl+Y)");
                    handle_redo(
                        &mut commands,
                        &mut edit_state,
                        selected_entity,
                        transform,
                        collider,
                        created_collider,
                    );
                }
            }
        }
    }

    // Handle reset
    if keyboard.just_pressed(KeyCode::KeyR) && keyboard.pressed(KeyCode::ControlLeft) {
        if let Some(selected_entity) = selection.primary() {
            if let Ok((transform, collider, created_collider)) = collider_query.get(selected_entity)
            {
                info!("Edit mode: Resetting collider to original state");
                handle_reset(
                    &mut commands,
                    &mut edit_state,
                    selected_entity,
                    transform,
                    collider,
                    created_collider,
                );
            }
        }
    }
}

/// Update control points for the selected collider (optimized for performance)
pub fn update_control_points(
    mut commands: Commands,
    mut edit_state: ResMut<ColliderEditState>,
    selection: Res<EditorSelection>,
    collider_query: Query<(Entity, &Transform, &Collider, &ColliderType)>,
    control_point_query: Query<(Entity, &ControlPointMarker, &Transform), With<ControlPointEntity>>,
) {
    // Always update when selection changes or control points are empty
    let current_selection = selection.primary();
    let needs_update = match (current_selection, edit_state.last_selected_entity) {
        (Some(selected_entity), Some(last_entity)) if selected_entity != last_entity => {
            // Selection changed to a different entity
            edit_state.last_selected_entity = Some(selected_entity);
            if let Ok((_, transform, collider, created_collider)) =
                collider_query.get(selected_entity)
            {
                generate_control_points(&mut edit_state, transform, collider, created_collider);
            }
            true
        }
        (Some(selected_entity), None) => {
            // New selection when previously none
            edit_state.last_selected_entity = Some(selected_entity);
            if let Ok((_, transform, collider, created_collider)) =
                collider_query.get(selected_entity)
            {
                generate_control_points(&mut edit_state, transform, collider, created_collider);
            }
            true
        }
        (None, Some(_last_entity)) => {
            // Selection cleared - but don't clear if we're currently dragging
            if edit_state.dragging_point.is_some() {
                // Don't clear the drag state, just update the last_selected_entity
                edit_state.last_selected_entity = None;
                // Keep control_points and dragging_point to allow drag to continue
            } else {
                edit_state.last_selected_entity = None;
                edit_state.control_points.clear();
                edit_state.dragging_point = None; // Clear dragging state when selection is cleared
            }
            true
        }
        (Some(selected_entity), Some(last_entity)) if selected_entity == last_entity => {
            // Same entity selected, only update if control points are empty
            if edit_state.control_points.is_empty() {
                if let Ok((_, transform, collider, created_collider)) =
                    collider_query.get(selected_entity)
                {
                    generate_control_points(&mut edit_state, transform, collider, created_collider);
                }
                true
            } else {
                false
            }
        }
        _ => {
            // No selection and no previous selection
            false
        }
    };

    if needs_update {
        // Remove existing control point entities only when needed
        for (entity, _, _) in control_point_query.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn new control point entities and store mapping
        for (index, control_point) in edit_state.control_points.iter().enumerate() {
            let control_point_entity = commands
                .spawn((
                    Transform::from_translation(control_point.position.extend(1.0)),
                    ControlPointMarker {
                        index,
                        point_type: control_point.point_type.clone(),
                    },
                    ControlPointEntity, // Add marker to prevent selection conflicts
                    // Add picking components to ensure proper interaction
                    bevy::picking::Pickable::default(),
                    // IMPORTANT: Control points must NOT have Selectable component
                    // to prevent selection system from interfering with drag operations
                ))
                .id();

            info!(
                "Spawned control point entity {:?} for index {} at position {:?}",
                control_point_entity, index, control_point.position
            );
        }
    } else {
        // Just update positions of existing entities for better performance
        for (entity, marker, existing_transform) in control_point_query.iter() {
            if let Some(control_point) = edit_state.control_points.get(marker.index) {
                let new_pos = control_point.position.extend(1.0);
                if existing_transform.translation.distance_squared(new_pos) > 0.01 {
                    commands
                        .entity(entity)
                        .insert(Transform::from_translation(new_pos));
                }
            }
        }
    }
}

/// Apply control point changes to the collider
pub fn apply_control_point_changes(
    edit_state: &mut ColliderEditState,
    transform: &mut Transform,
    collider: &mut Collider,
    collider_type: &ColliderType,
) {
    // Handle vertex/shape modifications
    match collider_type {
        ColliderType::Rectangle => {
            // Update rectangle based on single dynamic control point
            let current_center = transform.translation.truncate();

            // Get the dragged control point position
            if let Some(control_point) = edit_state.control_points.first() {
                if control_point.point_type == ControlPointType::Vertex {
                    // Calculate the distance from the dragged corner to the center
                    let corner_offset = control_point.position - current_center;

                    // Calculate new size based on the corner offset
                    // The corner index tells us which corner is being dragged
                    let new_size = if let Some(corner_index) = control_point.vertex_index {
                        match corner_index {
                            0 => {
                                // Bottom-left corner
                                Vec2::new(
                                    (-corner_offset.x * 2.0).abs().max(0.01),
                                    (-corner_offset.y * 2.0).abs().max(0.01),
                                )
                            }
                            1 => {
                                // Bottom-right corner
                                Vec2::new(
                                    (corner_offset.x * 2.0).abs().max(0.01),
                                    (-corner_offset.y * 2.0).abs().max(0.01),
                                )
                            }
                            2 => {
                                // Top-right corner
                                Vec2::new(
                                    (corner_offset.x * 2.0).abs().max(0.01),
                                    (corner_offset.y * 2.0).abs().max(0.01),
                                )
                            }
                            3 => {
                                // Top-left corner
                                Vec2::new(
                                    (-corner_offset.x * 2.0).abs().max(0.01),
                                    (corner_offset.y * 2.0).abs().max(0.01),
                                )
                            }
                            _ => {
                                // Fallback to symmetric sizing
                                Vec2::new(corner_offset.x.abs() * 2.0, corner_offset.y.abs() * 2.0)
                            }
                        }
                    } else {
                        // Fallback to symmetric sizing
                        Vec2::new(corner_offset.x.abs() * 2.0, corner_offset.y.abs() * 2.0)
                    };

                    // Store current size for future use
                    edit_state.last_rectangle_size = Some(new_size);

                    // Update collider
                    *collider = Collider::rectangle(new_size.x, new_size.y);

                    // The control point position will be updated by the dynamic control point system
                    // so we don't need to update it here
                }
            }
        }
        ColliderType::Circle => {
            // Update circle based on single dynamic control point
            if let Some(control_point) = edit_state.control_points.first() {
                let current_center = transform.translation.truncate();
                let new_radius = control_point.position.distance(current_center);
                *collider = Collider::circle(new_radius);
            }
        }
        ColliderType::Triangle => {
            // Update triangle based on vertex control points
            if edit_state.control_points.len() >= 3 {
                // Get the current center for coordinate transformation
                let current_center = transform.translation.truncate();
                let rotation = transform.rotation;

                // Helper function to transform world point to local space
                let world_to_local = |world_point: Vec2| -> Vec2 {
                    let relative_pos = world_point - current_center;
                    let inv_rotation = rotation.inverse();
                    let rotated = inv_rotation * Vec3::new(relative_pos.x, relative_pos.y, 0.0);
                    Vec2::new(rotated.x, rotated.y)
                };

                // Extract vertex positions from control points and convert to local space
                let mut local_vertices = Vec::new();
                for control_point in &edit_state.control_points {
                    if control_point.point_type == ControlPointType::Vertex {
                        local_vertices.push(world_to_local(control_point.position));
                    }
                }

                // Ensure we have exactly 3 vertices for a triangle
                if local_vertices.len() >= 3 {
                    // Calculate the centroid of the triangle
                    let centroid =
                        (local_vertices[0] + local_vertices[1] + local_vertices[2]) / 3.0;

                    // Center the vertices around the origin (relative to centroid)
                    let centered_vertices = [
                        local_vertices[0] - centroid,
                        local_vertices[1] - centroid,
                        local_vertices[2] - centroid,
                    ];

                    // Check if the triangle has sufficient area (avoid degenerate triangles)
                    let area = (centered_vertices[1] - centered_vertices[0])
                        .perp_dot(centered_vertices[2] - centered_vertices[0])
                        .abs();

                    if area > 1.0 {
                        // Update the transform to position the triangle at the centroid
                        transform.translation = Vec3::new(
                            current_center.x + centroid.x,
                            current_center.y + centroid.y,
                            transform.translation.z,
                        );

                        // Create new triangle collider with centered vertices
                        *collider = Collider::triangle(
                            centered_vertices[0],
                            centered_vertices[1],
                            centered_vertices[2],
                        );
                    }
                }
            }
        }
        ColliderType::Capsule => {
            // Update capsule based on single dynamic control point
            if let Some(control_point) = edit_state.control_points.first() {
                let current_center = transform.translation.truncate();

                // Extract current capsule properties
                let mut radius = 10.0; // Default radius
                let mut half_height = 25.0; // Default half height

                if let TypedShape::Capsule(capsule_shape) = collider.shape_scaled().as_typed_shape()
                {
                    radius = capsule_shape.radius;
                    half_height = capsule_shape.half_height();
                }

                // Determine what property to update based on the control point type
                match control_point.point_type {
                    ControlPointType::LengthControl => {
                        // Update length and center
                        let control_distance = control_point.position.distance(current_center);
                        let new_half_height = control_distance.max(5.0);

                        // Calculate direction from center to control point
                        let direction =
                            (control_point.position - current_center).normalize_or_zero();

                        // Update transform rotation to match new direction
                        let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
                        transform.rotation = Quat::from_rotation_z(angle);

                        // Update the capsule with new dimensions
                        let relative_start = Vec2::new(0.0, -new_half_height);
                        let relative_end = Vec2::new(0.0, new_half_height);
                        *collider =
                            Collider::capsule_endpoints(radius, relative_start, relative_end);
                    }
                    ControlPointType::RadiusControl => {
                        // Update radius based on capsule geometry
                        // Get the current capsule direction
                        let capsule_direction = transform.rotation * Vec3::new(0.0, 1.0, 0.0);
                        let capsule_direction_2d =
                            Vec2::new(capsule_direction.x, capsule_direction.y).normalize_or_zero();

                        // Project the control point position onto the capsule's main axis
                        let to_control_point = control_point.position - current_center;
                        let projection_length = to_control_point.dot(capsule_direction_2d);

                        // Clamp the projection to the capsule's length
                        let clamped_projection = projection_length.clamp(-half_height, half_height);

                        // Find the closest point on the capsule's center line
                        let closest_center_point =
                            current_center + capsule_direction_2d * clamped_projection;

                        // Calculate the actual radius as the distance from the center line to the control point
                        let new_radius = control_point
                            .position
                            .distance(closest_center_point)
                            .max(2.5);

                        // Update the capsule with new radius
                        let relative_start = Vec2::new(0.0, -half_height);
                        let relative_end = Vec2::new(0.0, half_height);
                        *collider =
                            Collider::capsule_endpoints(new_radius, relative_start, relative_end);
                    }
                    _ => {}
                }
            }
        }
        ColliderType::Polygon => {
            // Update polygon based on vertex control points
            if edit_state.control_points.len() >= 3 {
                // Get the current center for coordinate transformation
                let current_center = transform.translation.truncate();
                let rotation = transform.rotation;

                // Helper function to transform world point to local space
                let world_to_local = |world_point: Vec2| -> Vec2 {
                    let relative_pos = world_point - current_center;
                    let inv_rotation = rotation.inverse();
                    let rotated = inv_rotation * Vec3::new(relative_pos.x, relative_pos.y, 0.0);
                    Vec2::new(rotated.x, rotated.y)
                };

                // Extract vertex positions from control points and convert to local space
                let mut local_vertices = Vec::new();
                for control_point in &edit_state.control_points {
                    if control_point.point_type == ControlPointType::Vertex {
                        local_vertices.push(world_to_local(control_point.position));
                    }
                }

                // Ensure we have at least 3 vertices for a valid polygon
                if local_vertices.len() >= 3 {
                    // Calculate the centroid of the polygon
                    let centroid = local_vertices.iter().fold(Vec2::ZERO, |acc, &v| acc + v)
                        / local_vertices.len() as f32;

                    // Center the vertices around the origin (relative to centroid)
                    let centered_vertices: Vec<Vec2> =
                        local_vertices.iter().map(|&v| v - centroid).collect();

                    // Check if the polygon has sufficient area (avoid degenerate polygons)
                    let mut area = 0.0;
                    for i in 0..centered_vertices.len() {
                        let j = (i + 1) % centered_vertices.len();
                        area += centered_vertices[i].x * centered_vertices[j].y;
                        area -= centered_vertices[j].x * centered_vertices[i].y;
                    }
                    area = area.abs() * 0.5;

                    if area > 1.0 {
                        // Update the transform to position the polygon at the centroid
                        transform.translation = Vec3::new(
                            current_center.x + centroid.x,
                            current_center.y + centroid.y,
                            transform.translation.z,
                        );

                        // Convert Vec2 to avian2d::math::Vector for convex hull creation
                        let avian_vertices: Vec<avian2d::math::Vector> = centered_vertices
                            .iter()
                            .map(|&v| avian2d::math::Vector::new(v.x, v.y))
                            .collect();

                        // Create new polygon collider using convex hull
                        if let Some(new_collider) = Collider::convex_hull(avian_vertices) {
                            *collider = new_collider;
                        }
                    }
                }
            }
        }
    }
}

/// Handle undo operation
fn handle_undo(
    commands: &mut Commands,
    edit_state: &mut ColliderEditState,
    selected_entity: Entity,
    current_transform: &Transform,
    current_collider: &Collider,
    collider_type: &ColliderType,
) {
    if let Some(previous_state) = edit_state.edit_history.undo() {
        // Store current state for redo
        let current_state = ColliderData {
            transform: *current_transform,
            collider: current_collider.clone(),
            collider_type: *collider_type,
        };
        edit_state.edit_history.push_redo(current_state);

        // Apply previous state
        apply_collider_data(commands, selected_entity, &previous_state);

        // Regenerate control points using the applied data
        generate_control_points(
            edit_state,
            &previous_state.transform,
            &previous_state.collider,
            &previous_state.collider_type,
        );
    }
}

/// Handle redo operation
fn handle_redo(
    commands: &mut Commands,
    edit_state: &mut ColliderEditState,
    selected_entity: Entity,
    current_transform: &Transform,
    current_collider: &Collider,
    collider_type: &ColliderType,
) {
    if let Some(redo_state) = edit_state.edit_history.redo() {
        // Store current state for undo
        let current_state = ColliderData {
            transform: *current_transform,
            collider: current_collider.clone(),
            collider_type: *collider_type,
        };
        edit_state.edit_history.push_state(current_state);

        // Apply redo state
        apply_collider_data(commands, selected_entity, &redo_state);

        // Regenerate control points
        generate_control_points(
            edit_state,
            &redo_state.transform,
            &redo_state.collider,
            &redo_state.collider_type,
        );
    }
}

/// Handle reset operation
fn handle_reset(
    commands: &mut Commands,
    edit_state: &mut ColliderEditState,
    selected_entity: Entity,
    current_transform: &Transform,
    current_collider: &Collider,
    collider_type: &ColliderType,
) {
    if let Some(original_data) = edit_state.original_collider_data.clone() {
        // Store current state for undo
        let current_state = ColliderData {
            transform: *current_transform,
            collider: current_collider.clone(),
            collider_type: *collider_type,
        };
        edit_state.edit_history.push_state(current_state);

        // Apply original state
        apply_collider_data(commands, selected_entity, &original_data);

        // Regenerate control points
        generate_control_points(
            edit_state,
            &original_data.transform,
            &original_data.collider,
            &original_data.collider_type,
        );
    }
}

/// Apply collider data to an entity
fn apply_collider_data(commands: &mut Commands, entity: Entity, data: &ColliderData) {
    commands
        .entity(entity)
        .insert(data.transform)
        .insert(data.collider.clone());
}

/// System called when entering Edit mode
pub(super) fn on_enter_edit_mode(
    mut commands: Commands,
    mut state: ResMut<ColliderCreationState>,
    mut edit_state: ResMut<ColliderEditState>,
    mut selection: ResMut<EditorSelection>,
    collider_query: Query<(Entity, &Transform, &Collider, &ColliderType)>,
) {
    info!("Entering Edit mode");

    // Clear any ongoing creation state
    state.preview_collider = None;
    state.triangle_creation_step = None;
    state.triangle_base_edge = None;

    // Reset edit state except selection
    clear_selection(&mut edit_state, &mut selection);

    // Generate control points for the selected collider if any
    if let Some(selected_entity) = selection.primary() {
        if let Ok((_, transform, collider, created_collider)) = collider_query.get(selected_entity)
        {
            generate_control_points(&mut edit_state, transform, collider, created_collider);

            // Spawn control point entities for visualization
            for (index, control_point) in edit_state.control_points.iter().enumerate() {
                let control_point_entity = commands
                    .spawn((
                        Transform::from_translation(control_point.position.extend(1.0)),
                        ControlPointMarker {
                            index,
                            point_type: control_point.point_type.clone(),
                        },
                        ControlPointEntity, // Add marker to prevent selection conflicts
                        // Add picking components to ensure proper interaction
                        bevy::picking::Pickable::default(),
                        // IMPORTANT: Control points must NOT have Selectable component
                        // to prevent selection system from interfering with drag operations
                    ))
                    .id();

                info!(
                    "Initial control point spawn: entity {:?} for index {} at position {:?}",
                    control_point_entity, index, control_point.position
                );
            }
            info!(
                "Generated {} control points for entity {:?}",
                edit_state.control_points.len(),
                selected_entity
            );
        }
    }
}

/// System called when exiting Edit mode
pub(super) fn on_exit_edit_mode(
    mut commands: Commands,
    mut edit_state: ResMut<ColliderEditState>,
    mut selection: ResMut<EditorSelection>,
    control_point_query: Query<Entity, With<ControlPointEntity>>,
) {
    info!("Exiting Edit mode");

    // Clean up control point markers
    for entity in control_point_query.iter() {
        commands.entity(entity).despawn();
    }

    // Clear edit state
    clear_selection(&mut edit_state, &mut selection);
}

/// Plugin for collider editing functionality
#[derive(Default)]
pub struct EditPlugin;

impl Plugin for EditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(super::ToolMode::Edit), on_enter_edit_mode)
            .add_systems(OnExit(super::ToolMode::Edit), on_exit_edit_mode)
            .add_systems(
                Update,
                (
                    handle_control_point_interaction,
                    update_control_points,
                    super::update_edit_visualization,
                )
                    .run_if(in_state(super::ToolMode::Edit).and(not(egui_wants_any_input))),
            )
            .add_observer(handle_control_point_selection);
    }
}

/// Enhanced control point selection handler
///
/// Handles selection of control points in Edit mode with sophisticated
/// interaction patterns similar to professional editing tools.
pub fn handle_control_point_selection(
    mut trigger: Trigger<Pointer<DragCancelClick>>,
    mut edit_state: ResMut<ColliderEditState>,
    control_point_query: Query<&ControlPointMarker>,
) {
    // Only handle primary button clicks
    if trigger.button != PointerButton::Primary {
        return;
    }

    let target = trigger.target();

    // Check if target is a control point
    if let Ok(control_point_marker) = control_point_query.get(target) {
        info!(
            "ControlPoint selection: Entity {:?} is a control point (index: {}, type: {:?})",
            target, control_point_marker.index, control_point_marker.point_type
        );

        // Start dragging the control point
        edit_state.dragging_point = Some(control_point_marker.index);

        // Prevent event propagation to avoid selection conflicts
        trigger.propagate(false);

        // Store original state for undo functionality
        if edit_state.original_collider_data.is_none() {
            // This will be set when we start editing
            info!("ControlPoint selection: Preparing for edit operation");
        }
    }
}

/// Clear the current selection
fn clear_selection(edit_state: &mut ColliderEditState, selection: &mut EditorSelection) {
    info!("Clearing current selection and resetting edit state");
    // Clear selection using EditorSelection
    // Visual feedback is now handled by gizmos in draw_selected_collider_outlines
    selection.clear();
    edit_state.control_points.clear();
    edit_state.last_selected_entity = None;
    edit_state.dragging_point = None;
    edit_state.original_collider_data = None;
    edit_state.editing_entity = None;
}
