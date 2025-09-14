use super::{
    ColliderEditState, ColliderType, ControlPoint, ControlPointType, PreviewCollider, ToolMode,
    utils::rotate_point,
};

use crate::EditorSelection;
use avian2d::{parry::shape::TypedShape, prelude::*};
use bevy::prelude::*;
/// Handle input for collider selection mode
///
/// This system provides keyboard shortcuts and mode management for selection mode.
/// The actual selection is handled by the unified handle_collider_selection observer.
// handle_selection_mode_input has been moved to selection.rs module

/// 绘制Capsule的虚线轮廓（沿着连续轮廓绘制）
/// 生成胶囊体的完整轮廓点
pub(super) fn generate_capsule_polyline(
    point_a: Vec2,
    point_b: Vec2,
    radius: f32,
    resolution: u32,
) -> Vec<Vec2> {
    let mut points = Vec::new();

    // 计算线段方向和垂直方向
    let segment_dir = (point_b - point_a).normalize();
    let perpendicular = Vec2::new(-segment_dir.y, segment_dir.x);

    // 计算半圆的分辨率
    let half_circle_points = resolution / 2;

    // 第一个半圆 (围绕point_a)
    for i in 0..=half_circle_points {
        let angle = std::f32::consts::PI * i as f32 / half_circle_points as f32;
        let rotated_perp = Vec2::new(
            perpendicular.x * angle.cos() - perpendicular.y * angle.sin(),
            perpendicular.x * angle.sin() + perpendicular.y * angle.cos(),
        );
        points.push(point_a + rotated_perp * radius);
    }

    // 第二个半圆 (围绕point_b)
    for i in 0..=half_circle_points {
        let angle = std::f32::consts::PI * (i as f32 + half_circle_points as f32)
            / half_circle_points as f32;
        let rotated_perp = Vec2::new(
            perpendicular.x * angle.cos() - perpendicular.y * angle.sin(),
            perpendicular.x * angle.sin() + perpendicular.y * angle.cos(),
        );
        points.push(point_b + rotated_perp * radius);
    }

    points
}

/// 使用完整轮廓点绘制胶囊体虚线
pub(super) fn draw_dashed_capsule<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    point_a: Vec2,
    point_b: Vec2,
    radius: f32,
    color: Color,
    time_offset: f32,
) {
    // 生成胶囊体的完整轮廓点
    let polyline_points = generate_capsule_polyline(point_a, point_b, radius, 32);

    // 计算每段的长度
    let mut segment_lengths = Vec::new();
    let mut total_perimeter = 0.0;
    for i in 0..polyline_points.len() {
        let next_i = (i + 1) % polyline_points.len();
        let length = (polyline_points[next_i] - polyline_points[i]).length();
        segment_lengths.push(length);
        total_perimeter += length;
    }

    // 虚线参数
    let dash_length = 8.0;
    let gap_length = 4.0;
    let pattern_length = dash_length + gap_length;

    // 滚动速度（单位/秒）
    let scroll_speed = 30.0;
    let scroll_offset = (time_offset * scroll_speed) % pattern_length;

    // 计算需要绘制的虚线段数量
    let num_dashes = ((total_perimeter + pattern_length) / pattern_length) as usize;

    for i in 0..num_dashes {
        let dash_start_distance = (i as f32 * pattern_length) - scroll_offset;
        let dash_end_distance = dash_start_distance + dash_length;

        // 只绘制在轮廓范围内的部分
        if dash_end_distance > 0.0 && dash_start_distance < total_perimeter {
            let actual_start = dash_start_distance.max(0.0);
            let actual_end = dash_end_distance.min(total_perimeter);

            if actual_start < actual_end {
                draw_polyline_segment(
                    gizmos,
                    &polyline_points,
                    &segment_lengths,
                    actual_start,
                    actual_end,
                    color,
                );
            }
        }
    }
}

