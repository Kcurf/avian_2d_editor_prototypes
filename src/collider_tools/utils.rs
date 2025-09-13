//! # Utility Functions for Collider Tools
//!
//! This module provides utility functions for collider creation, manipulation, and interaction.
//!
//! ## Important Design Principles
//!
//! **All functions in this module follow the principle of explicit parameter passing rather than
//! directly using ECS queries.** This approach ensures:
//!
//! - Better separation of concerns
//! - Improved testability
//! - Clearer dependencies
//! - Better performance by avoiding unnecessary query operations
//! - Greater flexibility in usage
//!
//! ## Usage Guidelines
//!
//! When calling these functions from Bevy systems:
//!
//! ```rust
//! // ❌ INCORRECT: Don't pass query objects directly
//! let result = find_collider_at_position(pos, &collider_query);
//!
//! // ✅ CORRECT: Pass the specific data needed
//! let result = find_collider_at_position(pos, collider_query.iter().map(|(e, t)| (e, t)));
//!
//! // ❌ INCORRECT: Don't let functions access queries internally
//! fn bad_example() {
//!     let window = windows.single(); // Direct query access
//! }
//!
//! // ✅ CORRECT: Pass required parameters explicitly
//! fn good_example(window: &Window, camera: &Camera, transform: &GlobalTransform) {
//!     let pos = get_cursor_world_position(window, camera, transform);
//! }
//! ```

use crate::debug_render::AnchorPoint;

use super::{ColliderType, PreviewCollider, calculate_collider_vertices};
use avian2d::{math::Vector, prelude::*};
use bevy::prelude::*;

/// Add appropriate mass properties based on collider type
///
/// Calculates and adds mass properties to dynamic bodies based on the
/// collider type and dimensions.
///
/// # Parameters
///
/// - `entity_commands`: Entity commands for adding components
/// - `preview`: Preview collider data for dimension calculation
/// - `density`: Density value for mass calculation
pub(super) fn add_mass_properties(
    entity_commands: &mut EntityCommands,
    preview: &PreviewCollider,
    density: f32,
) {
    match preview.collider_type {
        ColliderType::Rectangle => {
            let size = (preview.current_pos - preview.start_pos).abs();
            let shape = Rectangle::new(size.x, size.y);
            entity_commands.insert(MassPropertiesBundle::from_shape(&shape, density));
        }
        ColliderType::Circle => {
            let radius = preview.start_pos.distance(preview.current_pos);
            let shape = Circle::new(radius);
            entity_commands.insert(MassPropertiesBundle::from_shape(&shape, density));
        }
        ColliderType::Capsule => {
            let height = preview.start_pos.distance(preview.current_pos).max(10.0);
            let radius = (height * 0.2).max(2.0);
            let shape = Capsule2d::new(radius, height / 2.0);
            entity_commands.insert(MassPropertiesBundle::from_shape(&shape, density));
        }
        ColliderType::Triangle => {
            let diff = preview.current_pos - preview.start_pos;
            let vertices = vec![
                preview.start_pos,
                preview.current_pos,
                preview.start_pos + Vec2::new(diff.x, 0.0),
            ];
            let min_x = vertices.iter().map(|v| v.x).fold(f32::MAX, f32::min);
            let max_x = vertices.iter().map(|v| v.x).fold(f32::MIN, f32::max);
            let min_y = vertices.iter().map(|v| v.y).fold(f32::MAX, f32::min);
            let max_y = vertices.iter().map(|v| v.y).fold(f32::MIN, f32::max);
            let size = Vec2::new(max_x - min_x, max_y - min_y).max(Vec2::splat(10.0));
            let shape = Rectangle::new(size.x, size.y);
            entity_commands.insert(MassPropertiesBundle::from_shape(&shape, density));
        }
        ColliderType::Polygon => {
            let vertices = calculate_collider_vertices(
                ColliderType::Polygon,
                preview.start_pos,
                preview.current_pos,
            );
            if vertices.len() >= 3 {
                let min_x = vertices.iter().map(|v| v.x).fold(f32::MAX, f32::min);
                let max_x = vertices.iter().map(|v| v.x).fold(f32::MIN, f32::max);
                let min_y = vertices.iter().map(|v| v.y).fold(f32::MAX, f32::min);
                let max_y = vertices.iter().map(|v| v.y).fold(f32::MIN, f32::max);
                let size = Vec2::new(max_x - min_x, max_y - min_y).max(Vec2::splat(10.0));
                let shape = Rectangle::new(size.x, size.y);
                entity_commands.insert(MassPropertiesBundle::from_shape(&shape, density));
            } else {
                let radius = preview.start_pos.distance(preview.current_pos).max(10.0);
                let shape = Circle::new(radius);
                entity_commands.insert(MassPropertiesBundle::from_shape(&shape, density));
            }
        }
    }
}

