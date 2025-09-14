//! Joint creation and manipulation tools
//!
//! Provides functionality for creating joints between anchors and collider origins.
//!
//! ## Important Concepts
//!
//! ### Origin Points vs Center of Mass
//! - **Origin Points**: The transform origin of a collider (transform.position).
//!   Users can create joints at origin points for simple connections.
//! - **Center of Mass**: Automatically computed by Avian based on collider shape.
//!   Users cannot directly manipulate center of mass - it's calculated by the physics engine.
//!
//! ### Joint Types
//! Joints can be created between:
//! - Anchor ↔ Anchor (custom attachment points on colliders)
//! - Anchor ↔ Origin (anchor point to collider transform origin)
//! - Origin ↔ Origin (collider transform origins)
//!
//! All joint attachments use local anchor positions relative to collider origins,
//! and Avian automatically handles the center of mass offset calculations.

use super::debug_render::{
    anchor::AnchorPoint,
    joint::{AnchorUsedBy, JointConfig, JointVisualization, VisualizedBy, joint_relationships},
};
use super::joint_config::JointConfiguration;
use super::utils::{
    calculate_anchor_world_position_from_anchor, find_collider_at_position_with_spatial_query,
    get_anchor_local_position, get_mouse_world_position,
};
use crate::physics_management::PhysicsManager;
use crate::selection::Selectable;
use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_input;