/// 在多边形轮廓上绘制一段线
pub(super) fn draw_polyline_segment<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    points: &[Vec2],
    segment_lengths: &[f32],
    start_distance: f32,
    end_distance: f32,
    color: Color,
) {
    let mut current_distance = 0.0;
    let mut draw_start: Option<Vec2> = None;

    for i in 0..points.len() {
        let next_i = (i + 1) % points.len();
        let segment_length = segment_lengths[i];
        let segment_end_distance = current_distance + segment_length;

        // 检查起始点
        if draw_start.is_none()
            && start_distance >= current_distance
            && start_distance <= segment_end_distance
        {
            let t = (start_distance - current_distance) / segment_length;
            draw_start = Some(points[i].lerp(points[next_i], t));
        }

        // 检查结束点
        if let Some(start_pos) = draw_start {
            if end_distance >= current_distance && end_distance <= segment_end_distance {
                let t = (end_distance - current_distance) / segment_length;
                let end_pos = points[i].lerp(points[next_i], t);
                gizmos.line_2d(start_pos, end_pos, color);
                return;
            } else if segment_end_distance <= end_distance {
                // 绘制到当前段的结束
                gizmos.line_2d(start_pos, points[next_i], color);
                draw_start = Some(points[next_i]);
            }
        }

        current_distance = segment_end_distance;
    }
}

/// 绘制选中collider的虚线轮廓系统
pub fn draw_selected_collider_outlines<Config: GizmoConfigGroup>(
    mut gizmos: Gizmos<Config>,
    selection: Res<EditorSelection>,
    collider_query: Query<(&Transform, &Collider), With<ColliderType>>,
    time: Res<Time>,
    theme_colors: Res<crate::ui::theme_colors::EditorThemeColors>,
) {
    let selection_color = theme_colors.selection_outline;
    let time_offset = time.elapsed_secs();

    // 遍历所有选中的实体，而不只是主选择
    for selected_entity in selection.iter() {
        if let Ok((transform, collider)) = collider_query.get(selected_entity) {
            draw_selection_outline(
                &mut gizmos,
                transform,
                collider,
                selection_color,
                time_offset,
            );
        }
    }
}

/// Calculate vertices for different collider types
///
/// Generates appropriate vertex arrays for visualization based on collider type
/// and mouse drag positions.
///
/// # Parameters
///
/// - `collider_type`: Type of collider to calculate vertices for
/// - `start`: Starting mouse position (world coordinates)
/// - `end`: Current mouse position (world coordinates)
///
/// # Returns
///
/// Vector of Vec2 points representing the collider shape
pub fn calculate_collider_vertices(
    collider_type: ColliderType,
    start: Vec2,
    end: Vec2,
) -> Vec<Vec2> {
    match collider_type {
        ColliderType::Rectangle => {
            let center = (start + end) / 2.0;
            let half_size = (end - start).abs() / 2.0;
            vec![
                center + Vec2::new(-half_size.x, -half_size.y),
                center + Vec2::new(half_size.x, -half_size.y),
                center + Vec2::new(half_size.x, half_size.y),
                center + Vec2::new(-half_size.x, half_size.y),
            ]
        }
        ColliderType::Circle => {
            let center = (start + end) / 2.0;
            let radius = start.distance(end);
            (0..32)
                .map(|i| {
                    let angle = (i as f32 / 32.0) * std::f32::consts::TAU;
                    center + Vec2::new(angle.cos(), angle.sin()) * radius
                })
                .collect()
        }
        ColliderType::Capsule => {
            // Constants for capsule generation
            const MIN_LENGTH: f32 = 10.0;
            const RADIUS_RATIO: f32 = 0.2;
            const MIN_RADIUS: f32 = 2.0;
            const ARC_SEGMENTS: usize = 16;

            let center = (start + end) / 2.0;
            let total_length = start.distance(end).max(MIN_LENGTH);
            let radius = (total_length * RADIUS_RATIO).max(MIN_RADIUS);
            let half_length = (total_length / 2.0).max(radius);

            // Calculate capsule axis direction and perpendicular vector
            let axis_direction = (end - start).normalize_or_zero();
            let perpendicular = Vec2::new(-axis_direction.y, axis_direction.x);

            let mut vertices = Vec::new();

            // Calculate the rectangle corners (based on Bevy's gizmos implementation)
            let top_left = center + axis_direction * half_length - perpendicular * radius;
            let top_right = center + axis_direction * half_length + perpendicular * radius;
            let bottom_left = center - axis_direction * half_length - perpendicular * radius;
            let bottom_right = center - axis_direction * half_length + perpendicular * radius;

            // Calculate the centers of the semicircles
            let top_center = center + axis_direction * half_length;
            let bottom_center = center - axis_direction * half_length;

            // Generate vertices for the capsule outline (counter-clockwise)

            // Left side line
            vertices.push(bottom_left);
            vertices.push(top_left);

            // Top semicircle
            for i in 1..ARC_SEGMENTS {
                let angle = -std::f32::consts::FRAC_PI_2
                    + (i as f32 / ARC_SEGMENTS as f32) * std::f32::consts::PI;
                let local_offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                // Transform local coordinates to world coordinates
                let offset = axis_direction * local_offset.x + perpendicular * local_offset.y;
                vertices.push(top_center + offset);
            }

            // Right side line
            vertices.push(top_right);
            vertices.push(bottom_right);

            // Bottom semicircle
            for i in 1..ARC_SEGMENTS {
                let angle = std::f32::consts::FRAC_PI_2
                    + (i as f32 / ARC_SEGMENTS as f32) * std::f32::consts::PI;
                let local_offset = Vec2::new(angle.cos(), angle.sin()) * radius;
                // Transform local coordinates to world coordinates
                let offset = axis_direction * local_offset.x + perpendicular * local_offset.y;
                vertices.push(bottom_center + offset);
            }

            vertices
        }
        ColliderType::Triangle => {
            let diff = end - start;
            vec![start, end, start + Vec2::new(diff.x, 0.0)]
        }
        ColliderType::Polygon => {
            let diff = end - start;
            let length = diff.length();

            if length < 10.0 {
                // For very small polygons, create a simple hexagon
                let center = (start + end) / 2.0;
                let radius = 10.0;
                (0..6)
                    .map(|i| {
                        let angle = (i as f32 / 6.0) * std::f32::consts::TAU;
                        center + Vec2::new(angle.cos(), angle.sin()) * radius
                    })
                    .collect()
            } else {
                // Create a polygon that adapts to the drag direction and length
                let direction = diff.normalize();
                let center = (start + end) / 2.0;

                // Base radius on the drag length
                let radius = length * 0.4;

                // Rotate the polygon to align with drag direction
                let base_angle = direction.y.atan2(direction.x);

                // Create a hexagon oriented towards the drag direction
                (0..6)
                    .map(|i| {
                        let angle = base_angle + (i as f32 / 6.0) * std::f32::consts::TAU;
                        center + Vec2::new(angle.cos(), angle.sin()) * radius
                    })
                    .collect()
            }
        }
    }
}