pub(super) fn get_cursor_world_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    if let Some(cursor_pos) = window.cursor_position() {
        camera
            .viewport_to_world_2d(camera_transform, cursor_pos)
            .ok()
    } else {
        None
    }
}

/// 旋转 2D 点
pub(super) fn rotate_point(point: Vec2, angle: f32) -> Vec2 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    Vec2::new(
        point.x * cos_a - point.y * sin_a,
        point.x * sin_a + point.y * cos_a,
    )
}

pub(crate) fn create_anchor_at_position(
    commands: &mut Commands,
    position: Vec2,
    parent_entity: Entity,
    parent_transform: Option<&GlobalTransform>,
) -> Entity {
    // Calculate local anchor position using the corrected method
    let local_anchor_position = if let Some(parent_transform) = parent_transform {
        let parent_pos = parent_transform.translation().truncate();
        let offset = position - parent_pos;
        let inverse_rotation = parent_transform.rotation().inverse();
        (inverse_rotation * offset.extend(0.0)).xy()
    } else {
        Vec2::ZERO
    };

    // For child entities, the Transform should be in local coordinates relative to parent
    // Convert world position to local position for the transform
    let local_transform_position = if let Some(parent_transform) = parent_transform {
        // Use the same calculation as calculate_local_anchor_position but without collider dependency
        let collider_origin = parent_transform.translation().truncate();
        let offset = position - collider_origin;
        let inverse_rotation = parent_transform.rotation().inverse();
        (inverse_rotation * offset.extend(0.0)).xy()
    } else {
        position
    };

    let anchor_entity = commands
        .spawn((
            AnchorPoint {
                local_anchor_position,
                parent_entity,
                in_joint: false,
                radius: 8.0,
            },
            Transform::from_xyz(local_transform_position.x, local_transform_position.y, 0.0),
            crate::selection::Selectable::default(),
            bevy::picking::Pickable {
                should_block_lower: true,
                is_hoverable: true,
            }, // Add picking component for mouse interaction
            bevy::prelude::ChildOf(parent_entity), // Use built-in parent-child relationship
        ))
        .id();

    anchor_entity
}

pub(crate) fn get_anchor_local_position(anchor_point: &AnchorPoint) -> Vector {
    Vector::new(
        anchor_point.local_anchor_position.x,
        anchor_point.local_anchor_position.y,
    )
}

pub(crate) fn is_point_inside_collider(
    point: Vec2,
    collider: &Collider,
    transform: &GlobalTransform,
) -> bool {
    // Transform point to local space
    let transform_inv = transform.compute_matrix().inverse();
    let local_point = transform_inv.transform_point(point.extend(0.0)).truncate();

    // Check if point is inside the collider shape (with default rotation)
    collider.contains_point(local_point, 0.0, Vector::ZERO)
}

/// Calculate the world position of an anchor point using its anchor component and parent collider transform
/// This is the same calculation used in sync_anchor_transforms and anchor dragging logic
pub(crate) fn calculate_anchor_world_position_from_anchor(
    anchor: &crate::debug_render::AnchorPoint,
    collider_transform: &GlobalTransform,
) -> Vec2 {
    let collider_center = collider_transform.translation().truncate();
    let rotated_anchor = collider_transform.rotation() * anchor.local_anchor_position.extend(0.0);
    collider_center + rotated_anchor.xy()
}

