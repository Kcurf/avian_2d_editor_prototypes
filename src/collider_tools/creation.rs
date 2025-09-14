use crate::{EditorSelection, Selectable};

use super::{
    ColliderType, PreviewCollider, calculate_collider_vertices, utils::add_mass_properties,
};
use avian2d::prelude::*;
use bevy::prelude::*;

use bevy_egui::input::egui_wants_any_input;

// === 配置子结构体 ===

/// 质量属性配置
#[derive(Debug, Clone, Default)]
pub struct MassPropertiesConfig {
    /// 显式质量（None = 自动计算）
    pub mass: Option<f32>,
    /// 密度（用于自动计算质量）
    pub density: f32,
    /// 显式转动惯量（None = 自动计算）
    pub angular_inertia: Option<f32>,
    /// 显式质心（None = 自动计算）
    pub center_of_mass: Option<Vec2>,
    /// 防止子实体质量贡献
    pub no_auto_mass: bool,
    pub no_auto_angular_inertia: bool,
    pub no_auto_center_of_mass: bool,
}

/// 材料属性配置
#[derive(Debug, Clone, Default)]
pub struct MaterialPropertiesConfig {
    /// 摩擦系数（None = 使用全局默认 0.5）
    pub friction: Option<f32>,
    /// 静摩擦系数（None = 使用动摩擦系数）
    pub static_friction: Option<f32>,
    /// 摩擦组合规则
    pub friction_combine_rule: Option<CoefficientCombine>,
    /// 弹性系数（None = 使用全局默认 0.0）
    pub restitution: Option<f32>,
    /// 弹性组合规则
    pub restitution_combine_rule: Option<CoefficientCombine>,
}

/// 运动属性配置
#[derive(Debug, Clone, Default)]
pub struct MotionPropertiesConfig {
    /// 线性阻尼（None = 使用全局默认 0.0）
    pub linear_damping: Option<f32>,
    /// 角度阻尼（None = 使用全局默认 0.0）
    pub angular_damping: Option<f32>,
    /// 重力缩放（None = 使用全局默认 1.0）
    pub gravity_scale: Option<f32>,
    /// 最大线速度（None = 无限制）
    pub max_linear_speed: Option<f32>,
    /// 最大角速度（None = 无限制）
    pub max_angular_speed: Option<f32>,
    /// 锁定轴
    pub locked_axes: Option<LockedAxes>,
    /// 优势值（-127 到 127）
    pub dominance: Option<i8>,
}

/// 碰撞属性配置
#[derive(Debug, Clone, Default)]
pub struct CollisionPropertiesConfig {
    /// 是否为传感器
    pub is_sensor: bool,
    /// 碰撞层（None = 使用默认）
    pub collision_layers: Option<CollisionLayers>,
    /// 碰撞边距
    pub collision_margin: f32,
    /// 推测接触边距（CCD）
    pub speculative_margin: Option<f32>,
    /// 启用扫描CCD
    pub swept_ccd: bool,
    /// 启用碰撞事件
    pub collision_events: bool,
    /// 禁用碰撞体
    pub collider_disabled: bool,
}

/// 性能优化配置
#[derive(Debug, Clone, Default)]
pub struct PerformancePropertiesConfig {
    /// 禁用睡眠
    pub disable_sleeping: bool,
    /// 禁用物理模拟
    pub physics_disabled: bool,
    /// 启用变换插值
    pub transform_interpolation: bool,
}

/// 高级物理配置
#[derive(Debug, Clone, Default)]
pub struct AdvancedPhysicsConfig {
    /// 常力（世界空间）
    pub constant_force: Option<Vec2>,
    /// 常力（本地空间）
    pub constant_local_force: Option<Vec2>,
    /// 常扭矩
    pub constant_torque: Option<f32>,
    /// 常线性加速度（世界空间）
    pub constant_linear_acceleration: Option<Vec2>,
    /// 常线性加速度（本地空间）
    pub constant_local_linear_acceleration: Option<Vec2>,
    /// 常角加速度
    pub constant_angular_acceleration: Option<f32>,
}

/// Creation properties for colliders
///
/// Defines the properties used when creating new colliders.
/// Features a hierarchical configuration system with sensible defaults
/// and extensive customization options.
#[derive(Resource, Debug, Clone)]
pub struct CreationProperties {
    // === 基础配置（极简默认） ===
    pub collider_type: ColliderType,
    pub body_type: RigidBody,
    pub color: Color,