/// Draw collider shape for visualization
///
/// Renders the collider shape using Bevy's Gizmos based on the preview data.
///
/// # Parameters
///
/// - `gizmos`: Gizmos resource for drawing
/// - `preview`: Preview collider data containing vertices
/// - `color`: Color to use for the visualization
pub fn draw_collider_shape<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    preview: &PreviewCollider,
    color: Color,
) {
    let vertices = &preview.vertices;

    if vertices.len() >= 2 {
        // For all shapes including capsules, use linestrip with closure
        gizmos.linestrip_2d(vertices.clone(), color);
        if vertices.len() > 2 {
            gizmos.line_2d(vertices[vertices.len() - 1], vertices[0], color);
        }
    }
}

/// Update visualization for edit mode
pub fn update_edit_visualization<Config: GizmoConfigGroup>(
    mut gizmos: Gizmos<Config>,
    edit_state: Res<ColliderEditState>,
    creation_mode: Res<State<ToolMode>>,
    theme_colors: Res<crate::ui::theme_colors::EditorThemeColors>,
) {
    // Only show edit visualization in Edit mode
    if *creation_mode.get() != ToolMode::Edit {
        return;
    }

    // Draw control points as gizmo circles
    for control_point in &edit_state.control_points {
        let (color, radius) = match control_point.point_type {
            ControlPointType::Vertex => (theme_colors.control_point_vertex, 8.0),
            ControlPointType::RadiusControl => (theme_colors.control_point_radius, 8.0),
            ControlPointType::LengthControl => (theme_colors.control_point_length, 8.0),
            _ => (Color::srgb(0.5, 0.5, 0.5), 6.0), // Gray, fallback for other types
        };

        // Draw filled circle for better visibility
        gizmos.circle_2d(
            bevy::math::Isometry2d::from_translation(control_point.position),
            radius,
            color,
        );

        // Add a colored outline for better contrast
        gizmos.circle_2d(
            bevy::math::Isometry2d::from_translation(control_point.position),
            radius + 1.0,
            theme_colors.control_point_outline,
        );
    }
}