/// ```
pub(crate) fn find_closest_point_on_collider(
    point: Vec2,
    collider: &Collider,
    transform: &GlobalTransform,
) -> Vec2 {
    // Transform point to local space
    let transform_inv = transform.compute_matrix().inverse();
    let local_point = transform_inv.transform_point(point.extend(0.0)).truncate();

    // Find closest point on collider in local space
    let (local_closest, _) = collider.project_point(Vector::ZERO, 0.0, local_point, false);

    // Transform back to world space
    transform
        .transform_point(local_closest.extend(0.0))
        .truncate()
}

pub(crate) fn find_collider_at_position_with_spatial_query(
    position: Vec2,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> Option<Entity> {
    let mut found_entity = None;

    spatial_query.point_intersections_callback(position, filter, |entity| {
        found_entity = Some(entity);
        false // Stop at first hit
    });

    found_entity
}

pub(crate) fn get_mouse_world_position(
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Option<Vec2> {
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.truncate())
}

/// Helper function for ray-line segment intersection
fn ray_line_segment_intersection(
    ray_origin: Vec2,
    ray_dir: Vec2,
    line_start: Vec2,
    line_end: Vec2,
) -> Option<Vec2> {
    let line_vec = line_end - line_start;
    let line_len_sq = line_vec.length_squared();

    if line_len_sq < 1e-10 {
        return None; // Line segment is too short
    }

    // Calculate parameters for ray-line intersection
    let ray_cross_line = ray_dir.x * line_vec.y - ray_dir.y * line_vec.x;

    if ray_cross_line.abs() < 1e-10 {
        return None; // Ray and line are parallel
    }

    let t = ((line_start.x - ray_origin.x) * line_vec.y
        - (line_start.y - ray_origin.y) * line_vec.x)
        / ray_cross_line;
    let u = ((line_start.x - ray_origin.x) * ray_dir.y - (line_start.y - ray_origin.y) * ray_dir.x)
        / ray_cross_line;

    // Check if intersection is within line segment bounds and in ray direction
    if u >= 0.0 && u <= 1.0 && t > 1e-6 {
        Some(ray_origin + ray_dir * t)
    } else {
        None
    }
}

/// Find ray-polygon intersection for directional Ctrl mode
fn find_ray_polygon_intersection(origin: Vec2, mouse_pos: Vec2, vertices: &[Vec2]) -> Option<Vec2> {
    let ray_dir = (mouse_pos - origin).normalize_or_zero();
    let mut closest_intersection = None;
    let mut min_distance = f32::INFINITY;

    for i in 0..vertices.len() {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % vertices.len()];

        if let Some(intersection) = ray_line_segment_intersection(origin, ray_dir, v1, v2) {
            let distance = origin.distance(intersection);
            if distance < min_distance && distance > 0.001 {
                min_distance = distance;
                closest_intersection = Some(intersection);
            }
        }
    }
    closest_intersection
}

/// Find the closest point on polygon edges to a given point
fn find_closest_point_on_polygon_edges(vertices: &[Vec2], point: Vec2) -> Option<Vec2> {
    let mut closest_point = None;
    let mut min_distance = f32::INFINITY;

    for i in 0..vertices.len() {
        let v1 = vertices[i];
        let v2 = vertices[(i + 1) % vertices.len()];

        // Calculate closest point on line segment
        let edge_vec = v2 - v1;
        let edge_len_sq = edge_vec.length_squared();

        if edge_len_sq < 1e-10 {
            continue;
        }

        let t = ((point - v1).dot(edge_vec) / edge_len_sq).clamp(0.0, 1.0);
        let closest_on_edge = v1 + edge_vec * t;
        let distance = point.distance(closest_on_edge);

        if distance < min_distance {
            min_distance = distance;
            closest_point = Some(closest_on_edge);
        }
    }

    closest_point
}

