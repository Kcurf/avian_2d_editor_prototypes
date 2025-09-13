//! Debug rendering for anchor points
//!
//! Provides visualization for anchor points on colliders.

use super::super::visualization::{draw_dashed_circle, draw_dashed_line};
use crate::selection::EditorSelection;
use avian2d::prelude::*;
use bevy::prelude::*;

/// Component for anchor points
///
/// # Local Anchor vs Center of Mass
///
/// This component stores LOCAL ANCHOR positions, which are:
/// - Specified by users relative to the collider's origin
/// - Used for joint attachment points
/// - Automatically converted by Avian to constraint anchors
///
/// # How Avian Transforms Local Anchors
///
/// Avian physics engine automatically computes:
/// ```rust
/// // What users set: local anchor position (relative to collider origin)
/// let local_anchor = anchor.local_anchor_position;
///
/// // What Avian computes for constraints: anchor relative to center of mass
/// let constraint_anchor = local_anchor - center_of_mass;
///
/// // The center of mass is automatically calculated by Avian based on:
/// // - Collider shape and geometry
/// // - Mass distribution
/// // - Material properties
/// ```
///
/// # Important Notes
///
/// - Users should only manipulate `local_anchor_position`
/// - Center of mass is read-only and computed by Avian
/// - This separation ensures physically accurate constraints
/// - Joint forces are properly applied relative to the center of mass
#[derive(Component, Debug, Clone)]
pub struct AnchorPoint {
    /// Local anchor position relative to the collider's origin
    /// This is what users set to specify where joints should attach
    pub local_anchor_position: Vec2,
    /// Parent collider entity
    pub parent_entity: Entity,
    /// Whether this anchor is being used in a joint
    pub in_joint: bool,
    /// Visual radius for rendering
    pub radius: f32,
}

impl Default for AnchorPoint {
    fn default() -> Self {
        Self {
            local_anchor_position: Vec2::ZERO,
            parent_entity: Entity::PLACEHOLDER,
            in_joint: false,
            radius: 8.0,
        }
    }
}

/// Plugin for anchor debug rendering
#[derive(Default)]
pub struct AnchorDebugRenderPlugin;

impl Plugin for AnchorDebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_anchor_transforms,
                draw_anchor_points,
                draw_anchor_gizmos,
            )
                .chain()
                .run_if(
                    in_state(crate::collider_tools::ToolMode::Anchor)
                        .or(in_state(crate::collider_tools::ToolMode::Joint)),
                ),
        );
    }
}

/// Synchronize anchor transforms with their parent entities
pub fn sync_anchor_transforms(
    mut anchor_query: Query<(&mut Transform, &AnchorPoint)>,
    collider_query: Query<&GlobalTransform, (With<Collider>, Without<AnchorPoint>)>,
) {
    for (mut transform, anchor) in anchor_query.iter_mut() {
        if let Ok(parent_transform) = collider_query.get(anchor.parent_entity) {
            // Calculate world position using the corrected method: parent_pos + rotation * local_anchor
            let parent_pos = parent_transform.translation().truncate();
            let rotated_anchor =
                parent_transform.rotation() * anchor.local_anchor_position.extend(0.0);
            let world_pos = parent_pos + rotated_anchor.xy();

            // Update transform to match calculated world position
            transform.translation = world_pos.extend(0.0);

            trace!(
                "Synced anchor transform: parent_pos={:?}, local_anchor_position={:?}, world_pos={:?}",
                parent_pos, anchor.local_anchor_position, world_pos
            );
        }
    }
}