/// Regenerate a joint with updated anchor positions using relationship system
pub fn regenerate_joint_for_anchor(
    commands: &mut Commands,
    joint_entity: Entity,
    config: &JointConfig,
    anchor_query: &Query<&AnchorPoint>,
    visualized_by_query: &Query<&VisualizedBy>,
) {
    // Get the visualization entity through relationship
    let visualization_entity =
        joint_relationships::get_visualization_for_joint(joint_entity, visualized_by_query);

    // Remove the old joint
    commands.entity(joint_entity).despawn();

    // Get updated anchor positions
    let anchor_a_offset = if config.anchor_a_is_anchor {
        if let Ok(anchor) = anchor_query.get(config.anchor_a) {
            get_anchor_local_position(anchor)
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO // Origin point
    };

    let anchor_b_offset = if config.anchor_b_is_anchor {
        if let Ok(anchor) = anchor_query.get(config.anchor_b) {
            get_anchor_local_position(anchor)
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO // Origin point
    };

    // Create new joint with updated positions
    let new_joint_entity = config.joint_config_details.create_physics_joint(
        commands,
        anchor_a_offset,
        anchor_b_offset,
        config.parent_entity,
        config.child_entity,
    );

    // Add JointConfig component to the new joint
    commands.entity(new_joint_entity).insert(config.clone());

    // Re-establish anchor relationships
    let mut anchor_entities = Vec::new();
    if config.anchor_a_is_anchor {
        anchor_entities.push(config.anchor_a);
    }
    if config.anchor_b_is_anchor {
        anchor_entities.push(config.anchor_b);
    }
    if !anchor_entities.is_empty() {
        joint_relationships::create_anchor_usage_relationships(
            commands,
            new_joint_entity,
            &anchor_entities,
        );
    }

    // Update the relationship if we have a visualization entity
    if let Some(viz_entity) = visualization_entity {
        joint_relationships::create_joint_visualization_relationship(
            commands,
            viz_entity,
            new_joint_entity,
        );
    }
}

/// Regenerate all joints associated with a specific anchor using relationship system
pub fn regenerate_joints_for_anchor(
    commands: &mut Commands,
    anchor_entity: Entity,
    anchor_query: &Query<&AnchorPoint>,
    all_anchor_used_by_query: &Query<(Entity, &AnchorUsedBy)>,
    joint_config_query: &Query<&JointConfig>,
    visualized_by_query: &Query<&VisualizedBy>,
) {
    let joint_entities =
        joint_relationships::get_joints_for_anchor(anchor_entity, all_anchor_used_by_query);

    for joint_entity in joint_entities {
        if let Ok(config) = joint_config_query.get(joint_entity) {
            regenerate_joint_for_anchor(
                commands,
                joint_entity,
                config,
                anchor_query,
                visualized_by_query,
            );
        }
    }
}

/// Delete all joints associated with a specific anchor using relationship system
pub fn delete_joints_for_anchor(
    commands: &mut Commands,
    anchor_entity: Entity,
    all_anchor_used_by_query: &Query<(Entity, &AnchorUsedBy)>,
    visualized_by_query: &Query<&VisualizedBy>,
) {
    let joint_entities =
        joint_relationships::get_joints_for_anchor(anchor_entity, all_anchor_used_by_query);

    for joint_entity in joint_entities {
        // Get the visualization entity before removing the joint
        if let Some(visualization_entity) =
            joint_relationships::get_visualization_for_joint(joint_entity, visualized_by_query)
        {
            // Only remove the visualization entity - the observer will handle joint entity deletion
            commands.entity(visualization_entity).despawn();
            info!(
                "Deleted joint {:?} associated with anchor {:?}",
                joint_entity, anchor_entity
            );
        }
    }
}

/// Regenerate all joints associated with a specific anchor (with visualization query) using relationship system
pub fn regenerate_joints_for_anchor_with_viz(
    commands: &mut Commands,
    anchor_entity: Entity,
    anchor_query: &Query<(&AnchorPoint, &Transform)>,
    all_anchor_used_by_query: &Query<(Entity, &AnchorUsedBy)>,
    joint_config_query: &Query<&JointConfig>,
    visualized_by_query: &Query<&VisualizedBy>,
) {
    let joint_entities =
        joint_relationships::get_joints_for_anchor(anchor_entity, all_anchor_used_by_query);

    for joint_entity in joint_entities {
        if let Ok(config) = joint_config_query.get(joint_entity) {
            // Get the visualization entity through relationship
            let visualization_entity =
                joint_relationships::get_visualization_for_joint(joint_entity, visualized_by_query);

            // Remove the old joint
            commands.entity(joint_entity).despawn();

            // Get updated anchor positions
            let anchor_a_offset = if config.anchor_a_is_anchor {
                if let Ok((anchor, _)) = anchor_query.get(config.anchor_a) {
                    get_anchor_local_position(anchor)
                } else {
                    Vec2::ZERO
                }
            } else {
                Vec2::ZERO // Origin point
            };

            let anchor_b_offset = if config.anchor_b_is_anchor {
                if let Ok((anchor, _)) = anchor_query.get(config.anchor_b) {
                    get_anchor_local_position(anchor)
                } else {
                    Vec2::ZERO
                }
            } else {
                Vec2::ZERO // Origin point
            };

            // Create new joint with updated positions
            let new_joint_entity = config.joint_config_details.create_physics_joint(
                commands,
                anchor_a_offset,
                anchor_b_offset,
                config.parent_entity,
                config.child_entity,
            );

            // Add JointConfig component to the new joint
            commands.entity(new_joint_entity).insert(config.clone());

            // Re-establish anchor relationships
            let mut anchor_entities = Vec::new();
            if config.anchor_a_is_anchor {
                anchor_entities.push(config.anchor_a);
            }
            if config.anchor_b_is_anchor {
                anchor_entities.push(config.anchor_b);
            }
            if !anchor_entities.is_empty() {
                joint_relationships::create_anchor_usage_relationships(
                    commands,
                    new_joint_entity,
                    &anchor_entities,
                );
            }

            // Update the relationship if we have a visualization entity
            if let Some(viz_entity) = visualization_entity {
                joint_relationships::create_joint_visualization_relationship(
                    commands,
                    viz_entity,
                    new_joint_entity,
                );
            }
        }
    }
}

// Re-use visualization functions from the visualization module
// use super::visualization::draw_preview_joint; // Currently unused

/// State for joint creation and manipulation
#[derive(Resource, Default, Debug, Clone, Reflect)]
#[reflect(Resource, Default)]
pub struct JointCreationState {
    /// Whether we're currently dragging to create a joint
    pub is_dragging: bool,
    /// Starting point of the drag (anchor or center of mass)
    pub drag_start_entity: Option<Entity>,
    /// Starting position of the drag
    pub drag_start_pos: Option<Vec2>,
    /// Current mouse position during drag
    pub drag_current_pos: Option<Vec2>,
    /// Type of the starting point (anchor)
    pub drag_start_type: DragPointType,
}

/// Type of point being dragged from/to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
pub enum DragPointType {
    #[default]
    /// Anchor point - user-defined attachment point on a collider
    Anchor,
    /// Origin point - collider transform origin (transform.position)
    Origin,
}

/// Plugin for joint creation and manipulation
#[derive(Default)]
pub struct JointCreationPlugin;

impl Plugin for JointCreationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JointCreationState>()
            .init_resource::<JointConfiguration>()
            .add_systems(
                OnEnter(crate::collider_tools::ToolMode::Joint),
                on_enter_joint_mode,
            )
            .add_systems(
                OnExit(crate::collider_tools::ToolMode::Joint),
                on_exit_joint_mode,
            )
            .add_systems(
                Update,
                (handle_joint_mode_input, update_joint_preview).run_if(
                    in_state(crate::collider_tools::ToolMode::Joint).and(not(egui_wants_any_input)),
                ),
            );
    }
}