pub(crate) fn find_line_collider_intersection(
    origin: Vec2,
    mouse_pos: Vec2,
    collider: &Collider,
    transform: &GlobalTransform,
) -> Option<Vec2> {
    // Transform to local space
    let transform_inv = transform.compute_matrix().inverse();
    let local_origin = transform_inv.transform_point(origin.extend(0.0)).truncate();
    let local_mouse = transform_inv
        .transform_point(mouse_pos.extend(0.0))
        .truncate();

    // Get vertices based on collider type
    let vertices = match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::Cuboid(cuboid) => {
            let half_extents = cuboid.half_extents;
            vec![
                Vec2::new(-half_extents.x, -half_extents.y),
                Vec2::new(half_extents.x, -half_extents.y),
                Vec2::new(half_extents.x, half_extents.y),
                Vec2::new(-half_extents.x, half_extents.y),
            ]
        }
        avian2d::parry::shape::TypedShape::Triangle(triangle) => {
            vec![
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ]
        }
        avian2d::parry::shape::TypedShape::Ball(ball) => {
            // For circles, use 32 points for better precision
            let radius = ball.radius;
            let mut vertices = Vec::new();
            for i in 0..32 {
                let angle = (i as f32) * std::f32::consts::TAU / 32.0;
                vertices.push(Vec2::new(radius * angle.cos(), radius * angle.sin()));
            }
            vertices
        }
        avian2d::parry::shape::TypedShape::Capsule(capsule) => {
            // For capsules, use direct mathematical ray-capsule intersection
            let radius = capsule.radius;
            let segment = &capsule.segment;
            let start = Vec2::new(segment.a.x, segment.a.y);
            let end = Vec2::new(segment.b.x, segment.b.y);

            // Calculate capsule direction and length
            let capsule_dir = (end - start).normalize_or_zero();

            // For now, fall back to polygon approximation with corrected vertex generation
            let mut vertices = Vec::new();
            let capsule_perp = Vec2::new(-capsule_dir.y, capsule_dir.x);
            let hemisphere_points = 16;

            // Generate vertices in proper order: top hemisphere from right to left, then bottom hemisphere from left to right
            for i in 0..=hemisphere_points {
                let t = (i as f32) / (hemisphere_points as f32);
                let angle = std::f32::consts::PI * t;
                // Top hemisphere: angles from 0 to π, pointing left/up
                let offset =
                    capsule_perp * radius * angle.cos() - capsule_dir * radius * angle.sin();
                vertices.push(start + offset);
            }

            for i in 0..=hemisphere_points {
                let t = (i as f32) / (hemisphere_points as f32);
                let angle = std::f32::consts::PI * (1.0 - t);
                // Bottom hemisphere: angles from π to 0, pointing right/down
                let offset =
                    capsule_perp * radius * angle.cos() + capsule_dir * radius * angle.sin();
                vertices.push(end + offset);
            }

            vertices
        }
        avian2d::parry::shape::TypedShape::ConvexPolygon(poly) => {
            // For convex polygons, use all vertices
            poly.points()
                .iter()
                .map(|point| Vec2::new(point.x, point.y))
                .collect()
        }
        avian2d::parry::shape::TypedShape::TriMesh(trimesh) => {
            // For triangle meshes, use all vertices
            trimesh
                .vertices()
                .iter()
                .map(|vertex| Vec2::new(vertex.x, vertex.y))
                .collect()
        }
        _ => {
            // Fallback to binary search for unsupported shapes
            return find_line_collider_intersection_fallback(
                origin, mouse_pos, collider, transform,
            );
        }
    };

    // Find intersection using ray-casting approach
    if let Some(local_intersection) =
        find_ray_polygon_intersection(local_origin, local_mouse, &vertices)
    {
        // Transform back to world space
        let world_intersection = transform
            .transform_point(local_intersection.extend(0.0))
            .truncate();
        return Some(world_intersection);
    }

    // Fallback to closest point on polygon edges
    if let Some(local_closest) = find_closest_point_on_polygon_edges(&vertices, local_mouse) {
        let world_closest = transform
            .transform_point(local_closest.extend(0.0))
            .truncate();
        return Some(world_closest);
    }

    // Final fallback
    Some(origin)
}