/// Draw anchor points as visual elements
pub fn draw_anchor_points(
    mut gizmos: Gizmos,
    anchor_query: Query<(Entity, &AnchorPoint)>,
    collider_query: Query<&GlobalTransform, (With<Collider>, Without<AnchorPoint>)>,
    selection: Res<EditorSelection>,
    time: Res<Time>,
) {
    let time_offset = time.elapsed_secs();
    let anchor_count = anchor_query.iter().count();

    if anchor_count > 0 {
        debug!("Drawing {} anchor points", anchor_count);
    }

    for (entity, anchor) in anchor_query.iter() {
        // Calculate the consistent anchor position using the utility function
        let calculated_anchor_pos =
            if let Ok(collider_transform) = collider_query.get(anchor.parent_entity) {
                crate::collider_tools::utils::calculate_anchor_world_position_from_anchor(
                    anchor,
                    &collider_transform,
                )
            } else {
                return;
            };

        let is_selected = selection.contains(entity);

        // Color based on state with higher opacity for better visibility
        let color = if anchor.in_joint {
            Color::srgba(1.0, 0.3, 0.3, 0.9) // Bright red for anchors in joints
        } else if is_selected {
            Color::srgba(1.0, 1.0, 0.0, 1.0) // Bright yellow for selected anchors
        } else {
            Color::srgba(0.3, 1.0, 0.3, 0.8) // Bright green for free anchors
        };

        // Draw anchor point as a dashed circle
        draw_dashed_circle(
            &mut gizmos,
            calculated_anchor_pos,
            anchor.radius,
            color,
            time_offset,
        );

        // Draw a solid inner circle for better visibility
        let inner_radius = anchor.radius * 0.3;
        gizmos.circle_2d(calculated_anchor_pos, inner_radius, color);

        // Draw crosshair at anchor center with thicker lines
        let cross_size = anchor.radius * 0.7;
        let cross_color = Color::srgba(
            color.to_srgba().red,
            color.to_srgba().green,
            color.to_srgba().blue,
            1.0,
        );

        // Horizontal line
        gizmos.line_2d(
            calculated_anchor_pos + Vec2::new(-cross_size, 0.0),
            calculated_anchor_pos + Vec2::new(cross_size, 0.0),
            cross_color,
        );
        // Vertical line
        gizmos.line_2d(
            calculated_anchor_pos + Vec2::new(0.0, -cross_size),
            calculated_anchor_pos + Vec2::new(0.0, cross_size),
            cross_color,
        );

        // Add a small center dot for precise positioning
        gizmos.circle_2d(calculated_anchor_pos, 1.0, cross_color);

        trace!(
            "Drew anchor at calculated position: {:?}, entity: {:?}",
            calculated_anchor_pos, entity
        );
    }
}

/// Draw anchor-related gizmos and UI elements
pub fn draw_anchor_gizmos(
    mut gizmos: Gizmos,
    anchor_query: Query<(&AnchorPoint, &Transform)>,
    collider_query: Query<&GlobalTransform, (With<Collider>, Without<AnchorPoint>)>,
    time: Res<Time>,
) {
    let time_offset = time.elapsed_secs();

    // Draw lines from anchors to their parent collider centers
    for (anchor, anchor_transform) in anchor_query.iter() {
        if let Ok(collider_transform) = collider_query.get(anchor.parent_entity) {
            // Use the same calculation method as sync_anchor_transforms for consistency
            let calculated_anchor_pos =
                crate::collider_tools::utils::calculate_anchor_world_position_from_anchor(
                    anchor,
                    collider_transform,
                );

            // For debugging: also show the actual transform position
            let actual_anchor_pos = anchor_transform.translation.truncate();
            let collider_center = collider_transform.translation().truncate();

            // Draw connection line using calculated position (matches sync_anchor_transforms)
            draw_dashed_line(
                &mut gizmos,
                collider_center,
                calculated_anchor_pos,
                Color::srgba(1.0, 1.0, 1.0, 0.3),
                time_offset,
            );

            // Debug: If positions don't match, draw a red line to show the discrepancy
            if (calculated_anchor_pos - actual_anchor_pos).length() > 0.01 {
                draw_dashed_line(
                    &mut gizmos,
                    calculated_anchor_pos,
                    actual_anchor_pos,
                    Color::srgba(1.0, 0.0, 0.0, 0.8),
                    time_offset,
                );
            }
        }
    }
}
