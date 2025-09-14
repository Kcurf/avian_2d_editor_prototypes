use crate::collider_tools::collision_layers::CollisionLayerPresets;
use crate::tr;
use avian2d::prelude::CollisionLayers;
use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::egui::TextEdit;
use bevy_egui::egui::Widget;

/// 统一的碰撞层管理UI组件
/// 将所有碰撞层管理功能集成到一个界面中，无需对话框
pub struct UnifiedCollisionLayerUI;

impl UnifiedCollisionLayerUI {
    /// 渲染统一的碰撞层管理界面
    pub fn render(ui: &mut egui::Ui, world: &mut World, current_layers: &mut CollisionLayers) {
        world.resource_scope::<CollisionLayerPresets, ()>(|world, mut presets| {
            ui.separator();

            // 使用CollapsingHeader包裹预设选择器
            Self::render_preset_selector(ui, &mut presets, current_layers);

            ui.add_space(10.0);

            // 使用CollapsingHeader包裹层管理区域
            ui.collapsing(tr!("layer_management"), |ui| {
                Self::render_layer_management(ui, world, &mut presets, current_layers);
            });

            ui.add_space(10.0);

            // 使用CollapsingHeader包裹详细配置
            Self::render_detailed_config(ui, &mut presets, current_layers);
        });
    }

    /// 渲染预设选择器
    fn render_preset_selector(
        ui: &mut egui::Ui,
        presets: &CollisionLayerPresets,
        current_layers: &mut CollisionLayers,
    ) {
        ui.label(tr!("preset_selector"));

        // 获取当前预设名称作为选中文本
        let selected_preset_name = presets
            .get_preset_name_for_layers(current_layers)
            .unwrap_or_else(|| "Custom".to_string());

        egui::ComboBox::from_label("")
            .selected_text(selected_preset_name)
            .show_ui(ui, |ui| {
                // 统一显示所有预设（基本预设 + 自定义预设）
                for preset_name in presets.get_all_preset_names() {
                    if ui.selectable_label(false, &preset_name).clicked() {
                        if let Some(new_layers) = presets.apply_any_preset(&preset_name) {
                            *current_layers = new_layers;
                        }
                    }
                }
            });
    }

    /// 渲染层管理区域
    fn render_layer_management(
        ui: &mut egui::Ui,
        world: &mut World,
        presets: &mut CollisionLayerPresets,
        current_layers: &mut CollisionLayers,
    ) {
        if !presets.has_layers() {
            ui.vertical(|ui| {
                ui.label(tr!("no_layers_defined"));
                Self::render_add_layer_inline(ui, world, presets);
            });
        } else {
            // 显示可用层
            ui.label(tr!("available_layers"));

            ui.add_space(1.0);

            ui.horizontal_wrapped(|ui| {
                // 收集层信息以避免借用冲突
                let layers: Vec<(String, String, u8)> = presets
                    .layers
                    .iter()
                    .map(|l| (l.name.clone(), l.description.clone(), l.bit))
                    .collect();

                for (name, description, bit) in layers {
                    if ui
                        .button(&name)
                        .on_hover_ui(|ui| {
                            re_ui::Help::new_without_title()
                                .control(tr!("delete"), "Left Click")
                                .ui(ui);
                            ui.separator();
                            ui.label(format!("Bit: {}", bit));
                            if !description.is_empty() {
                                ui.label(&description);
                            }
                        })
                        .clicked()
                    {
                        // 立即更新资源状态以实现实时同步
                        if let Ok(_) = presets.remove_custom_layer(&name) {
                            info!("立即删除层 '{}'", name);
                            // 同时发送事件用于其他系统
                            world.send_event(
                                crate::collider_tools::collision_layers::RemoveCustomLayerEvent {
                                    name: name.clone(),
                                },
                            );
                        }
                    }
                }
            });

            ui.separator();

            // 添加新层（内联）
            Self::render_add_layer_inline(ui, world, presets);

            ui.separator();
            Self::render_save_preset_inline(ui, world, presets, current_layers);
        }
    }

    /// 渲染内联添加层
    fn render_add_layer_inline(
        ui: &mut egui::Ui,
        world: &mut World,
        presets: &mut CollisionLayerPresets,
    ) {
        world.resource_scope(|world, mut state: Mut<AddLayerState>| {
            let show_add_input = state.show_input;
            let new_layer_name = state.name.clone();
            let new_layer_desc = state.description.clone();

            if show_add_input {
                ui.horizontal_wrapped(|ui| {
                    TextEdit::singleline(&mut state.name)
                        .hint_text(tr!("layer_name"))
                        .ui(ui);

                    TextEdit::singleline(&mut state.description)
                        .hint_text(tr!("description"))
                        .ui(ui);

                    if ui.button(tr!("add")).clicked() {
                        if !new_layer_name.trim().is_empty() {
                            // 立即更新资源状态以实现实时同步
                            if let Ok(bit) = presets
                                .add_custom_layer(new_layer_name.trim(), new_layer_desc.trim())
                            {
                                info!("立即添加层 '{}' (位: {})", new_layer_name.trim(), bit);
                            }

                            // 同时发送事件用于其他系统
                            world.send_event(
                                crate::collider_tools::collision_layers::AddCustomLayerEvent {
                                    name: new_layer_name.trim().to_string(),
                                    description: new_layer_desc.trim().to_string(),
                                },
                            );

                            // 重置状态
                            state.show_input = false;
                            state.name.clear();
                            state.description.clear();
                        }
                    }

                    if ui.button(tr!("cancel")).clicked() {
                        state.show_input = false;
                        state.name.clear();
                        state.description.clear();
                    }
                });
            } else {
                if ui.button(tr!("add_new_layer")).clicked() {
                    state.show_input = true;
                }
            }
        });
    }