/// Initialize joint mode
pub(super) fn on_enter_joint_mode(
    mut joint_state: ResMut<JointCreationState>,
    mut physics_manager: ResMut<PhysicsManager>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    info!("Entering Joint mode");
    joint_state.is_dragging = false;
    joint_state.drag_start_entity = None;
    joint_state.drag_start_pos = None;
    joint_state.drag_current_pos = None;
    joint_state.drag_start_type = DragPointType::Anchor;

    physics_manager.pause(&mut physics_time);
}

/// Cleanup when exiting joint mode
pub(super) fn on_exit_joint_mode(
    mut joint_state: ResMut<JointCreationState>,
    mut physics_manager: ResMut<PhysicsManager>,
    mut physics_time: ResMut<Time<Physics>>,
) {
    info!("Exiting Joint mode");

    // Clean up drag state
    joint_state.is_dragging = false;
    joint_state.drag_start_entity = None;
    joint_state.drag_start_pos = None;
    joint_state.drag_current_pos = None;
    joint_state.drag_start_type = DragPointType::Anchor;

    // Resume physics when exiting joint mode
    physics_manager.unpause(&mut physics_time);
}

/// Handle input for joint mode
fn handle_joint_mode_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut joint_state: ResMut<JointCreationState>,
    joint_config: Res<JointConfiguration>,
    mut commands: Commands,
    mut anchor_query: Query<(Entity, &mut AnchorPoint, &GlobalTransform)>,
    collider_query: Query<(Entity, &GlobalTransform, &Collider)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
    window_query: Query<&Window>,
) {
    // Cancel drag with Escape
    if keyboard_input.just_pressed(KeyCode::Escape) {
        if joint_state.is_dragging {
            joint_state.is_dragging = false;
            joint_state.drag_start_entity = None;
            joint_state.drag_start_pos = None;
            joint_state.drag_current_pos = None;
            info!("Cancelled joint creation drag");
        }
    }

    // Get current mouse position
    let current_mouse_pos = if let (Ok((camera, camera_transform)), Ok(window)) =
        (camera_query.single(), window_query.single())
    {
        get_mouse_world_position(&window, &camera, camera_transform)
    } else {
        None
    };

    // Start dragging on mouse press
    if mouse_input.just_pressed(MouseButton::Left) && !joint_state.is_dragging {
        if let Some(world_pos) = current_mouse_pos {
            // Check if clicking on an anchor
            for (entity, anchor, _transform) in anchor_query.iter() {
                // Calculate the actual anchor position using the same method as anchor.rs
                let calculated_anchor_pos = if let Ok((_, collider_transform, _)) =
                    collider_query.get(anchor.parent_entity)
                {
                    calculate_anchor_world_position_from_anchor(anchor, &collider_transform)
                } else {
                    continue; // Skip if we can't get the parent collider transform
                };
                let distance = world_pos.distance(calculated_anchor_pos);

                if distance <= anchor.radius {
                    joint_state.is_dragging = true;
                    joint_state.drag_start_entity = Some(entity);
                    joint_state.drag_start_pos = Some(calculated_anchor_pos);
                    joint_state.drag_current_pos = Some(world_pos);
                    joint_state.drag_start_type = DragPointType::Anchor;
                    info!("Started dragging from anchor");
                    return;
                }
            }

            // Check if clicking on a collider origin
            for (entity, transform, _) in collider_query.iter() {
                let collider_pos = transform.translation().truncate();
                let distance = world_pos.distance(collider_pos);

                if distance <= 10.0 {
                    // 10 pixel threshold for origin detection
                    joint_state.is_dragging = true;
                    joint_state.drag_start_entity = Some(entity);
                    joint_state.drag_start_pos = Some(collider_pos);
                    joint_state.drag_current_pos = Some(world_pos);
                    joint_state.drag_start_type = DragPointType::Origin;
                    info!("Started dragging from collider origin");
                    return;
                }
            }
        }
    }

    // Update drag position
    if joint_state.is_dragging {
        if let Some(world_pos) = current_mouse_pos {
            joint_state.drag_current_pos = Some(world_pos);
        }
    }

    // End dragging and create joint on mouse release
    if mouse_input.just_released(MouseButton::Left) && joint_state.is_dragging {
        if let Some(world_pos) = current_mouse_pos {
            let mut joint_created = false;

            // Check if releasing over an anchor
            for (entity, anchor, _transform) in anchor_query.iter() {
                // Calculate the actual anchor position using the same method as anchor.rs
                let calculated_anchor_pos = if let Ok((_, collider_transform, _)) =
                    collider_query.get(anchor.parent_entity)
                {
                    calculate_anchor_world_position_from_anchor(anchor, &collider_transform)
                } else {
                    continue; // Skip if we can't get the parent collider transform
                };
                let distance = world_pos.distance(calculated_anchor_pos);

                if distance <= anchor.radius && Some(entity) != joint_state.drag_start_entity {
                    // Extract anchor data for both start and target entities
                    let start_anchor_data =
                        if let Some(start_entity) = joint_state.drag_start_entity {
                            if let Ok(anchor_comp) = anchor_query.get(start_entity) {
                                Some((
                                    anchor_comp.1.parent_entity,
                                    get_anchor_local_position(&anchor_comp.1),
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                    let target_anchor_data = if let Ok(anchor_comp) = anchor_query.get(entity) {
                        Some((
                            anchor_comp.1.parent_entity,
                            get_anchor_local_position(&anchor_comp.1),
                        ))
                    } else {
                        None
                    };

                    // Mark anchors as being in a joint
                    if let Some(start_entity) = joint_state.drag_start_entity {
                        if let Ok(mut anchor_comp) = anchor_query.get_mut(start_entity) {
                            anchor_comp.1.in_joint = true;
                        }
                    }
                    if let Ok(mut anchor_comp) = anchor_query.get_mut(entity) {
                        anchor_comp.1.in_joint = true;
                    }

                    if let Some(start_entity) = joint_state.drag_start_entity {
                        create_joint_from_drag(
                            &mut commands,
                            start_entity,
                            entity,
                            joint_state.drag_start_type,
                            true, // is_target_anchor
                            start_anchor_data,
                            target_anchor_data,
                            &joint_config,
                        );
                    }
                    joint_created = true;
                    break;
                }
            }

            // Check if releasing over a collider origin
            if !joint_created {
                for (entity, transform, _) in collider_query.iter() {
                    let collider_pos = transform.translation().truncate();
                    let distance = world_pos.distance(collider_pos);

                    if distance <= 10.0 && Some(entity) != joint_state.drag_start_entity {
                        // Extract anchor data if start entity is an anchor
                        let start_anchor_data =
                            if let Some(start_entity) = joint_state.drag_start_entity {
                                if joint_state.drag_start_type == DragPointType::Anchor {
                                    if let Ok(anchor_comp) = anchor_query.get(start_entity) {
                                        Some((
                                            anchor_comp.1.parent_entity,
                                            get_anchor_local_position(&anchor_comp.1),
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                        // Mark start anchor as being in a joint if it's an anchor
                        if let Some(start_entity) = joint_state.drag_start_entity {
                            if joint_state.drag_start_type == DragPointType::Anchor {
                                if let Ok(mut anchor_comp) = anchor_query.get_mut(start_entity) {
                                    anchor_comp.1.in_joint = true;
                                }
                            }
                        }

                        if let Some(start_entity) = joint_state.drag_start_entity {
                            create_joint_from_drag(
                                &mut commands,
                                start_entity,
                                entity,
                                joint_state.drag_start_type,
                                false, // is_target_anchor
                                start_anchor_data,
                                None, // target is origin, no anchor data
                                &joint_config,
                            );
                        }
                        joint_created = true;
                        break;
                    }
                }
            }

            if joint_created {
                info!("Created joint via drag");
            } else {
                info!("Drag cancelled - no valid target");
            }
        }

        // Reset drag state
        joint_state.is_dragging = false;
        joint_state.drag_start_entity = None;
        joint_state.drag_start_pos = None;
        joint_state.drag_current_pos = None;
    }
}

/// Update joint preview during creation
fn update_joint_preview(
    mut gizmos: Gizmos,
    joint_state: Res<JointCreationState>,
    time: Res<Time>,
    anchor_query: Query<(Entity, &AnchorPoint, &GlobalTransform)>,
    collider_query: Query<(Entity, &GlobalTransform, &Collider)>,
    spatial_query: SpatialQuery,
) {
    if joint_state.is_dragging {
        if let (Some(start_pos), Some(current_pos)) =
            (joint_state.drag_start_pos, joint_state.drag_current_pos)
        {
            let color = Color::srgba(1.0, 1.0, 0.0, 0.8); // Yellow with transparency

            // Draw the preview line
            gizmos.line_2d(start_pos, current_pos, color);

            // Draw connection points
            gizmos.circle_2d(start_pos, 5.0, color);
            gizmos.circle_2d(current_pos, 3.0, color);

            // Check for nearby connection targets and highlight them
            let detection_radius = 15.0;

            // Check anchors
            for (entity, anchor, _transform) in anchor_query.iter() {
                if Some(entity) != joint_state.drag_start_entity {
                    // Calculate the actual anchor position using the same method as anchor.rs
                    let calculated_anchor_pos = if let Ok((_, collider_transform, _)) =
                        collider_query.get(anchor.parent_entity)
                    {
                        calculate_anchor_world_position_from_anchor(anchor, &collider_transform)
                    } else {
                        continue; // Skip if we can't get the parent collider transform
                    };
                    let distance = current_pos.distance(calculated_anchor_pos);

                    if distance <= detection_radius {
                        // Highlight detected anchor
                        let highlight_color = Color::srgba(0.0, 1.0, 0.0, 0.8); // Green highlight
                        let pulse_size = anchor.radius + 3.0 * (time.elapsed_secs() * 5.0).sin();
                        gizmos.circle_2d(
                            calculated_anchor_pos,
                            pulse_size.max(anchor.radius),
                            highlight_color,
                        );
                    }
                }
            }

            // Check collider origins
            let filter = SpatialQueryFilter::default();
            if let Some(nearby_collider) =
                find_collider_at_position_with_spatial_query(current_pos, &spatial_query, &filter)
            {
                if Some(nearby_collider) != joint_state.drag_start_entity {
                    if let Ok((_, transform, _)) = collider_query.get(nearby_collider) {
                        let collider_pos = transform.translation().truncate();
                        let distance = current_pos.distance(collider_pos);

                        if distance <= detection_radius {
                            // Highlight detected collider origin
                            let highlight_color = Color::srgba(0.0, 1.0, 0.0, 0.8); // Green highlight
                            let pulse_size = 8.0 + 3.0 * (time.elapsed_secs() * 5.0).sin();
                            gizmos.circle_2d(collider_pos, pulse_size, highlight_color);
                        }
                    }
                }
            }
        }
    }
}

/// Create a joint from drag operation using relationship system
fn create_joint_from_drag(
    commands: &mut Commands,
    start_entity: Entity,
    target_entity: Entity,
    start_type: DragPointType,
    is_target_anchor: bool,
    start_anchor_data: Option<(Entity, Vec2)>,
    target_anchor_data: Option<(Entity, Vec2)>,
    joint_config: &JointConfiguration,
) {
    match start_type {
        DragPointType::Anchor => {
            // Started from anchor, check if target is anchor or origin
            if is_target_anchor {
                // Anchor to anchor joint
                if let (Some((start_parent, start_offset)), Some((target_parent, target_offset))) =
                    (start_anchor_data, target_anchor_data)
                {
                    create_joint_between_anchors(
                        commands,
                        start_entity,
                        target_entity,
                        start_parent,
                        target_parent,
                        start_offset,
                        target_offset,
                        joint_config,
                    );
                }
            } else {
                // Anchor to origin joint
                if let Some((anchor_parent_entity, anchor_offset)) = start_anchor_data {
                    create_anchor_to_origin_joint(
                        commands,
                        start_entity,
                        target_entity,
                        anchor_parent_entity,
                        anchor_offset,
                        joint_config,
                    );
                }
            }
        }
        DragPointType::Origin => {
            // Started from origin, check if target is anchor or origin
            if is_target_anchor {
                // Origin to anchor joint
                if let Some((anchor_parent_entity, anchor_offset)) = target_anchor_data {
                    create_anchor_to_origin_joint(
                        commands,
                        target_entity,
                        start_entity,
                        anchor_parent_entity,
                        anchor_offset,
                        joint_config,
                    );
                }
            } else {
                // Origin to origin joint
                create_origin_to_origin_joint(commands, start_entity, target_entity, joint_config);
            }
        }
    }
}

/// Create a joint between an anchor and a collider origin using relationship system
fn create_anchor_to_origin_joint(
    commands: &mut Commands,
    anchor_entity: Entity,
    origin_entity: Entity,
    anchor_parent_entity: Entity,
    anchor_offset: Vec2,
    joint_config: &JointConfiguration,
) {
    // Create the physics joint based on type and capture the entity
    let parent_entity = if anchor_parent_entity != origin_entity {
        anchor_parent_entity
    } else {
        origin_entity
    };
    let child_entity = if anchor_parent_entity != origin_entity {
        origin_entity
    } else {
        anchor_parent_entity
    };
    let joint_config_details = joint_config.to_enum();
    let joint_entity = joint_config_details.create_physics_joint(
        commands,
        anchor_offset,
        Vec2::ZERO,
        parent_entity,
        child_entity,
    );

    // Create joint configuration component
    let config = JointConfig {
        anchor_a: anchor_entity,
        anchor_b: origin_entity,
        anchor_a_is_anchor: true,
        anchor_b_is_anchor: false,
        parent_entity,
        child_entity,
        joint_config_details,
    };
    commands.entity(joint_entity).insert(config);

    // Create joint visualization and establish relationships
    let visualization_entity = commands
        .spawn((
            JointVisualization {
                anchor_a: anchor_entity,
                anchor_b: origin_entity,
                joint_type: joint_config.joint_type,
                selected: false,
            },
            Selectable::default(),
        ))
        .id();

    // Establish relationships using the relationship system
    joint_relationships::create_joint_visualization_relationship(
        commands,
        visualization_entity,
        joint_entity,
    );
    joint_relationships::create_anchor_usage_relationships(
        commands,
        joint_entity,
        &[anchor_entity],
    );

    info!(
        "Created {:?} joint between anchor and origin",
        joint_config.joint_type
    );
}

/// Create a joint between two collider origins using relationship system
fn create_origin_to_origin_joint(
    commands: &mut Commands,
    origin_a: Entity,
    origin_b: Entity,
    joint_config: &JointConfiguration,
) {
    let joint_config_details = joint_config.to_enum();
    // Create the physics joint based on type and capture the entity
    let joint_entity = joint_config_details.create_physics_joint(
        commands,
        Vec2::ZERO,
        Vec2::ZERO,
        origin_a,
        origin_b,
    );

    // Create joint configuration component
    let config = JointConfig {
        anchor_a: origin_a,
        anchor_b: origin_b,
        anchor_a_is_anchor: false,
        anchor_b_is_anchor: false,
        parent_entity: origin_a,
        child_entity: origin_b,
        joint_config_details,
    };
    commands.entity(joint_entity).insert(config);

    // Create joint visualization and establish relationships
    let visualization_entity = commands
        .spawn((
            JointVisualization {
                anchor_a: origin_a,
                anchor_b: origin_b,
                joint_type: joint_config.joint_type,
                selected: false,
            },
            Selectable::default(),
        ))
        .id();

    // Establish relationships using the relationship system
    joint_relationships::create_joint_visualization_relationship(
        commands,
        visualization_entity,
        joint_entity,
    );
    // Origins don't need anchor usage relationships since they're not anchors

    info!(
        "Created {:?} joint between two origins",
        joint_config.joint_type
    );
}

/// Create a joint between two anchors using relationship system
fn create_joint_between_anchors(
    commands: &mut Commands,
    anchor_a: Entity,
    anchor_b: Entity,
    parent_entity: Entity,
    child_entity: Entity,
    anchor_a_offset: Vec2,
    anchor_b_offset: Vec2,
    joint_config: &JointConfiguration,
) {
    let joint_config_details = joint_config.to_enum();
    // Create the physics joint based on type and capture the entity
    let joint_entity = joint_config_details.create_physics_joint(
        commands,
        anchor_a_offset,
        anchor_b_offset,
        parent_entity,
        child_entity,
    );

    // Create joint configuration component
    let config = JointConfig {
        anchor_a,
        anchor_b,
        anchor_a_is_anchor: true,
        anchor_b_is_anchor: true,
        parent_entity,
        child_entity,
        joint_config_details,
    };
    commands.entity(joint_entity).insert(config);

    // Create joint visualization and establish relationships
    let visualization_entity = commands
        .spawn((
            JointVisualization {
                anchor_a,
                anchor_b,
                joint_type: joint_config.joint_type,
                selected: false,
            },
            Selectable::default(),
        ))
        .id();

    // Establish relationships using the relationship system
    joint_relationships::create_joint_visualization_relationship(
        commands,
        visualization_entity,
        joint_entity,
    );
    joint_relationships::create_anchor_usage_relationships(
        commands,
        joint_entity,
        &[anchor_a, anchor_b],
    );

    info!(
        "Created {:?} joint between anchors",
        joint_config.joint_type
    );
}