/// Fallback binary search implementation for unsupported collider shapes
fn find_line_collider_intersection_fallback(
    origin: Vec2,
    mouse_pos: Vec2,
    collider: &Collider,
    transform: &GlobalTransform,
) -> Option<Vec2> {
    // Transform to local space
    let transform_inv = transform.compute_matrix().inverse();
    let local_origin = transform_inv.transform_point(origin.extend(0.0)).truncate();
    let local_mouse = transform_inv
        .transform_point(mouse_pos.extend(0.0))
        .truncate();

    // Calculate direction from origin to mouse
    let direction = (local_mouse - local_origin).normalize_or_zero();

    // If direction is zero, return origin
    if direction == Vec2::ZERO {
        return Some(origin);
    }

    // The goal is to find where the ray from origin in direction of mouse
    // intersects with the collider boundary

    // For a more precise approach, we'll use binary search along the ray
    let max_distance = local_origin.distance(local_mouse) * 1.5; // Extend beyond mouse
    let mut min_dist = 0.0;
    let mut max_dist = max_distance;
    let mut intersection = None;

    // Binary search for the boundary
    for _ in 0..20 {
        // 20 iterations should be enough for good precision
        let mid_dist = (min_dist + max_dist) / 2.0;
        let test_point = local_origin + direction * mid_dist;
        let world_test_point = transform.transform_point(test_point.extend(0.0)).truncate();

        if is_point_inside_collider(world_test_point, collider, transform) {
            min_dist = mid_dist;
            intersection = Some(world_test_point);
        } else {
            max_dist = mid_dist;
        }
    }

    // If we found an intersection, return it
    if let Some(point) = intersection {
        return Some(point);
    }

    // Fallback: if no intersection found, this means origin is outside the collider
    // In this case, find the closest point on the ray that's inside the collider
    let step = 1.0;
    for dist in (0..(max_distance as i32)).step_by(step as usize) {
        let test_point = local_origin + direction * (dist as f32);
        let world_test_point = transform.transform_point(test_point.extend(0.0)).truncate();

        if is_point_inside_collider(world_test_point, collider, transform) {
            return Some(world_test_point);
        }
    }

    // Last resort: return the origin if nothing else works
    Some(origin)
}

