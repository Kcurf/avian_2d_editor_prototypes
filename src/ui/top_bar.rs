use avian2d::prelude::Physics;
use bevy::prelude::*;
use bevy_egui::egui;
use re_ui::{
    UiExt as _,
    icons::{PAUSE, PLAY},
};

use crate::ui::panel_state::PanelControlEvent;
use crate::{EditorSelection, ExportSceneEvent, PhysicsManager, tr, ui::i18n};

pub(super) fn ui(ctx: &egui::Context, world: &mut World, physics_paused: bool) {
    // Top bar for physics controls and scene export
    egui::TopBottomPanel::top("top_bar")
        .default_height(40.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(tr!("app_title"));
                ui.separator();

                // Scene export menu
                ui.menu_button(tr!("export_scene"), |ui| {
                    if ui.button(tr!("export_all")).clicked() {
                        world.send_event(ExportSceneEvent::All);
                    }
                    ui.separator();
                    if ui.button(tr!("export_selected")).clicked() {
                        let selected_entities: Vec<Entity> = world
                            .get_resource::<EditorSelection>()
                            .map(|selection| selection.iter().collect())
                            .unwrap_or_default();
                        world.send_event(ExportSceneEvent::Entities(selected_entities));
                        ui.close_kind(bevy_egui::egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button(tr!("export_colliders")).clicked() {
                        world.send_event(ExportSceneEvent::CollidersOnly);
                        ui.close_kind(bevy_egui::egui::UiKind::Menu);
                    }
                    ui.separator();
                    if ui.button(tr!("export_joints")).clicked() {
                        world.send_event(ExportSceneEvent::JointsOnly);
                        ui.close_kind(bevy_egui::egui::UiKind::Menu);
                    }
                });

                ui.separator();

                // Panel controls
                render_panel_controls(ui, world);

                ui.separator();

                // Language switcher
                ui.menu_button(tr!("language_switcher"), |ui| {
                    if ui.button(tr!("english")).clicked() {
                        i18n::set_language("en");
                        ui.close_kind(bevy_egui::egui::UiKind::Menu);
                    }
                    if ui.button(tr!("chinese")).clicked() {
                        i18n::set_language("zh");
                        ui.close_kind(bevy_egui::egui::UiKind::Menu);
                    }
                });

                ui.separator();

                // Theme toggle with proper font color management
                let current_theme = ctx.theme();
                let next_theme = match current_theme {
                    egui::Theme::Light => egui::Theme::Dark,
                    egui::Theme::Dark => egui::Theme::Light,
                };

                // Choose appropriate icon based on current theme
                let (icon_uri, icon_bytes) = match current_theme {
                    egui::Theme::Light => {
                        let uri = "theme_dark.svg";
                        let bytes = include_bytes!("../../assets/icons/theme_dark.svg");
                        (uri, bytes as &[u8])
                    }
                    egui::Theme::Dark => {
                        let uri = "theme_light.svg";
                        let bytes = include_bytes!("../../assets/icons/theme_light.svg");
                        (uri, bytes as &[u8])
                    }
                };

                // Load the icon and create re_ui icon
                ui.ctx().include_bytes(icon_uri, icon_bytes);
                let theme_icon = re_ui::Icon::new(icon_uri, icon_bytes);

                if ui.small_icon_button(&theme_icon, "Toggle theme").clicked() {
                    // Apply theme with proper font color management
                    ctx.set_theme(next_theme);
                }

                ui.separator();

                // Physics controls
                if physics_paused {
                    if ui.small_icon_button(&PLAY, tr!("resume_physics")).clicked() {
                        // Use a closure to handle the resource borrowing
                        world.resource_scope(|world, mut physics_manager: Mut<PhysicsManager>| {
                            if let Some(mut physics_time) =
                                world.get_resource_mut::<Time<Physics>>()
                            {
                                physics_manager.unpause(&mut physics_time);
                            }
                        });
                    }
                } else {
                    if ui.small_icon_button(&PAUSE, tr!("pause_physics")).clicked() {
                        // Use a closure to handle the resource borrowing
                        world.resource_scope(|world, mut physics_manager: Mut<PhysicsManager>| {
                            if let Some(mut physics_time) =
                                world.get_resource_mut::<Time<Physics>>()
                            {
                                physics_manager.pause(&mut physics_time);
                            }
                        });
                    }
                }
            });
        });
}

/// 面板控制按钮
fn render_panel_controls(ui: &mut egui::Ui, world: &mut World) {
    // 提取面板状态，避免借用冲突
    let left_visible = world
        .get_resource::<crate::ui::panel_state::PanelState>()
        .map(|state| state.left_panel_visible)
        .unwrap_or(false);
    let right_visible = world
        .get_resource::<crate::ui::panel_state::PanelState>()
        .map(|state| state.right_panel_visible)
        .unwrap_or(false);
    let bottom_visible = world
        .get_resource::<crate::ui::panel_state::PanelState>()
        .map(|state| state.bottom_panel_visible)
        .unwrap_or(false);

    ui.horizontal(|ui| {
        // 左侧面板按钮
        let left_button = ui.selectable_label(left_visible, tr!("left_panel"));
        if left_button.clicked() {
            world.send_event(PanelControlEvent::ToggleLeftPanel);
        }

        // 右侧面板按钮
        let right_button = ui.selectable_label(right_visible, tr!("right_panel"));
        if right_button.clicked() {
            world.send_event(PanelControlEvent::ToggleRightPanel);
        }

        // 下方面板按钮
        let bottom_button = ui.selectable_label(bottom_visible, tr!("asset_panel"));
        if bottom_button.clicked() {
            world.send_event(PanelControlEvent::ToggleBottomPanel);
        }

        ui.separator();

        if ui.button(tr!("max_viewport")).clicked() {
            world.send_event(PanelControlEvent::MaximizeViewport);
        }
    });
}
