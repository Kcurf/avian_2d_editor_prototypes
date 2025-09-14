//! 主题颜色管理模块
//!
//! 统一管理所有可视化元素的颜色，确保与egui主题保持一致

use bevy::prelude::*;
use bevy_egui::egui;

/// 编辑器可视化主题颜色
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct EditorThemeColors {
    /// 选中轮廓颜色
    pub selection_outline: Color,
    /// 控制点颜色（顶点）
    pub control_point_vertex: Color,
    /// 控制点颜色（半径控制）
    pub control_point_radius: Color,
    /// 控制点颜色（长度控制）
    pub control_point_length: Color,
    /// 控制点轮廓颜色
    pub control_point_outline: Color,
    /// 预览锚点颜色
    pub preview_anchor: Color,
    /// 预览关节颜色
    pub preview_joint: Color,
    /// 锚点颜色（空闲状态）
    pub anchor_free: Color,
    /// 锚点颜色（已选中）
    pub anchor_selected: Color,
    /// 锚点颜色（在关节中使用）
    pub anchor_in_joint: Color,
    /// 关节连接线颜色（距离关节）
    pub joint_distance: Color,
    /// 关节连接线颜色（旋转关节）
    pub joint_revolute: Color,
    /// 关节连接线颜色（滑动关节）
    pub joint_prismatic: Color,
    /// 关节连接线颜色（固定关节）
    pub joint_fixed: Color,
    /// 选中的关节颜色
    pub joint_selected: Color,
    /// 虚线动画基础颜色
    pub dashed_line_base: Color,
    /// 虚线动画基础颜色（带透明度）
    pub dashed_line_base_alpha: Color,
    /// 滑动关节约束轴颜色（带透明度）
    pub joint_prismatic_constraint_alpha: Color,
    /// 固定关节连接指示器颜色（带透明度）
    pub joint_fixed_constraint_alpha: Color,
    /// TransformGizmo X轴颜色
    pub gizmo_x_axis: Color,
    /// TransformGizmo Y轴颜色
    pub gizmo_y_axis: Color,
    /// TransformGizmo Z轴颜色
    pub gizmo_z_axis: Color,
    /// TransformGizmo X轴选中颜色
    pub gizmo_x_axis_selected: Color,
    /// TransformGizmo Y轴选中颜色
    pub gizmo_y_axis_selected: Color,
    /// TransformGizmo Z轴选中颜色
    pub gizmo_z_axis_selected: Color,
    /// TransformGizmo 视图控制颜色
    pub gizmo_view_control: Color,
}

impl EditorThemeColors {
    /// 创建适用于深色主题的颜色配置
    pub fn dark() -> Self {
        Self {
            selection_outline: Color::srgb(1.0, 0.8, 0.0), // 橙色
            control_point_vertex: Color::srgb(1.0, 0.0, 0.0), // 红色
            control_point_radius: Color::srgb(1.0, 0.5, 0.0), // 橙色
            control_point_length: Color::srgb(0.5, 0.0, 1.0), // 紫色
            control_point_outline: Color::srgb(1.0, 1.0, 1.0), // 白色
            preview_anchor: Color::srgba(1.0, 1.0, 0.0, 0.5), // 半透明黄色
            preview_joint: Color::srgba(1.0, 1.0, 0.0, 0.5), // 半透明黄色
            anchor_free: Color::srgba(0.3, 1.0, 0.3, 0.8), // 亮绿色
            anchor_selected: Color::srgba(1.0, 1.0, 0.0, 1.0), // 亮黄色
            anchor_in_joint: Color::srgba(1.0, 0.3, 0.3, 0.9), // 亮红色
            joint_distance: Color::srgb(0.2, 0.8, 0.8),    // 青色
            joint_revolute: Color::srgb(0.8, 0.2, 0.8),    // 品红色
            joint_prismatic: Color::srgb(0.8, 0.8, 0.2),   // 黄色
            joint_fixed: Color::srgb(0.5, 0.5, 0.5),       // 灰色
            joint_selected: Color::srgb(1.0, 1.0, 0.0),    // 黄色
            dashed_line_base: Color::srgb(0.7, 0.7, 0.7),  // 中灰色
            dashed_line_base_alpha: Color::srgba(0.7, 0.7, 0.7, 0.3), // 半透明中灰色
            joint_prismatic_constraint_alpha: Color::srgba(0.8, 0.8, 0.2, 0.3), // 半透明黄色
            joint_fixed_constraint_alpha: Color::srgba(0.5, 0.5, 0.5, 0.5), // 半透明灰色
            gizmo_x_axis: Color::srgba(0.8, 0.25, 0.32, 0.9), // 红色 X轴: #CC3F51
            gizmo_y_axis: Color::srgba(0.36, 0.7, 0.05, 0.9), // 绿色 Y轴: #5CB20D
            gizmo_z_axis: Color::srgba(0.13, 0.5, 0.8, 0.9), // 蓝色 Z轴: #2180CC
            gizmo_x_axis_selected: Color::srgba(1.0, 0.4, 0.45, 1.0), // 亮红色
            gizmo_y_axis_selected: Color::srgba(0.5, 0.9, 0.2, 1.0), // 亮绿色
            gizmo_z_axis_selected: Color::srgba(0.25, 0.65, 1.0, 1.0), // 亮蓝色
            gizmo_view_control: Color::srgba(0.9, 0.9, 0.9, 0.8), // 中性白/灰
        }
    }

