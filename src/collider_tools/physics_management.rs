use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{ColliderType, EditorSelection};

/// 物理忽略标记组件
///
/// 标记带有此组件的实体，使其在物理管理中被忽略，
/// 不会被自动添加或移除 RigidBodyDisabled 组件。
#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct PhysicsIgnore;

/// 物理状态管理器
///
/// 负责跟踪当前活跃的物理操作并控制物理模拟的暂停/恢复状态。
/// 使用引用计数机制，只有当所有操作都结束时才恢复物理模拟。
#[derive(Resource, Debug, Default)]
pub struct PhysicsManager {
    /// 物理模拟是否被暂停
    is_paused: bool,
    /// 暂停前的相对速度（用于恢复）
    previous_relative_speed: f64,
}

impl PhysicsManager {
    pub fn pause(&mut self, physics_time: &mut Time<Physics>) {
        if !self.is_paused {
            self.previous_relative_speed = physics_time.relative_speed_f64();
            physics_time.pause();
            self.is_paused = true;
        }
    }

    pub fn unpause(&mut self, physics_time: &mut Time<Physics>) {
        if self.is_paused {
            physics_time.unpause();
            physics_time.set_relative_speed_f64(self.previous_relative_speed);
            self.is_paused = false;
        }
    }

    /// 检查物理模拟是否被暂停
    pub fn is_physics_paused(&self) -> bool {
        self.is_paused
    }
}

/// 物理管理插件
///
/// 初始化物理状态管理系统并提供调试功能。
pub struct PhysicsManagementPlugin;

impl Plugin for PhysicsManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsManager>()
            .register_type::<PhysicsIgnore>()
            .add_systems(
                Update,
                debug_physics_state.run_if(|manager: Res<PhysicsManager>| manager.is_changed()),
            )
            .add_systems(
                Update,
                manage_selected_entity_physics.run_if(
                    in_state(crate::collider_tools::ToolMode::Anchor)
                        .or(in_state(crate::collider_tools::ToolMode::Edit))
                        .or(in_state(crate::collider_tools::ToolMode::Select))
                        .and(resource_changed::<EditorSelection>),
                ),
            )
            // OnEnter systems for Create and Joint modes
            .add_systems(
                OnEnter(crate::collider_tools::ToolMode::Create),
                remove_rigid_body_disabled,
            )
            .add_systems(
                OnEnter(crate::collider_tools::ToolMode::Joint),
                remove_rigid_body_disabled,
            );
    }
}

/// 调试系统：显示当前物理状态
fn debug_physics_state(physics_time: Res<Time<Physics>>) {
    debug!("Physics paused - Time paused: {}", physics_time.is_paused());
}

/// 自动管理选中物体的物理状态
///
/// 当编辑器中选择了一个带有碰撞体的实体时，自动为其添加 [RigidBodyDisabled] 组件以暂停其物理行为；
/// 当选择变更或取消时，移除该组件以恢复物理行为。
fn manage_selected_entity_physics(
    mut commands: Commands,
    selection: Res<EditorSelection>,
    collider: Query<(), (With<ColliderType>, Without<PhysicsIgnore>)>,
    mut last_selection: Local<Option<Entity>>,
) {
    let this_selection = selection
        .primary()
        .filter(|e| collider.contains(*e))
        .map(|e| commands.entity(e).insert(RigidBodyDisabled).id());

    if let Some(last) = *last_selection {
        if this_selection != Some(last) {
            if collider.contains(last) {
                commands.entity(last).remove::<RigidBodyDisabled>();
            }
            *last_selection = this_selection;
        }
    }
}

fn remove_rigid_body_disabled(
    mut commands: Commands,
    rigid_body_query: Query<(Entity, &RigidBody), Without<PhysicsIgnore>>,
) {
    info!("Disabling physics for all rigid bodies");

    for (entity, _) in rigid_body_query.iter() {
        commands.entity(entity).insert(RigidBodyDisabled);
    }
}