// ===== HELPER FUNCTIONS =====

/// Generate control points for a collider
pub fn generate_control_points(
    edit_state: &mut ColliderEditState,
    transform: &Transform,
    collider: &Collider,
    collider_type: &ColliderType,
) {
    edit_state.control_points.clear();

    let center = transform.translation.truncate();
    let rotation = transform.rotation;

    // Helper function to transform a local point to world space
    let transform_point = |local_point: Vec2| -> Vec2 {
        let rotated = rotation * Vec3::new(local_point.x, local_point.y, 0.0);
        center + Vec2::new(rotated.x, rotated.y)
    };

    // Get collider AABB to determine geometry
    let aabb = collider.aabb(avian2d::math::Vector::ZERO, 0.0);
    let half_extents = aabb.max - aabb.min;
    let size = Vec2::new(half_extents.x, half_extents.y);

    // Generate type-specific control points based on collider type and geometry
    match collider_type {
        ColliderType::Rectangle => {
            // For rectangles, add a single dynamic control point at the closest corner
            match collider.shape_scaled().as_typed_shape() {
                TypedShape::Cuboid(cuboid) => {
                    let half_extents = cuboid.half_extents;
                    let local_corners = [
                        Vec2::new(-half_extents.x, -half_extents.y), // Bottom-left
                        Vec2::new(half_extents.x, -half_extents.y),  // Bottom-right
                        Vec2::new(half_extents.x, half_extents.y),   // Top-right
                        Vec2::new(-half_extents.x, half_extents.y),  // Top-left
                    ];

                    // Transform all corners to world space
                    let world_corners: Vec<Vec2> = local_corners
                        .iter()
                        .map(|local_point| transform_point(*local_point))
                        .collect();

                    // Add a single dynamic control point
                    edit_state.control_points.push(ControlPoint {
                        position: world_corners[0], // Default to first corner, will be updated dynamically
                        point_type: ControlPointType::Vertex,
                        vertex_index: Some(0), // Will track which corner is currently active
                    });
                }
                _ => {
                    // Fallback to AABB-based control points
                    let half_size = size * 0.5;
                    let local_corners = [
                        Vec2::new(-half_size.x, -half_size.y), // Bottom-left
                        Vec2::new(half_size.x, -half_size.y),  // Bottom-right
                        Vec2::new(half_size.x, half_size.y),   // Top-right
                        Vec2::new(-half_size.x, half_size.y),  // Top-left
                    ];

                    // Transform all corners to world space
                    let world_corners: Vec<Vec2> = local_corners
                        .iter()
                        .map(|local_point| transform_point(*local_point))
                        .collect();

                    // Add a single dynamic control point
                    edit_state.control_points.push(ControlPoint {
                        position: world_corners[0], // Default to first corner, will be updated dynamically
                        point_type: ControlPointType::Vertex,
                        vertex_index: Some(0), // Will track which corner is currently active
                    });
                }
            }
        }
        ColliderType::Circle => {
            // Add a single dynamic radius control point
            let radius = size.x * 0.5;
            let local_radius_point = Vec2::new(radius, 0.0); // Default to right side, will be updated dynamically
            edit_state.control_points.push(ControlPoint {
                position: transform_point(local_radius_point),
                point_type: ControlPointType::RadiusControl,
                vertex_index: Some(0), // Will track angle position
            });
        }
        ColliderType::Capsule => {
            // Extract actual capsule dimensions from the collider
            match collider.shape_scaled().as_typed_shape() {
                TypedShape::Capsule(capsule) => {
                    let radius = capsule.radius;

                    // Get the actual capsule endpoints in local space
                    let local_start = Vec2::new(capsule.segment.a.x, capsule.segment.a.y);
                    let local_end = Vec2::new(capsule.segment.b.x, capsule.segment.b.y);

                    // Generate a single dynamic control point (prioritize radius control)
                    let capsule_direction = (local_end - local_start).normalize_or_zero();
                    let perpendicular = Vec2::new(-capsule_direction.y, capsule_direction.x);
                    let local_radius_point = local_start + perpendicular * radius;

                    edit_state.control_points.push(ControlPoint {
                        position: transform_point(local_radius_point),
                        point_type: ControlPointType::RadiusControl,
                        vertex_index: Some(0), // Will track which control type is active
                    });
                }
                _ => {
                    // Fallback for other capsule representations
                    // Assume vertical orientation as default
                    let _half_height = size.y * 0.5;
                    let radius = size.x * 0.5;

                    // Single dynamic radius control point
                    let local_radius_point = Vec2::new(radius, 0.0);
                    edit_state.control_points.push(ControlPoint {
                        position: transform_point(local_radius_point),
                        point_type: ControlPointType::RadiusControl,
                        vertex_index: Some(0), // Will track which control type is active
                    });
                }
            }
        }
        ColliderType::Triangle => {
            // For triangles, extract actual vertices from the collider
            match collider.shape_scaled().as_typed_shape() {
                TypedShape::Triangle(triangle) => {
                    let local_vertices = [
                        Vec2::new(triangle.a.x, triangle.a.y),
                        Vec2::new(triangle.b.x, triangle.b.y),
                        Vec2::new(triangle.c.x, triangle.c.y),
                    ];

                    for (i, local_vertex) in local_vertices.iter().enumerate() {
                        edit_state.control_points.push(ControlPoint {
                            position: transform_point(*local_vertex),
                            point_type: ControlPointType::Vertex,
                            vertex_index: Some(i),
                        });
                    }
                }
                _ => {
                    // Fallback for other triangle representations
                    let half_size = size * 0.5;
                    let local_vertices = [
                        Vec2::new(0.0, half_size.y),           // Top
                        Vec2::new(-half_size.x, -half_size.y), // Bottom-left
                        Vec2::new(half_size.x, -half_size.y),  // Bottom-right
                    ];
                    for (i, local_vertex) in local_vertices.iter().enumerate() {
                        edit_state.control_points.push(ControlPoint {
                            position: transform_point(*local_vertex),
                            point_type: ControlPointType::Vertex,
                            vertex_index: Some(i),
                        });
                    }
                }
            }
        }
        ColliderType::Polygon => {
            // For polygons, extract actual vertices from the collider
            let vertices: Vec<Vec2> = match collider.shape_scaled().as_typed_shape() {
                TypedShape::ConvexPolygon(poly) => poly
                    .points()
                    .iter()
                    .map(|p| transform_point(Vec2::new(p.x, p.y)))
                    .collect(),
                _ => {
                    // Fallback to AABB-based vertices
                    let half_size = size * 0.5;
                    let local_vertices = vec![
                        Vec2::new(-half_size.x, -half_size.y),
                        Vec2::new(half_size.x, -half_size.y),
                        Vec2::new(half_size.x, half_size.y),
                        Vec2::new(-half_size.x, half_size.y),
                    ];
                    local_vertices.iter().map(|v| transform_point(*v)).collect()
                }
            };

            for (i, vertex) in vertices.iter().enumerate() {
                edit_state.control_points.push(ControlPoint {
                    position: *vertex,
                    point_type: ControlPointType::Vertex,
                    vertex_index: Some(i),
                });
            }
        }
    }
}

