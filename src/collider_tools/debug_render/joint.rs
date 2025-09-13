//! Debug rendering for joints
//!
//! Provides visualization for joint connections and constraints.
//! Uses Bevy's relationship system for managing entity relationships.

use super::super::utils::calculate_anchor_world_position_from_anchor;
use super::super::visualization::draw_dashed_line;
use crate::debug_render::anchor::AnchorPoint;
use crate::joint_config::JointConfigurationEnum;
use crate::selection::EditorSelection;
use avian2d::prelude::*;
use bevy::prelude::*;

use super::EditorGizmoConfigGroup;

/// Component for joint visualization
#[derive(Component, Debug, Clone)]
pub struct JointVisualization {
    /// First anchor point entity
    pub anchor_a: Entity,
    /// Second anchor point entity
    pub anchor_b: Entity,
    /// Joint type for visualization
    pub joint_type: JointType,
    /// Whether this joint is selected
    pub selected: bool,
}

/// Relationship component linking a visualization entity to its joint entity.
/// This is the source of truth for the visualization-joint relationship.
#[derive(Component, Debug, Clone)]
#[relationship(relationship_target = VisualizedBy)]
pub struct JointVisualizationOf(pub Entity);

/// Relationship target component tracking which visualization entity belongs to a joint.
/// This component is updated reactively and should not be modified directly.
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = JointVisualizationOf)]
pub struct VisualizedBy(Vec<Entity>);

impl VisualizedBy {
    /// Get the visualization entity if one exists (should only be one)
    pub fn get(&self) -> Option<Entity> {
        self.0.first().copied()
    }
}

/// Relationship component linking an anchor entity to a joint entity that uses it.
#[derive(Component, Debug, Clone)]
#[relationship(relationship_target = UsesAnchors)]
pub struct AnchorUsedBy(pub Entity);

/// Relationship target component tracking which anchors a joint uses.
/// This component is updated reactively and should not be modified directly.
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = AnchorUsedBy)]
pub struct UsesAnchors(Vec<Entity>);

impl UsesAnchors {
    /// Get the list of anchor entities
    pub fn get(&self) -> &[Entity] {
        &self.0
    }

    /// Check if this joint uses a specific anchor
    pub fn contains(&self, anchor: Entity) -> bool {
        self.0.contains(&anchor)
    }
}

/// Component storing joint configuration data
#[derive(Component, Debug, Clone)]
pub struct JointConfig {
    /// First anchor entity
    pub anchor_a: Entity,
    /// Second anchor entity
    pub anchor_b: Entity,
    /// Whether anchor_a is actually an anchor (false = origin)
    pub anchor_a_is_anchor: bool,
    /// Whether anchor_b is actually an anchor (false = origin)
    pub anchor_b_is_anchor: bool,
    /// Parent entity for the joint
    pub parent_entity: Entity,
    /// Child entity for the joint
    pub child_entity: Entity,
    /// Joint configuration details
    pub joint_config_details: JointConfigurationEnum,
}

/// Types of joints for visualization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JointType {
    /// Distance joint - maintains fixed distance
    #[default]
    Distance,
    /// Revolute joint - allows rotation around anchor
    Revolute,
    /// Prismatic joint - allows sliding along axis
    Prismatic,
    /// Fixed joint - locks relative position and rotation
    Fixed,
}

impl JointType {
    pub fn display_name(&self) -> &'static str {
        match self {
            JointType::Distance => "Distance Joint",
            JointType::Revolute => "Revolute Joint",
            JointType::Prismatic => "Prismatic Joint",
            JointType::Fixed => "Fixed Joint",
        }
    }
}

/// Plugin for joint debug rendering
#[derive(Default)]
pub struct JointDebugRenderPlugin;

impl Plugin for JointDebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (draw_joints, draw_joint_constraints)
                .run_if(in_state(crate::collider_tools::ToolMode::Joint)),
        )
        .add_observer(cleanup_joint_on_remove);
    }
}