/// Find the closest vertex of a collider to a given point
/// Returns the world position of the closest vertex
pub(crate) fn find_closest_vertex(
    point: Vec2,
    collider: &Collider,
    transform: &GlobalTransform,
) -> Option<Vec2> {
    // Transform point to local space
    let transform_inv = transform.compute_matrix().inverse();
    let local_point = transform_inv.transform_point(point.extend(0.0)).truncate();

    // Get vertices based on collider type
    let vertices = match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::Cuboid(cuboid) => {
            let half_extents = cuboid.half_extents;
            vec![
                Vec2::new(-half_extents.x, -half_extents.y),
                Vec2::new(half_extents.x, -half_extents.y),
                Vec2::new(half_extents.x, half_extents.y),
                Vec2::new(-half_extents.x, half_extents.y),
            ]
        }
        avian2d::parry::shape::TypedShape::Triangle(triangle) => {
            vec![
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ]
        }
        avian2d::parry::shape::TypedShape::Ball(ball) => {
            // For circles, use 8 points around the circumference
            let radius = ball.radius;
            let mut vertices = Vec::new();
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::TAU / 8.0;
                vertices.push(Vec2::new(radius * angle.cos(), radius * angle.sin()));
            }
            vertices
        }
        avian2d::parry::shape::TypedShape::Capsule(capsule) => {
            // For capsules, generate proper vertices around the capsule shape
            let radius = capsule.radius;
            let segment = &capsule.segment;
            let start = Vec2::new(segment.a.x, segment.a.y);
            let end = Vec2::new(segment.b.x, segment.b.y);

            let mut vertices = Vec::new();

            // Calculate capsule direction and perpendicular
            let capsule_dir = (end - start).normalize_or_zero();
            let capsule_perp = Vec2::new(-capsule_dir.y, capsule_dir.x);

            // Number of points for each hemisphere (use fewer for vertex snapping)
            let hemisphere_points = 12;

            // Add endpoints
            vertices.push(start);
            vertices.push(end);

            // Top hemisphere points (around start) - pointing outward from capsule
            for i in 0..hemisphere_points {
                let angle = std::f32::consts::PI * (i as f32) / (hemisphere_points as f32);
                // For top hemisphere, use negative capsule_dir to point outward
                let offset =
                    capsule_perp * radius * angle.cos() - capsule_dir * radius * angle.sin();
                vertices.push(start + offset);
            }

            // Bottom hemisphere points (around end) - pointing outward from capsule
            for i in 0..hemisphere_points {
                let angle = std::f32::consts::PI * (i as f32) / (hemisphere_points as f32);
                // For bottom hemisphere, use positive capsule_dir to point outward
                let offset =
                    capsule_perp * radius * angle.cos() + capsule_dir * radius * angle.sin();
                vertices.push(end + offset);
            }

            vertices
        }
        avian2d::parry::shape::TypedShape::ConvexPolygon(poly) => {
            // For convex polygons, use all vertices
            poly.points()
                .iter()
                .map(|point| Vec2::new(point.x, point.y))
                .collect()
        }
        avian2d::parry::shape::TypedShape::TriMesh(trimesh) => {
            // For triangle meshes, use all vertices
            trimesh
                .vertices()
                .iter()
                .map(|vertex| Vec2::new(vertex.x, vertex.y))
                .collect()
        }
        _ => {
            return None; // No vertex-based snapping available for this shape
        }
    };

    // Find closest vertex
    let mut closest_vertex = None;
    let mut min_distance = f32::MAX;

    for vertex in vertices {
        let distance = (local_point - vertex).length();
        if distance < min_distance {
            min_distance = distance;
            closest_vertex = Some(vertex);
        }
    }

    // Transform back to world space
    closest_vertex.map(|v| transform.transform_point(v.extend(0.0)).truncate())
}

/// Unified snapping function for both anchor creation and dragging
///
/// This function provides consistent snapping behavior for both creating new anchors
/// and dragging existing ones. It handles different snapping modes:
/// - Shift+Click: Vertex/center snapping
/// - Ctrl+Click: Precise placement (origin-to-mouse line intersection)
/// - Free placement: Direct mouse position
pub(crate) fn calculate_snapped_position(
    mouse_pos: Vec2,
    collider_transform: &GlobalTransform,
    collider: &Collider,
    shift_pressed: bool,
    ctrl_pressed: bool,
) -> Vec2 {
    let collider_origin = collider_transform.translation().truncate();

    if shift_pressed {
        // Snap mode: find closest vertex or center point
        if let Some(vertex_pos) = find_closest_vertex(mouse_pos, collider, collider_transform) {
            vertex_pos
        } else {
            // Fallback to closest point on collider
            find_closest_point_on_collider(mouse_pos, collider, collider_transform)
        }
    } else if ctrl_pressed {
        // Precise mode: find intersection with origin-to-mouse line
        if let Some(intersection_pos) = find_line_collider_intersection(
            collider_origin,
            mouse_pos,
            collider,
            collider_transform,
        ) {
            intersection_pos
        } else {
            // Fallback to closest point on collider
            find_closest_point_on_collider(mouse_pos, collider, collider_transform)
        }
    } else {
        // Free placement mode: use mouse position directly
        mouse_pos
    }
}