/// 绘制虚线（带滚动动画效果）
pub(super) fn draw_dashed_line<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    start: Vec2,
    end: Vec2,
    color: Color,
    time_offset: f32,
) {
    let direction = end - start;
    let length = direction.length();
    let dash_length = 5.0;
    let gap_length = 3.0;
    let total_pattern = dash_length + gap_length;

    // 滚动速度（像素/秒）
    let scroll_speed = 20.0;
    let scroll_offset = (time_offset * scroll_speed) % total_pattern;

    let num_patterns = ((length + total_pattern) / total_pattern) as usize;
    let unit_direction = direction.normalize();

    for i in 0..num_patterns {
        let pattern_start = (i as f32 * total_pattern) - scroll_offset;
        let dash_start_pos = pattern_start;
        let dash_end_pos = pattern_start + dash_length;

        // 只绘制在线段范围内的部分
        if dash_end_pos > 0.0 && dash_start_pos < length {
            let actual_start = dash_start_pos.max(0.0);
            let actual_end = dash_end_pos.min(length);

            if actual_start < actual_end {
                let dash_start = start + unit_direction * actual_start;
                let dash_end = start + unit_direction * actual_end;
                gizmos.line_2d(dash_start, dash_end, color);
            }
        }
    }
}

/// 绘制虚线圆圈（带滚动动画效果）
pub(super) fn draw_dashed_circle<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    center: Vec2,
    radius: f32,
    color: Color,
    time_offset: f32,
) {
    draw_dashed_arc(
        gizmos,
        center,
        radius,
        0.0,
        std::f32::consts::TAU,
        color,
        time_offset,
    );
}