/// Draw joint connections and visual elements
pub fn draw_joints(
    mut gizmos: Gizmos<EditorGizmoConfigGroup>,
    joint_query: Query<(Entity, &JointVisualization)>,
    anchor_query: Query<(&AnchorPoint, &GlobalTransform), With<AnchorPoint>>,
    collider_query: Query<&GlobalTransform, With<Collider>>,
    selection: Res<EditorSelection>,
    time: Res<Time>,
) {
    let time_offset = time.elapsed_secs();

    for (entity, joint) in joint_query.iter() {
        let is_selected = selection.contains(entity);

        // Get positions - handle both anchors and collider origins
        let (pos_a, pos_b) = get_joint_positions(&joint, &anchor_query, &collider_query);

        if let (Some(pos_a), Some(pos_b)) = (pos_a, pos_b) {
            // Color based on joint type and selection
            let color = match joint.joint_type {
                JointType::Distance => Color::srgb(0.2, 0.8, 0.8), // Cyan
                JointType::Revolute => Color::srgb(0.8, 0.2, 0.8), // Magenta
                JointType::Prismatic => Color::srgb(0.8, 0.8, 0.2), // Yellow
                JointType::Fixed => Color::srgb(0.5, 0.5, 0.5),    // Gray
            };

            let final_color = if is_selected {
                Color::srgb(1.0, 1.0, 0.0) // Yellow for selected
            } else {
                color
            };

            // Draw joint connection
            draw_joint_connection(
                &mut gizmos,
                pos_a,
                pos_b,
                joint.joint_type,
                final_color,
                time_offset,
            );
        }
    }
}

/// Get positions for joint visualization, handling both anchors and collider origins
fn get_joint_positions(
    joint: &JointVisualization,
    anchor_query: &Query<(&AnchorPoint, &GlobalTransform), With<AnchorPoint>>,
    collider_query: &Query<&GlobalTransform, With<Collider>>,
) -> (Option<Vec2>, Option<Vec2>) {
    let pos_a = if anchor_query.contains(joint.anchor_a) {
        // Calculate the actual anchor position using the same method as anchor.rs
        anchor_query
            .get(joint.anchor_a)
            .ok()
            .and_then(|(anchor, _)| {
                // Need to get the parent collider transform to calculate anchor position
                collider_query
                    .get(anchor.parent_entity)
                    .ok()
                    .map(|collider_transform| {
                        calculate_anchor_world_position_from_anchor(anchor, collider_transform)
                    })
            })
    } else {
        collider_query
            .get(joint.anchor_a)
            .ok()
            .map(|transform| transform.translation().truncate())
    };

    let pos_b = if anchor_query.contains(joint.anchor_b) {
        // Calculate the actual anchor position using the same method as anchor.rs
        anchor_query
            .get(joint.anchor_b)
            .ok()
            .and_then(|(anchor, _)| {
                // Need to get the parent collider transform to calculate anchor position
                collider_query
                    .get(anchor.parent_entity)
                    .ok()
                    .map(|collider_transform| {
                        calculate_anchor_world_position_from_anchor(anchor, collider_transform)
                    })
            })
    } else {
        collider_query
            .get(joint.anchor_b)
            .ok()
            .map(|transform| transform.translation().truncate())
    };

    (pos_a, pos_b)
}

/// Draw joint constraints and limits
pub fn draw_joint_constraints(
    mut gizmos: Gizmos<EditorGizmoConfigGroup>,
    joint_query: Query<&JointVisualization>,
    anchor_query: Query<(&AnchorPoint, &GlobalTransform), With<AnchorPoint>>,
    collider_query: Query<&GlobalTransform, With<Collider>>,
    time: Res<Time>,
) {
    let time_offset = time.elapsed_secs();

    for joint in joint_query.iter() {
        // Get positions - handle both anchors and collider origins
        let (pos_a, pos_b) = get_joint_positions(&joint, &anchor_query, &collider_query);

        if let (Some(pos_a), Some(pos_b)) = (pos_a, pos_b) {
            // Draw type-specific constraint visualizations
            match joint.joint_type {
                JointType::Distance => {
                    // Distance joint - no additional visualization needed
                }
                JointType::Revolute => {
                    // Revolute joint - no additional visualization needed
                }
                JointType::Prismatic => {
                    // Draw sliding axis
                    let direction = (pos_b - pos_a).normalize();
                    let axis_length = 40.0;

                    draw_dashed_line(
                        &mut gizmos,
                        pos_a - direction * axis_length,
                        pos_a + direction * axis_length,
                        Color::srgba(0.8, 0.8, 0.2, 0.3),
                        time_offset,
                    );
                }
                JointType::Fixed => {
                    // Draw rigid connection indicators
                    let mid_point = (pos_a + pos_b) / 2.0;
                    let offset = (pos_b - pos_a).perp().normalize() * 10.0;

                    gizmos.line_2d(
                        mid_point - offset,
                        mid_point + offset,
                        Color::srgba(0.5, 0.5, 0.5, 0.5),
                    );
                }
            }
        }
    }
}