    /// 创建适用于浅色主题的颜色配置
    pub fn light() -> Self {
        Self {
            selection_outline: Color::srgb(0.8, 0.5, 0.0), // 深橙色
            control_point_vertex: Color::srgb(0.8, 0.0, 0.0), // 深红色
            control_point_radius: Color::srgb(0.8, 0.4, 0.0), // 深橙色
            control_point_length: Color::srgb(0.4, 0.0, 0.8), // 深紫色
            control_point_outline: Color::srgb(0.0, 0.0, 0.0), // 黑色
            preview_anchor: Color::srgba(0.8, 0.8, 0.0, 0.5), // 半透明深黄色
            preview_joint: Color::srgba(0.8, 0.8, 0.0, 0.5), // 半透明深黄色
            anchor_free: Color::srgba(0.2, 0.8, 0.2, 0.8), // 深绿色
            anchor_selected: Color::srgba(0.8, 0.8, 0.0, 1.0), // 深黄色
            anchor_in_joint: Color::srgba(0.8, 0.2, 0.2, 0.9), // 深红色
            joint_distance: Color::srgb(0.1, 0.6, 0.6),    // 深青色
            joint_revolute: Color::srgb(0.6, 0.1, 0.6),    // 深品红色
            joint_prismatic: Color::srgb(0.6, 0.6, 0.1),   // 深黄色
            joint_fixed: Color::srgb(0.3, 0.3, 0.3),       // 深灰色
            joint_selected: Color::srgb(0.8, 0.8, 0.0),    // 深黄色
            dashed_line_base: Color::srgb(0.3, 0.3, 0.3),  // 深灰色
            dashed_line_base_alpha: Color::srgba(0.3, 0.3, 0.3, 0.3), // 半透明深灰色
            joint_prismatic_constraint_alpha: Color::srgba(0.6, 0.6, 0.1, 0.3), // 半透明深黄色
            joint_fixed_constraint_alpha: Color::srgba(0.3, 0.3, 0.3, 0.5), // 半透明深灰色
            gizmo_x_axis: Color::srgba(0.7, 0.2, 0.25, 0.9), // 深红色 X轴
            gizmo_y_axis: Color::srgba(0.25, 0.5, 0.04, 0.9), // 深绿色 Y轴
            gizmo_z_axis: Color::srgba(0.1, 0.4, 0.6, 0.9), // 深蓝色 Z轴
            gizmo_x_axis_selected: Color::srgba(0.8, 0.3, 0.35, 1.0), // 深亮红色
            gizmo_y_axis_selected: Color::srgba(0.4, 0.7, 0.15, 1.0), // 深亮绿色
            gizmo_z_axis_selected: Color::srgba(0.2, 0.5, 0.8, 1.0), // 深亮蓝色
            gizmo_view_control: Color::srgba(0.4, 0.4, 0.4, 0.8), // 深灰色
        }
    }

    /// 根据egui主题创建颜色配置
    pub fn from_egui_theme(theme: egui::Theme) -> Self {
        match theme {
            egui::Theme::Light => Self::light(),
            egui::Theme::Dark => Self::dark(),
        }
    }

    /// 更新当前颜色配置以匹配指定主题
    pub fn update_from_egui_theme(&mut self, theme: egui::Theme) {
        *self = Self::from_egui_theme(theme);
    }
}

impl Default for EditorThemeColors {
    fn default() -> Self {
        Self::dark() // 默认使用深色主题
    }
}

/// 系统函数：根据当前egui主题更新编辑器主题颜色
pub fn update_theme_colors(
    mut theme_colors: ResMut<EditorThemeColors>,
    egui_context: Query<&bevy_egui::EguiContext, With<crate::ui::PrimaryEguiContext>>,
) {
    if let Ok(egui_context) = egui_context.single() {
        let current_theme = egui_context.get().theme();
        theme_colors.update_from_egui_theme(current_theme);
    }
}

/// 插件：主题颜色管理
pub struct ThemeColorsPlugin;

impl Plugin for ThemeColorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorThemeColors>()
            .add_systems(Update, update_theme_colors);
    }
}