/// Update the dynamic control point position for rectangle colliders
/// Positions the control point at the corner closest to the mouse cursor
pub(crate) fn update_dynamic_rectangle_control_point(
    cursor_pos: Option<Vec2>,
    collider: &Collider,
    transform: &Transform,
    edit_state: &mut super::edit::ColliderEditState,
) {
    // Only update if we're not currently dragging and have a cursor position
    if edit_state.dragging_point.is_none() && cursor_pos.is_some() {
        let cursor_pos = cursor_pos.unwrap();

        // Get the rectangle corners in world space
        let world_corners = match collider.shape_scaled().as_typed_shape() {
            avian2d::parry::shape::TypedShape::Cuboid(cuboid) => {
                let half_extents = cuboid.half_extents;
                let center = transform.translation.truncate();
                let rotation = transform.rotation;

                let local_corners = [
                    Vec2::new(-half_extents.x, -half_extents.y), // Bottom-left
                    Vec2::new(half_extents.x, -half_extents.y),  // Bottom-right
                    Vec2::new(half_extents.x, half_extents.y),   // Top-right
                    Vec2::new(-half_extents.x, half_extents.y),  // Top-left
                ];

                // Transform corners to world space
                local_corners
                    .iter()
                    .map(|&local_point| {
                        let rotated = rotation * Vec3::new(local_point.x, local_point.y, 0.0);
                        center + Vec2::new(rotated.x, rotated.y)
                    })
                    .collect::<Vec<Vec2>>()
            }
            _ => {
                // Fallback for other rectangle representations
                let center = transform.translation.truncate();
                let aabb = collider.aabb(avian2d::math::Vector::ZERO, 0.0);
                let half_extents = (aabb.max - aabb.min) * 0.5;

                vec![
                    center + Vec2::new(-half_extents.x, -half_extents.y), // Bottom-left
                    center + Vec2::new(half_extents.x, -half_extents.y),  // Bottom-right
                    center + Vec2::new(half_extents.x, half_extents.y),   // Top-right
                    center + Vec2::new(-half_extents.x, half_extents.y),  // Top-left
                ]
            }
        };

        // Find the closest corner to the cursor
        let mut closest_corner_index = 0;
        let mut min_distance = f32::INFINITY;

        for (i, corner) in world_corners.iter().enumerate() {
            let distance = corner.distance_squared(cursor_pos);
            if distance < min_distance {
                min_distance = distance;
                closest_corner_index = i;
            }
        }

        // Update the control point position and index
        if let Some(control_point) = edit_state.control_points.first_mut() {
            control_point.position = world_corners[closest_corner_index];
            control_point.vertex_index = Some(closest_corner_index);
        }
    }
}

/// Update the dynamic control point position for circle colliders
/// Positions the control point at the edge closest to the mouse cursor
pub(crate) fn update_dynamic_circle_control_point(
    cursor_pos: Option<Vec2>,
    collider: &Collider,
    transform: &Transform,
    edit_state: &mut super::edit::ColliderEditState,
) {
    // Only update if we're not currently dragging and have a cursor position
    if edit_state.dragging_point.is_none() && cursor_pos.is_some() {
        let cursor_pos = cursor_pos.unwrap();
        let center = transform.translation.truncate();

        // Get the circle radius
        let radius = match collider.shape_scaled().as_typed_shape() {
            avian2d::parry::shape::TypedShape::Ball(ball) => ball.radius,
            _ => {
                // Fallback to AABB-based radius
                let aabb = collider.aabb(avian2d::math::Vector::ZERO, 0.0);
                let size = aabb.max - aabb.min;
                (size.x + size.y) * 0.25 // Average of half extents
            }
        };

        // Calculate the direction from center to cursor
        let direction = (cursor_pos - center).normalize_or_zero();

        // Position the control point at the edge in the direction of the cursor
        let control_point_pos = center + direction * radius;

        // Update the control point position
        if let Some(control_point) = edit_state.control_points.first_mut() {
            control_point.position = control_point_pos;

            // Calculate and store the angle for reference
            let _angle = direction.y.atan2(direction.x);
            control_point.vertex_index = Some(0); // Store angle could be done here if needed
        }
    }
}