    // === 质量属性（可选覆盖） ===
    pub mass_properties: MassPropertiesConfig,

    // === 材料属性（可选覆盖） ===
    pub material: MaterialPropertiesConfig,

    // === 运动控制（可选覆盖） ===
    pub motion: MotionPropertiesConfig,

    // === 碰撞检测（可选覆盖） ===
    pub collision: CollisionPropertiesConfig,

    // === 性能优化（可选覆盖） ===
    pub performance: PerformancePropertiesConfig,

    // === 高级物理（可选覆盖） ===
    pub advanced: AdvancedPhysicsConfig,
}

impl Default for CreationProperties {
    fn default() -> Self {
        Self::minimal(ColliderType::Rectangle)
    }
}

impl CreationProperties {
    /// 极简默认配置
    pub fn minimal(collider_type: ColliderType) -> Self {
        Self {
            collider_type,
            body_type: RigidBody::Dynamic,
            color: Color::srgb(0.2, 0.8, 0.2),
            mass_properties: MassPropertiesConfig::default(),
            material: MaterialPropertiesConfig::default(),
            motion: MotionPropertiesConfig::default(),
            collision: CollisionPropertiesConfig::default(),
            performance: PerformancePropertiesConfig::default(),
            advanced: AdvancedPhysicsConfig::default(),
        }
    }

    // === 预设配置 ===

    /// 重置为当前碰撞体类型的默认配置
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// 角色控制器预设
    pub fn character_controller(&mut self) {
        self.reset_to_defaults();
        self.mass_properties.mass = Some(1.0);
        self.material.friction = Some(0.0);
        self.material.restitution = Some(0.0);
        self.motion.locked_axes = Some(LockedAxes::ROTATION_LOCKED);
        self.performance.disable_sleeping = true;
    }

    /// 高速物体预设（子弹等）
    pub fn high_speed_object(&mut self) {
        self.reset_to_defaults();
        self.collision.swept_ccd = true;
        self.collision.speculative_margin = Some(0.1);
        self.performance.disable_sleeping = true;
        self.material.friction = Some(0.1);
        self.material.restitution = Some(0.0);
    }

    /// 弹性球预设
    pub fn bouncy_ball(&mut self) {
        self.reset_to_defaults();
        self.mass_properties.mass = Some(0.5);
        self.material.restitution = Some(0.8);
        self.material.friction = Some(0.2);
        self.motion.linear_damping = Some(0.05);
        self.motion.angular_damping = Some(0.02);
    }

    /// 静态平台预设
    pub fn static_platform(&mut self) {
        self.reset_to_defaults();
        self.body_type = RigidBody::Static;
        self.material.friction = Some(0.8);
        self.material.restitution = Some(0.1);
        self.collision.collision_margin = 0.05;
    }

    /// 传感器触发器预设
    pub fn trigger_zone(&mut self) {
        self.reset_to_defaults();
        self.collision.is_sensor = true;
        self.body_type = RigidBody::Static;
        self.collision.collision_events = true;
    }

    /// 物理道具预设（可拾取物品）
    pub fn physics_prop(&mut self) {
        self.reset_to_defaults();
        self.mass_properties.mass = Some(0.1);
        self.material.friction = Some(0.3);
        self.material.restitution = Some(0.4);
        self.motion.linear_damping = Some(0.2);
        self.motion.angular_damping = Some(0.1);
    }

    /// 车辆预设
    pub fn vehicle(&mut self) {
        self.reset_to_defaults();
        self.mass_properties.mass = Some(10.0);
        self.material.friction = Some(0.7);
        self.material.restitution = Some(0.2);
        self.motion.linear_damping = Some(0.3);
        self.motion.angular_damping = Some(0.5);
        self.motion.locked_axes = Some(LockedAxes::TRANSLATION_LOCKED.lock_rotation());
    }

    /// 重力忽略物体预设
    pub fn anti_gravity(&mut self) {
        self.reset_to_defaults();
        self.motion.gravity_scale = Some(0.0);
        self.performance.disable_sleeping = true;
    }

    /// 破坏物体预设
    pub fn destructible(&mut self) {
        self.reset_to_defaults();
        self.mass_properties.mass = Some(0.5);
        self.material.friction = Some(0.4);
        self.material.restitution = Some(0.3);
        self.collision.collision_events = true;
    }
}