/// Draw joint connection based on type
fn draw_joint_connection<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    pos_a: Vec2,
    pos_b: Vec2,
    joint_type: JointType,
    color: Color,
    time_offset: f32,
) {
    match joint_type {
        JointType::Distance => {
            // Simple line for distance joint
            draw_dashed_line(gizmos, pos_a, pos_b, color, time_offset);
        }
        JointType::Revolute => {
            // Direct connection for revolute joint (removed circle)
            draw_dashed_line(gizmos, pos_a, pos_b, color, time_offset);
        }
        JointType::Prismatic => {
            // Parallel lines for prismatic joint
            let direction = (pos_b - pos_a).normalize();
            let perpendicular = Vec2::new(-direction.y, direction.x);
            let offset = perpendicular * 5.0;

            draw_dashed_line(gizmos, pos_a + offset, pos_b + offset, color, time_offset);
            draw_dashed_line(gizmos, pos_a - offset, pos_b - offset, color, time_offset);
        }
        JointType::Fixed => {
            // Double line for fixed joint
            let direction = (pos_b - pos_a).normalize();
            let perpendicular = Vec2::new(-direction.y, direction.x);
            let offset = perpendicular * 3.0;

            draw_dashed_line(gizmos, pos_a + offset, pos_b + offset, color, time_offset);
            draw_dashed_line(gizmos, pos_a - offset, pos_b - offset, color, time_offset);
        }
    }
}

/// Helper functions for working with joint relationships using ECS queries
pub mod joint_relationships {
    use super::*;

    /// Get the joint entity for a visualization entity
    pub fn get_joint_for_visualization(
        visualization_entity: Entity,
        joint_viz_query: &Query<&JointVisualizationOf>,
    ) -> Option<Entity> {
        joint_viz_query
            .get(visualization_entity)
            .ok()
            .map(|viz_of| viz_of.0)
    }

    /// Get the visualization entity for a joint entity
    pub fn get_visualization_for_joint(
        joint_entity: Entity,
        visualized_by_query: &Query<&VisualizedBy>,
    ) -> Option<Entity> {
        visualized_by_query
            .get(joint_entity)
            .ok()
            .and_then(|viz_by| viz_by.get())
    }

    /// Get all joints that use a specific anchor
    pub fn get_joints_for_anchor(
        anchor_entity: Entity,
        all_anchor_used_by_query: &Query<(Entity, &AnchorUsedBy)>,
    ) -> Vec<Entity> {
        // Find the AnchorUsedBy relationship component on the specific anchor entity
        // and return the referenced joint entity (if any).
        all_anchor_used_by_query
            .iter()
            .filter_map(|(entity, anchor_used_by)| {
                if entity == anchor_entity {
                    // entity is the anchor; anchor_used_by.0 is the joint entity using this anchor
                    Some(anchor_used_by.0)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Create a joint visualization relationship
    pub fn create_joint_visualization_relationship(
        commands: &mut Commands,
        visualization_entity: Entity,
        joint_entity: Entity,
    ) {
        commands
            .entity(visualization_entity)
            .insert(JointVisualizationOf(joint_entity));
    }

    /// Create anchor usage relationships
    pub fn create_anchor_usage_relationships(
        commands: &mut Commands,
        joint_entity: Entity,
        anchor_entities: &[Entity],
    ) {
        for &anchor_entity in anchor_entities {
            commands
                .entity(anchor_entity)
                .insert(AnchorUsedBy(joint_entity));
        }
    }
}

/// Observer to cleanup Joint entities when JointVisualization is removed
pub fn cleanup_joint_on_remove(
    trigger: Trigger<OnRemove, JointVisualization>,
    mut commands: Commands,
    joint_viz_query: Query<&JointVisualizationOf>,
) {
    let visualization_entity = trigger.target();

    // Find the joint entity through the relationship component
    if let Ok(joint_viz_of) = joint_viz_query.get(visualization_entity) {
        let joint_entity = joint_viz_of.0;
        info!(
            "Despawning joint entity {} due to visualization removal",
            joint_entity
        );
        commands.entity(joint_entity).despawn();
    }
}