    /// 渲染内联保存预设
    fn render_save_preset_inline(
        ui: &mut egui::Ui,
        world: &mut World,
        presets: &mut CollisionLayerPresets,
        current_layers: &CollisionLayers,
    ) {
        world.resource_scope(|world, mut state: Mut<SavePresetState>| {
            let show_save_input = state.show_input;
            let preset_name = state.name.clone();
            let preset_desc = state.description.clone();

            if show_save_input {
                ui.vertical(|ui| {
                    TextEdit::singleline(&mut state.name)
                        .hint_text(tr!("preset_name"))
                        .ui(ui);

                    TextEdit::singleline(&mut state.description)
                        .hint_text(tr!("description"))
                        .ui(ui);

                    ui.horizontal(|ui| {
                        if ui.button(tr!("save")).clicked() {
                            if !preset_name.trim().is_empty() {
                                // 立即更新资源状态以实现实时同步
                                if let Ok(_) = presets.save_custom_preset(
                                    preset_name.trim(),
                                    preset_desc.trim(),
                                    current_layers.clone(),
                                ) {
                                    info!("立即保存预设 '{}'", preset_name.trim());
                                }

                                // 同时发送事件用于其他系统
                                world.send_event(
                                    crate::collider_tools::collision_layers::SaveCustomPresetEvent {
                                        name: preset_name.trim().to_string(),
                                        description: preset_desc.trim().to_string(),
                                        layers: current_layers.clone(),
                                    },
                                );

                                // 重置状态
                                state.show_input = false;
                                state.name.clear();
                                state.description.clear();
                            }
                        }

                        if ui.button(tr!("cancel")).clicked() {
                            state.show_input = false;
                            state.name.clear();
                            state.description.clear();
                        }
                    });
                });
            } else {
                if ui.button(tr!("save_as_preset")).clicked() {
                    state.show_input = true;
                }
            }
        });
    }

    /// 渲染详细配置
    fn render_detailed_config(
        ui: &mut egui::Ui,
        presets: &CollisionLayerPresets,
        current_layers: &mut CollisionLayers,
    ) {
        ui.collapsing(tr!("detailed_config"), |ui| {
            // 成员层配置
            ui.label(tr!("member_layers"));
            ui.add_space(5.0);

            let membership_names = presets.layer_mask_to_names(current_layers.memberships);
            ui.horizontal_wrapped(|ui| {
                for layer in &presets.layers {
                    let is_selected = membership_names.contains(&layer.name);
                    if ui.selectable_label(is_selected, &layer.name).clicked() {
                        let mut new_selection = membership_names.clone();
                        if is_selected {
                            new_selection.retain(|name| name != &layer.name);
                        } else {
                            new_selection.push(layer.name.clone());
                        }
                        current_layers.memberships = presets.names_to_layer_mask(&new_selection);
                    }
                }
            });

            ui.add_space(10.0);

            // 过滤器层配置
            ui.label(tr!("filter_layers"));
            ui.add_space(5.0);

            let filter_names = presets.layer_mask_to_names(current_layers.filters);
            ui.horizontal_wrapped(|ui| {
                for layer in &presets.layers {
                    let is_selected = filter_names.contains(&layer.name);
                    if ui.selectable_label(is_selected, &layer.name).clicked() {
                        let mut new_selection = filter_names.clone();
                        if is_selected {
                            new_selection.retain(|name| name != &layer.name);
                        } else {
                            new_selection.push(layer.name.clone());
                        }
                        current_layers.filters = presets.names_to_layer_mask(&new_selection);
                    }
                }
            });
        });
    }
}

/// 添加层状态资源
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct AddLayerState {
    pub show_input: bool,
    pub name: String,
    pub description: String,
}

/// 保存预设状态资源
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct SavePresetState {
    pub show_input: bool,
    pub name: String,
    pub description: String,
}

/// 碰撞层管理UI插件
pub struct CollisionLayerUIPlugin;

impl Plugin for CollisionLayerUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AddLayerState>()
            .init_resource::<SavePresetState>();
    }
}
