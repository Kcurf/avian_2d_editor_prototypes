#![allow(missing_docs)]

mod render;

use bevy::render::view::{
    NoFrustumCulling, VisibilityClass, VisibleEntities, add_visibility_class,
};
use bevy::{prelude::*, render::sync_world::SyncToRenderWorld};

pub struct InfiniteGridPlugin;

impl Plugin for InfiniteGridPlugin {
    fn build(&self, _: &mut App) {}

    fn finish(&self, app: &mut App) {
        render::render_app_builder(app);
    }
}

#[derive(Component, Default)]
#[require(
    InfiniteGridSettings = InfiniteGridSettings::for_2d(),
    Transform = Transform::from_rotation(Quat::from_rotation_arc(Vec3::Y, Vec3::Z)),
    Visibility,
    VisibleEntities,
    NoFrustumCulling,
    SyncToRenderWorld
)]
pub struct InfiniteGrid;

#[derive(Component, Copy, Clone)]
#[require(VisibilityClass)]
#[component(on_add = add_visibility_class::<InfiniteGridSettings>)]
pub struct InfiniteGridSettings {
    /// X轴颜色 (通常为红色系)
    pub x_axis_color: Color,
    /// Z轴颜色 (通常为蓝色系，在3D中表示深度轴)
    pub z_axis_color: Color,
    /// 次要网格线颜色 (较细的网格线)
    pub minor_line_color: Color,
    /// 主要网格线颜色 (较粗的网格线，通常每10个单位一条)
    pub major_line_color: Color,
    /// 网格淡出距离，距离相机超过此值后网格开始淡出
    pub fadeout_distance: f32,
    /// 点状淡出强度，控制网格在视角边缘的淡出效果
    pub dot_fadeout_strength: f32,
    /// 网格缩放比例，控制网格线之间的间距
    pub scale: f32,
}

impl Default for InfiniteGridSettings {
    fn default() -> Self {
        Self {
            x_axis_color: Color::oklch(0.65, 0.24, 27.0),  // 红色X轴
            z_axis_color: Color::oklch(0.65, 0.19, 255.0), // 蓝色Z轴（在2D中可能不太明显）
            minor_line_color: Color::srgb(0.15, 0.15, 0.15), // 较浅的次要线
            major_line_color: Color::srgb(0.3, 0.3, 0.3),  // 较浅的主要线
            fadeout_distance: 50.,                         // 较短的淡出距离，适合2D视图
            dot_fadeout_strength: 0.,                      // 较弱的点状淡出
            scale: 15.0,                                   // 较小的缩放，适合2D的密集网格
        }
    }
}

impl InfiniteGridSettings {
    /// 创建专门用于2D场景的网格设置
    pub const fn for_2d() -> Self {
        Self {
            x_axis_color: Color::oklch(0.65, 0.24, 27.0),  // 红色X轴
            z_axis_color: Color::oklch(0.65, 0.19, 255.0), // 蓝色Z轴（在2D中可能不太明显）
            minor_line_color: Color::srgb(0.15, 0.15, 0.15), // 较浅的次要线
            major_line_color: Color::srgb(0.3, 0.3, 0.3),  // 较浅的主要线
            fadeout_distance: 50.,                         // 较短的淡出距离，适合2D视图
            dot_fadeout_strength: 0.,                      // 较弱的点状淡出
            scale: 15.0,                                   // 较小的缩放，适合2D的密集网格
        }
    }
}