pub(super) fn draw_dashed_arc<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    center: Vec2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: Color,
    time_offset: f32,
) {
    let segments = 64; // 增加段数以获得更平滑的动画
    let dash_segments = 3;
    let gap_segments = 2;
    let total_pattern = dash_segments + gap_segments;

    // 滚动速度（弧度/秒）
    let scroll_speed = 2.0;
    let angle_offset = -time_offset * scroll_speed;

    // 计算角度范围
    let mut angle_range = end_angle - start_angle;
    if angle_range < 0.0 {
        angle_range += std::f32::consts::TAU;
    }

    // 根据角度范围调整段数
    let arc_segments = ((segments as f32 * angle_range / std::f32::consts::TAU) as usize).max(1);
    let segment_angle = angle_range / arc_segments as f32;

    for i in 0..arc_segments {
        let pattern_position = i % total_pattern;

        // 只在虚线段绘制，跳过间隙段
        if pattern_position < dash_segments {
            let base_angle = start_angle + i as f32 * segment_angle + angle_offset;
            let current_start_angle = base_angle;
            let current_end_angle = base_angle + segment_angle;

            let start_point =
                center + Vec2::new(current_start_angle.cos(), current_start_angle.sin()) * radius;
            let end_point =
                center + Vec2::new(current_end_angle.cos(), current_end_angle.sin()) * radius;
            gizmos.line_2d(start_point, end_point, color);
        }
    }
}

/// 绘制预览锚点
pub fn draw_preview_anchor<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    position: Vec2,
    time_offset: f32,
    theme_colors: &crate::ui::theme_colors::EditorThemeColors,
) {
    let color = theme_colors.preview_anchor;

    // Draw preview circle
    draw_dashed_circle(gizmos, position, 8.0, color, time_offset);

    // Draw crosshair
    let cross_size = 5.0;
    gizmos.line_2d(
        position + Vec2::new(-cross_size, 0.0),
        position + Vec2::new(cross_size, 0.0),
        color,
    );
    gizmos.line_2d(
        position + Vec2::new(0.0, -cross_size),
        position + Vec2::new(0.0, cross_size),
        color,
    );
}

/// 绘制预览关节
pub fn draw_preview_joint<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    pos_a: Vec2,
    pos_b: Vec2,
    joint_type: crate::debug_render::joint::JointType,
    time_offset: f32,
    theme_colors: &crate::ui::theme_colors::EditorThemeColors,
) {
    let color = theme_colors.preview_joint;

    match joint_type {
        crate::debug_render::joint::JointType::Distance => {
            draw_dashed_line(gizmos, pos_a, pos_b, color, time_offset);
        }
        crate::debug_render::joint::JointType::Revolute => {
            let mid_point = (pos_a + pos_b) / 2.0;
            let radius = pos_a.distance(pos_b) / 2.0;

            draw_dashed_circle(gizmos, mid_point, radius, color, time_offset);
            draw_dashed_line(gizmos, pos_a, mid_point, color, time_offset);
            draw_dashed_line(gizmos, pos_b, mid_point, color, time_offset);
        }
        crate::debug_render::joint::JointType::Prismatic => {
            let direction = (pos_b - pos_a).normalize();
            let perpendicular = Vec2::new(-direction.y, direction.x);
            let offset = perpendicular * 5.0;

            draw_dashed_line(gizmos, pos_a + offset, pos_b + offset, color, time_offset);
            draw_dashed_line(gizmos, pos_a - offset, pos_b - offset, color, time_offset);
        }
        crate::debug_render::joint::JointType::Fixed => {
            let direction = (pos_b - pos_a).normalize();
            let perpendicular = Vec2::new(-direction.y, direction.x);
            let offset = perpendicular * 3.0;

            draw_dashed_line(gizmos, pos_a + offset, pos_b + offset, color, time_offset);
            draw_dashed_line(gizmos, pos_a - offset, pos_b - offset, color, time_offset);
        }
    }
}