/// Update the dynamic control point position for capsule colliders
/// Positions the control point based on cursor proximity to endpoints or radius
pub(crate) fn update_dynamic_capsule_control_point(
    cursor_pos: Option<Vec2>,
    collider: &Collider,
    transform: &Transform,
    edit_state: &mut super::edit::ColliderEditState,
) {
    // Only update if we're not currently dragging and have a cursor position
    if edit_state.dragging_point.is_none() && cursor_pos.is_some() {
        let cursor_pos = cursor_pos.unwrap();
        let center = transform.translation.truncate();
        let rotation = transform.rotation;

        // Get capsule dimensions and endpoints
        let (radius, local_start, local_end) = match collider.shape_scaled().as_typed_shape() {
            avian2d::parry::shape::TypedShape::Capsule(capsule) => {
                let local_start = Vec2::new(capsule.segment.a.x, capsule.segment.a.y);
                let local_end = Vec2::new(capsule.segment.b.x, capsule.segment.b.y);
                (capsule.radius, local_start, local_end)
            }
            _ => {
                // Fallback to AABB-based capsule
                let aabb = collider.aabb(avian2d::math::Vector::ZERO, 0.0);
                let size = aabb.max - aabb.min;
                let radius = size.x * 0.5;
                let half_height = size.y * 0.5;
                (
                    radius,
                    Vec2::new(0.0, half_height),
                    Vec2::new(0.0, -half_height),
                )
            }
        };

        // Transform endpoints to world space
        let world_start =
            center + (rotation * Vec3::new(local_start.x, local_start.y, 0.0)).truncate();
        let world_end = center + (rotation * Vec3::new(local_end.x, local_end.y, 0.0)).truncate();

        // Calculate distances to endpoints and midpoint
        let dist_to_start = cursor_pos.distance_squared(world_start);
        let dist_to_end = cursor_pos.distance_squared(world_end);
        let _dist_to_midpoint = cursor_pos.distance_squared((world_start + world_end) * 0.5);

        // Determine if cursor is close to endpoints (for length control)
        let endpoint_threshold = 30.0 * 30.0; // Squared threshold for efficiency
        let (control_point_pos, point_type) = if dist_to_start < endpoint_threshold {
            // Closest to start endpoint - use length control
            (world_start, super::ControlPointType::LengthControl)
        } else if dist_to_end < endpoint_threshold {
            // Closest to end endpoint - use length control
            (world_end, super::ControlPointType::LengthControl)
        } else {
            // Always show radius control with dynamic edge following
            let capsule_center = (world_start + world_end) * 0.5;
            let capsule_direction = (world_end - world_start).normalize_or_zero();
            let _perpendicular = Vec2::new(-capsule_direction.y, capsule_direction.x);

            // Calculate the projection of cursor position onto the capsule's main axis
            let to_cursor = cursor_pos - capsule_center;
            let projection_length = to_cursor.dot(capsule_direction);

            // Clamp the projection to the capsule's length
            let capsule_half_length = world_start.distance(world_end) * 0.5;
            let clamped_projection =
                projection_length.clamp(-capsule_half_length, capsule_half_length);

            // Find the closest point on the capsule's center line
            let closest_center_point = capsule_center + capsule_direction * clamped_projection;

            // Calculate direction from center line to cursor
            let to_cursor_from_center = cursor_pos - closest_center_point;
            let direction_to_cursor = to_cursor_from_center.normalize_or_zero();

            // Position the radius control point on the capsule edge
            let radius_point = closest_center_point + direction_to_cursor * radius;

            (radius_point, super::ControlPointType::RadiusControl)
        };

        // Update the control point
        if let Some(control_point) = edit_state.control_points.first_mut() {
            control_point.position = control_point_pos;
            control_point.point_type = point_type;

            // Set vertex index based on control type
            control_point.vertex_index = match point_type {
                super::ControlPointType::LengthControl => {
                    if dist_to_start < endpoint_threshold {
                        Some(0)
                    } else {
                        Some(1)
                    }
                }
                super::ControlPointType::RadiusControl => Some(2),
                _ => Some(0),
            };
        }
    }
}