/// Resource for collider creation state management
///
/// Tracks the current state of collider creation including:
/// - Selected collider type
/// - Active preview collider (if any)
/// - List of created collider entities
/// Triangle creation step for multi-step triangle creation
#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum TriangleCreationStep {
    /// First step: defining the base edge
    DefiningBaseEdge,
    /// Second step: positioning the third vertex
    PositioningThirdVertex,
}

#[derive(Resource, Default, Clone)]
pub struct ColliderCreationState {
    /// Active preview collider during mouse drag operations
    pub preview_collider: Option<PreviewCollider>,
    /// List of entities representing created colliders
    pub created_colliders: Vec<Entity>,
    /// Current step for triangle creation (None for other shapes)
    pub triangle_creation_step: Option<TriangleCreationStep>,
    /// Base edge for triangle creation (stored after first step)
    pub triangle_base_edge: Option<(Vec2, Vec2)>,
}

pub fn handle_collider_creation_input(
    mut commands: Commands,
    mut state: ResMut<ColliderCreationState>,
    properties: Res<CreationProperties>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
) {
    // Handle keyboard input for canceling creation
    if keyboard.just_pressed(KeyCode::Escape) {
        state.preview_collider = None;
        // Reset triangle creation state if active
        if state.triangle_creation_step.is_some() {
            state.triangle_creation_step = None;
            state.triangle_base_edge = None;
        }
    }
    
    // Handle right mouse button for canceling creation
    if mouse_button.just_pressed(MouseButton::Right) {
        state.preview_collider = None;
        // Reset triangle creation state if active
        if state.triangle_creation_step.is_some() {
            state.triangle_creation_step = None;
            state.triangle_base_edge = None;
        }
    }
    
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        let world_pos = camera
            .viewport_to_world_2d(camera_transform, cursor_pos)
            .unwrap_or(Vec2::ZERO);

        if mouse_button.just_pressed(MouseButton::Left) {
            match properties.collider_type {
                ColliderType::Triangle => {
                    match state.triangle_creation_step {
                        None => {
                            // Start first step: defining base edge
                            state.triangle_creation_step =
                                Some(TriangleCreationStep::DefiningBaseEdge);
                            state.preview_collider = Some(PreviewCollider {
                                start_pos: world_pos,
                                current_pos: world_pos,
                                collider_type: properties.collider_type,
                                vertices: vec![world_pos, world_pos],
                            });
                        }
                        Some(TriangleCreationStep::DefiningBaseEdge) => {
                            // Check if the base edge is too short (point-like)
                            let start_pos = if let Some(ref preview) = state.preview_collider {
                                Some(preview.start_pos)
                            } else {
                                None
                            };
                            
                            if let Some(start_pos) = start_pos {
                                let edge_length = start_pos.distance(world_pos);
                                // If edge is too short, treat this click as confirming the first point
                                // and move to positioning the third vertex
                                if edge_length < 5.0 {
                                    state.triangle_creation_step =
                                        Some(TriangleCreationStep::PositioningThirdVertex);
                                    state.triangle_base_edge = Some((start_pos, start_pos));
                                    
                                    // Update preview to show a point and the potential third vertex
                                    if let Some(ref mut preview) = state.preview_collider {
                                        preview.vertices = vec![start_pos, start_pos, world_pos];
                                    }
                                } else {
                                    // Normal case: Move to second step: positioning third vertex
                                    state.triangle_creation_step =
                                        Some(TriangleCreationStep::PositioningThirdVertex);
                                    state.triangle_base_edge = Some((start_pos, world_pos));
                                    
                                    // Update preview vertices
                                    if let Some(ref mut preview) = state.preview_collider {
                                        preview.vertices = vec![start_pos, world_pos, world_pos];
                                    }
                                }
                            }
                        }
                        Some(TriangleCreationStep::PositioningThirdVertex) => {
                            // Check if we're clicking on the same point as the base (in point mode)
                            let should_create = if let Some((base_start, base_end)) = state.triangle_base_edge {
                                // If base_start and base_end are the same point (point mode)
                                if base_start.distance(base_end) < 1.0 {
                                    // Only create if the third point is different and forms a valid triangle
                                    let third_point = world_pos;
                                    base_start.distance(third_point) > 5.0
                                } else {
                                    // Normal mode, always create
                                    true
                                }
                            } else {
                                // Fallback, always create
                                true
                            };
                            
                            if should_create {
                                // Complete triangle creation
                                if let Some(preview) = state.preview_collider.take() {
                                    create_collider_from_preview(
                                        &mut commands,
                                        &mut state,
                                        &properties,
                                        preview,
                                    );
                                }
                            }
                            // Reset triangle creation state
                            state.triangle_creation_step = None;
                            state.triangle_base_edge = None;
                        }
                    }
                }
                _ => {
                    // Standard single-step creation for other shapes
                    state.preview_collider = Some(PreviewCollider {
                        start_pos: world_pos,
                        current_pos: world_pos,
                        collider_type: properties.collider_type,
                        vertices: calculate_collider_vertices(
                            properties.collider_type,
                            world_pos,
                            world_pos,
                        ),
                    });
                }
            }
        }

        // Handle mouse release to complete creation for non-triangle shapes
        if mouse_button.just_released(MouseButton::Left) {
            match properties.collider_type {
                ColliderType::Triangle => {
                    // Triangle uses multi-step creation, handle in mouse press
                    match state.triangle_creation_step {
                        Some(TriangleCreationStep::DefiningBaseEdge) => {
                            // Move to second step: positioning third vertex
                            state.triangle_creation_step =
                                Some(TriangleCreationStep::PositioningThirdVertex);

                            // Get start_pos before borrowing preview_collider mutably
                            let start_pos = if let Some(ref preview) = state.preview_collider {
                                preview.start_pos
                            } else {
                                world_pos
                            };

                            // Set triangle_base_edge first
                            state.triangle_base_edge = Some((start_pos, world_pos));

                            // Then update preview vertices
                            if let Some(ref mut preview) = state.preview_collider {
                                preview.vertices = vec![start_pos, world_pos, world_pos];
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    // Complete creation for other shapes
                    if let Some(preview) = state.preview_collider.take() {
                        // Only create if there's meaningful size (avoid zero-size colliders)
                        let size_threshold = 0.1;
                        let size = (preview.current_pos - preview.start_pos).length();
                        if size > size_threshold {
                            create_collider_from_preview(
                                &mut commands,
                                &mut state,
                                &properties,
                                preview,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn update_collider_preview(
    mut state: ResMut<ColliderCreationState>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<SpritePickingCamera>>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position() {
        let world_pos = camera
            .viewport_to_world_2d(camera_transform, cursor_pos)
            .unwrap_or(Vec2::ZERO);

        // Store triangle creation state to avoid borrowing issues
        let triangle_step = state.triangle_creation_step;
        let triangle_base_edge = state.triangle_base_edge;

        if let Some(preview) = &mut state.preview_collider {
            preview.current_pos = world_pos;

            match preview.collider_type {
                ColliderType::Triangle => {
                    match triangle_step {
                        Some(TriangleCreationStep::DefiningBaseEdge) => {
                            // First step: show base edge being drawn
                            preview.vertices = vec![preview.start_pos, world_pos];
                        }
                        Some(TriangleCreationStep::PositioningThirdVertex) => {
                            // Second step: show complete triangle with third vertex
                            if let Some((base_start, base_end)) = triangle_base_edge {
                                // Check if we're in point mode (base_start and base_end are the same)
                                if base_start.distance(base_end) < 1.0 {
                                    // In point mode, show the point and the potential third vertex
                                    preview.vertices = vec![base_start, base_start, world_pos];
                                } else {
                                    // Normal mode, show the complete triangle
                                    preview.vertices = vec![base_start, base_end, world_pos];
                                }
                            }
                        }
                        _ => {
                            preview.vertices = calculate_collider_vertices(
                                preview.collider_type,
                                preview.start_pos,
                                world_pos,
                            );
                        }
                    }
                }
                _ => {
                    preview.vertices = calculate_collider_vertices(
                        preview.collider_type,
                        preview.start_pos,
                        world_pos,
                    );
                }
            }
        }
    }
}

pub fn create_collider_from_preview(
    commands: &mut Commands,
    state: &mut ColliderCreationState,
    properties: &CreationProperties,
    preview: PreviewCollider,
) {
    let distance = preview.start_pos.distance(preview.current_pos);
    if distance < 5.0 {
        return; // Skip creating colliders that are too small
    }

    let center = match preview.collider_type {
        ColliderType::Triangle => {
            // Use vertices from preview (which handles both single-step and two-step creation)
            if preview.vertices.len() >= 3 {
                let vertices = &preview.vertices[0..3];
                (vertices[0] + vertices[1] + vertices[2]) / 3.0
            } else {
                // Fallback to midpoint for incomplete triangles
                (preview.start_pos + preview.current_pos) / 2.0
            }
        }
        ColliderType::Polygon => {
            let vertices = calculate_collider_vertices(
                ColliderType::Polygon,
                preview.start_pos,
                preview.current_pos,
            );
            if vertices.len() >= 3 {
                let sum: Vec2 = vertices.iter().sum();
                sum / vertices.len() as f32
            } else {
                (preview.start_pos + preview.current_pos) / 2.0
            }
        }
        ColliderType::Capsule => {
            // For capsule, use the midpoint between start and end positions
            (preview.start_pos + preview.current_pos) / 2.0
        }
        _ => (preview.start_pos + preview.current_pos) / 2.0,
    };

    let collider = match preview.collider_type {
        ColliderType::Rectangle => {
            let size = (preview.current_pos - preview.start_pos)
                .abs()
                .max(Vec2::splat(5.0));
            Collider::rectangle(size.x, size.y)
        }
        ColliderType::Circle => {
            let radius = distance.max(5.0);
            Collider::circle(radius)
        }
        ColliderType::Capsule => {
            let height = distance.max(10.0);
            let radius = (height * 0.2).max(2.0);
            // Calculate center and relative endpoints
            let capsule_center = (preview.start_pos + preview.current_pos) / 2.0;
            let relative_start = preview.start_pos - capsule_center;
            let relative_end = preview.current_pos - capsule_center;
            // Use capsule_endpoints with relative positions since Transform will be at center
            Collider::capsule_endpoints(radius, relative_start, relative_end)
        }
        ColliderType::Triangle => {
            // Use vertices from preview (which handles both single-step and two-step creation)
            if preview.vertices.len() >= 3 {
                let vertices = &preview.vertices[0..3];
                // Check if we have a valid triangle (not degenerate)
                let area = (vertices[1] - vertices[0])
                    .perp_dot(vertices[2] - vertices[0])
                    .abs();
                
                // Additional check for point-mode triangles
                let is_point_mode = vertices[0].distance(vertices[1]) < 1.0;
                let has_valid_third_point = vertices[0].distance(vertices[2]) > 5.0;
                
                if area > 1.0 || (is_point_mode && has_valid_third_point) {
                    let triangle_center = (vertices[0] + vertices[1] + vertices[2]) / 3.0;
                    let centered_vertices = vertices
                        .iter()
                        .map(|v| *v - triangle_center)
                        .collect::<Vec<Vec2>>();
                    Collider::triangle(
                        centered_vertices[0],
                        centered_vertices[1],
                        centered_vertices[2],
                    )
                } else {
                    let diff = preview.current_pos - preview.start_pos;
                    let size = diff.abs().max(Vec2::splat(10.0));
                    Collider::rectangle(size.x, size.y)
                }
            } else {
                // Fallback for incomplete triangles
                let diff = preview.current_pos - preview.start_pos;
                let size = diff.abs().max(Vec2::splat(10.0));
                Collider::rectangle(size.x, size.y)
            }
        }
        ColliderType::Polygon => {
            let vertices = calculate_collider_vertices(
                ColliderType::Polygon,
                preview.start_pos,
                preview.current_pos,
            );
            if vertices.len() >= 3 {
                let polygon_center: Vec2 = vertices.iter().sum();
                let polygon_center = polygon_center / vertices.len() as f32;
                let centered_vertices = vertices
                    .iter()
                    .map(|v| *v - polygon_center)
                    .collect::<Vec<Vec2>>();

                let mut valid = false;
                for i in 2..centered_vertices.len() {
                    let area = (centered_vertices[1] - centered_vertices[0])
                        .perp_dot(centered_vertices[i] - centered_vertices[0])
                        .abs();
                    if area > 1.0 {
                        valid = true;
                        break;
                    }
                }

                if valid {
                    Collider::convex_hull(centered_vertices)
                        .unwrap_or_else(|| Collider::circle(distance.max(10.0)))
                } else {
                    Collider::circle(distance.max(10.0))
                }
            } else {
                Collider::circle(distance.max(10.0))
            }
        }
    };

    let mut entity_commands = commands.spawn((
        Transform::from_xyz(center.x, center.y, 0.0),
        collider,
        properties.body_type,
        preview.collider_type,
        DebugRender {
            collider_color: Some(properties.color),
            axis_lengths: Some(avian2d::math::Vector::new(0.8, 0.8)),
            ..default()
        },
        Selectable,
        Pickable::default(),
    ));

    // === 智能添加质量属性 ===
    add_mass_properties_components(&mut entity_commands, &properties, &preview);

    // === 智能添加材料属性 ===
    add_material_properties(&mut entity_commands, &properties);

    // === 智能添加运动属性 ===
    add_motion_properties(&mut entity_commands, &properties);

    // === 智能添加碰撞属性 ===
    add_collision_properties(&mut entity_commands, &properties);

    // === 智能添加性能属性 ===
    add_performance_properties(&mut entity_commands, &properties);

    // === 智能添加高级物理属性 ===
    add_advanced_physics_properties(&mut entity_commands, &properties);

    let entity = entity_commands.id();
    state.created_colliders.push(entity);
}

/// System called when entering Create mode
pub(super) fn on_enter_create_mode(
    mut state: ResMut<ColliderCreationState>,
    mut selection: ResMut<EditorSelection>,
) {
    info!("Entering Create mode");
    // Clear selection when entering create mode
    selection.clear();

    // Reset creation state
    state.preview_collider = None;
    state.triangle_creation_step = None;
    state.triangle_base_edge = None;
}

/// System called when exiting Create mode
pub(super) fn on_exit_create_mode(mut state: ResMut<ColliderCreationState>) {
    info!("Exiting Create mode");
    // Clear any ongoing creation state
    state.preview_collider = None;
    state.triangle_creation_step = None;
    state.triangle_base_edge = None;
}

/// Plugin for collider creation functionality
#[derive(Default)]
pub struct CreationPlugin;

impl Plugin for CreationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CreationProperties>()
            .init_resource::<ColliderCreationState>()
            .add_systems(OnEnter(super::ToolMode::Create), on_enter_create_mode)
            .add_systems(OnExit(super::ToolMode::Create), on_exit_create_mode)
            .add_systems(
                Update,
                (
                    handle_collider_creation_input,
                    update_collider_preview,
                    update_collider_visualization,
                )
                    .run_if(in_state(super::ToolMode::Create).and(not(egui_wants_any_input))),
            );
    }
}

/// Visualize the preview collider
///
/// Renders the preview collider using Bevy's Gizmos system.
/// Shows a yellow outline of the collider being created.
///
/// #[derive(Resource, Reflect, Debug, Clone)]Parameters
///
/// - `gizmos`: Gizmos resource for drawing
/// - `state`: Current collider creation state
fn update_collider_visualization(mut gizmos: Gizmos, state: Res<ColliderCreationState>) {
    if let Some(ref preview) = state.preview_collider {
        let color = Color::srgba(1.0, 1.0, 0.0, 0.5); // Yellow preview
        super::draw_collider_shape(&mut gizmos, preview, color);
    }
}

// === 智能组件添加函数 ===

fn add_mass_properties_components(
    entity_commands: &mut EntityCommands,
    properties: &CreationProperties,
    preview: &PreviewCollider,
) {
    let mass_config = &properties.mass_properties;

    // 显式质量
    if let Some(mass) = mass_config.mass {
        entity_commands.insert(Mass(mass));
    }

    // 显式转动惯量
    if let Some(inertia) = mass_config.angular_inertia {
        entity_commands.insert(AngularInertia(inertia));
    }

    // 显式质心
    if let Some(com) = mass_config.center_of_mass {
        entity_commands.insert(CenterOfMass(com));
    }

    // 质量属性控制标志
    if mass_config.no_auto_mass {
        entity_commands.insert(NoAutoMass);
    }
    if mass_config.no_auto_angular_inertia {
        entity_commands.insert(NoAutoAngularInertia);
    }
    if mass_config.no_auto_center_of_mass {
        entity_commands.insert(NoAutoCenterOfMass);
    }

    // 自动计算质量（仅动态体且未指定显式质量）
    if properties.body_type == RigidBody::Dynamic
        && mass_config.mass.is_none()
        && mass_config.angular_inertia.is_none()
        && mass_config.center_of_mass.is_none()
    {
        add_mass_properties(entity_commands, preview, mass_config.density);
    }
}

fn add_material_properties(entity_commands: &mut EntityCommands, properties: &CreationProperties) {
    let material = &properties.material;

    // 摩擦力
    if let Some(friction) = material.friction {
        let mut friction_component = Friction::new(friction);

        // 静摩擦系数
        if let Some(static_friction) = material.static_friction {
            friction_component.static_coefficient = static_friction;
        }

        // 组合规则
        if let Some(rule) = material.friction_combine_rule {
            friction_component.combine_rule = rule;
        }

        entity_commands.insert(friction_component);
    }

    // 弹性
    if let Some(restitution) = material.restitution {
        let mut restitution_component = Restitution::new(restitution);

        if let Some(rule) = material.restitution_combine_rule {
            restitution_component.combine_rule = rule;
        }

        entity_commands.insert(restitution_component);
    }
}

fn add_motion_properties(entity_commands: &mut EntityCommands, properties: &CreationProperties) {
    let motion = &properties.motion;

    // 线性阻尼
    if let Some(damping) = motion.linear_damping {
        entity_commands.insert(LinearDamping(damping));
    }

    // 角度阻尼
    if let Some(damping) = motion.angular_damping {
        entity_commands.insert(AngularDamping(damping));
    }

    // 重力缩放
    if let Some(scale) = motion.gravity_scale {
        entity_commands.insert(GravityScale(scale));
    }

    // 最大速度
    if let Some(speed) = motion.max_linear_speed {
        entity_commands.insert(MaxLinearSpeed(speed));
    }

    // 最大角速度
    if let Some(speed) = motion.max_angular_speed {
        entity_commands.insert(MaxAngularSpeed(speed));
    }

    // 锁定轴
    if let Some(locked_axes) = motion.locked_axes {
        if locked_axes.is_translation_x_locked()
            || locked_axes.is_translation_y_locked()
            || locked_axes.is_rotation_locked()
        {
            entity_commands.insert(locked_axes);
        }
    }

    // 优势值
    if let Some(dominance) = motion.dominance {
        entity_commands.insert(Dominance(dominance));
    }
}

fn add_collision_properties(entity_commands: &mut EntityCommands, properties: &CreationProperties) {
    let collision = &properties.collision;

    // 传感器
    if collision.is_sensor {
        entity_commands.insert(Sensor);
    }

    // 碰撞层
    if let Some(layers) = &collision.collision_layers {
        entity_commands.insert(layers.clone());
    }

    // 碰撞边距
    if collision.collision_margin > 0.0 {
        entity_commands.insert(CollisionMargin(collision.collision_margin));
    }

    // 推测接触边距
    if let Some(margin) = collision.speculative_margin {
        entity_commands.insert(SpeculativeMargin(margin));
    }

    // 扫描CCD
    if collision.swept_ccd {
        entity_commands.insert(SweptCcd::default());
    }

    // 碰撞事件
    if collision.collision_events {
        entity_commands.insert(CollisionEventsEnabled);
    }

    // 禁用碰撞体
    if collision.collider_disabled {
        entity_commands.insert(ColliderDisabled);
    }
}

fn add_performance_properties(
    entity_commands: &mut EntityCommands,
    properties: &CreationProperties,
) {
    let performance = &properties.performance;

    // 禁用睡眠
    if performance.disable_sleeping {
        entity_commands.insert(SleepingDisabled);
    }

    // 禁用物理
    if performance.physics_disabled {
        entity_commands.insert(RigidBodyDisabled);
    }

    // 变换插值
    if performance.transform_interpolation {
        entity_commands.insert(TransformInterpolation);
    }
}

fn add_advanced_physics_properties(
    entity_commands: &mut EntityCommands,
    properties: &CreationProperties,
) {
    let advanced = &properties.advanced;

    // 常力
    if let Some(force) = advanced.constant_force {
        entity_commands.insert(ConstantForce(force));
    }

    // 常本地力
    if let Some(force) = advanced.constant_local_force {
        entity_commands.insert(ConstantLocalForce(force));
    }

    // 常扭矩
    if let Some(torque) = advanced.constant_torque {
        entity_commands.insert(ConstantTorque(torque));
    }

    // 常线性加速度
    if let Some(accel) = advanced.constant_linear_acceleration {
        entity_commands.insert(ConstantLinearAcceleration(accel));
    }

    // 常本地线性加速度
    if let Some(accel) = advanced.constant_local_linear_acceleration {
        entity_commands.insert(ConstantLocalLinearAcceleration(accel));
    }

    // 常角加速度
    if let Some(accel) = advanced.constant_angular_acceleration {
        entity_commands.insert(ConstantAngularAcceleration(accel));
    }
}