/// 绘制选中collider的虚线轮廓
pub(super) fn draw_selection_outline<Config: GizmoConfigGroup>(
    gizmos: &mut Gizmos<Config>,
    transform: &Transform,
    collider: &Collider,
    color: Color,
    time_offset: f32,
) {
    let center = transform.translation.truncate();
    let rotation = transform.rotation.to_euler(EulerRot::YXZ).2; // 2D rotation

    match collider.shape_scaled().as_typed_shape() {
        TypedShape::Cuboid(cuboid) => {
            // Rectangle collider in 2D
            let half_size = cuboid.half_extents;
            let corners = [
                Vec2::new(-half_size.x, -half_size.y),
                Vec2::new(half_size.x, -half_size.y),
                Vec2::new(half_size.x, half_size.y),
                Vec2::new(-half_size.x, half_size.y),
            ];

            let rotated_corners: Vec<Vec2> = corners
                .iter()
                .map(|&corner| rotate_point(corner, rotation) + center)
                .collect();

            // 绘制虚线轮廓
            for i in 0..4 {
                let next = (i + 1) % 4;
                draw_dashed_line(
                    gizmos,
                    rotated_corners[i],
                    rotated_corners[next],
                    color,
                    time_offset,
                );
            }
        }
        TypedShape::Ball(sphere) => {
            // Circle collider in 2D
            let radius = sphere.radius;
            // 绘制虚线圆圈
            draw_dashed_circle(gizmos, center, radius, color, time_offset);
        }
        TypedShape::Capsule(capsule) => {
            // Capsule collider - 使用专门的Capsule虚线绘制函数
            let radius = capsule.radius;
            let segment_a = Vec2::new(capsule.segment.a.x, capsule.segment.a.y);
            let segment_b = Vec2::new(capsule.segment.b.x, capsule.segment.b.y);

            // 应用变换到端点
            let point_a = rotate_point(segment_a, rotation) + center;
            let point_b = rotate_point(segment_b, rotation) + center;

            draw_dashed_capsule(gizmos, point_a, point_b, radius, color, time_offset);
        }
        TypedShape::Triangle(triangle) => {
            // Triangle collider
            let points = [
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ];
            let rotated_points: Vec<Vec2> = points
                .iter()
                .map(|&point| rotate_point(point, rotation) + center)
                .collect();

            // 绘制三角形虚线轮廓
            for i in 0..3 {
                let next = (i + 1) % 3;
                draw_dashed_line(
                    gizmos,
                    rotated_points[i],
                    rotated_points[next],
                    color,
                    time_offset,
                );
            }
        }
        TypedShape::ConvexPolygon(polygon) => {
            // Convex polygon collider
            let vertices: Vec<Vec2> = polygon
                .points()
                .iter()
                .map(|p| Vec2::new(p.x, p.y))
                .collect();
            let rotated_vertices: Vec<Vec2> = vertices
                .iter()
                .map(|&vertex| rotate_point(vertex, rotation) + center)
                .collect();

            // 绘制多边形虚线轮廓
            for i in 0..rotated_vertices.len() {
                let next = (i + 1) % rotated_vertices.len();
                draw_dashed_line(
                    gizmos,
                    rotated_vertices[i],
                    rotated_vertices[next],
                    color,
                    time_offset,
                );
            }
        }
        _ => {
            // 其他类型的简单轮廓
            let size = Vec2::splat(20.0);
            let corners = [
                Vec2::new(-size.x / 2.0, -size.y / 2.0),
                Vec2::new(size.x / 2.0, -size.y / 2.0),
                Vec2::new(size.x / 2.0, size.y / 2.0),
                Vec2::new(-size.x / 2.0, size.y / 2.0),
            ];

            let rotated_corners: Vec<Vec2> = corners
                .iter()
                .map(|&corner| rotate_point(corner, rotation) + center)
                .collect();

            for i in 0..4 {
                let next = (i + 1) % 4;
                draw_dashed_line(
                    gizmos,
                    rotated_corners[i],
                    rotated_corners[next],
                    color,
                    time_offset,
                );
            }
        }
    }
}
