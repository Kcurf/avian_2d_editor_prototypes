use crate::tr;
use bevy::prelude::*;

use bevy_egui::{
    EguiContext, EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
mod asset_management;
mod font_loader;
mod i18n;
mod image_preview;
mod panel_state;
pub mod theme_colors;

mod collision_layer_ui;
mod entity_inspector;
mod tool_panel;
mod top_bar;

use crate::collider_tools::{PhysicsManager, ToolMode};
use asset_management::AssetManagementPlugin;
use panel_state::PanelControlPlugin;
use theme_colors::ThemeColorsPlugin;

use crate::GizmoCamera;
use crate::selection::EditorSelection;
use collision_layer_ui::CollisionLayerUIPlugin;

pub struct EditorUIPlugin;

impl Plugin for EditorUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_plugins(DefaultInspectorConfigPlugin)
            .add_plugins(PanelControlPlugin)
            .add_plugins(CollisionLayerUIPlugin)
            .add_plugins(AssetManagementPlugin)
            .add_plugins(ThemeColorsPlugin)
            .init_resource::<entity_inspector::TriangleLockState>()
            .add_systems(PostStartup, setup)
            .add_systems(EguiPrimaryContextPass, ui_main)
            .add_systems(PreStartup, i18n::init_translations);
    }
}

pub fn setup(
    mut commands: Commands,
    main_camera: Single<Entity, With<GizmoCamera>>,
    mut setting: ResMut<EguiGlobalSettings>,
) {
    setting.auto_create_primary_context = false;
    let egui_context = EguiContext::default();

    // Apply re_ui styling first (this sets up fonts and base styling)
    re_ui::apply_style_and_install_loaders(egui_context.get());

    // Then enhance with additional fonts for internationalization
    font_loader::initialize_fonts(egui_context.get());

    commands
        .entity(main_camera.into_inner())
        .insert((PrimaryEguiContext, egui_context));
}

// UI with top bar, left tool panel, and right entity inspector
pub fn ui_main(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut bevy_egui::EguiContext, With<PrimaryEguiContext>>()
        .single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();
    let ctx = egui_context.get_mut();

    // 提取所需状态，避免借用冲突
    let current_mode = world
        .get_resource::<State<ToolMode>>()
        .map(|state| *state.get())
        .unwrap_or(ToolMode::Select);

    let physics_paused = world
        .get_resource::<PhysicsManager>()
        .map(|manager| manager.is_physics_paused())
        .unwrap_or(false);

    let selected_entity = world
        .get_resource::<EditorSelection>()
        .and_then(|selection| selection.primary());

    let panel_state = world.get_resource::<panel_state::PanelState>().unwrap();

    // 提取面板状态
    let left_visible = panel_state.left_panel_visible;
    let right_visible = panel_state.right_panel_visible;
    let bottom_visible = panel_state.bottom_panel_visible;

    // Top bar (always visible)
    top_bar::ui(ctx, world, physics_paused);

    // Left panel for tool controls (conditionally visible)
    if left_visible {
        tool_panel::ui(ctx, world, current_mode, selected_entity);
    }

    // Right-side entity inspector panel (conditionally visible)
    if right_visible {
        entity_inspector::ui(ctx, world);
    }

    // Bottom asset management panel (conditionally visible)
    if bottom_visible {
        asset_management_ui(ctx, world);
    }
}

/// Asset management UI for the bottom panel
fn asset_management_ui(ctx: &mut bevy_egui::egui::Context, world: &mut World) {
    bevy_egui::egui::TopBottomPanel::bottom("asset_management")
        .resizable(true)
        .show(ctx, |ui| {
        ui.heading(tr!("asset_management"));

        // Get image asset channel
        let Some(image_channel) = world.get_resource::<asset_management::ImageAssetChannel>()
        else {
            ui.label(tr!("asset_channel_not_available"));
            return;
        };

        let available_images = asset_management::get_available_images(image_channel);
        let sender = image_channel.send.clone();

        ui.horizontal(|ui| {
            ui.label(tr!("loaded_images"));
            ui.label(format!("({})", available_images.len()));

            if ui.button(tr!("import_image")).clicked() {
                let extensions = asset_management::get_supported_image_extensions();
                asset_management::open_load_image_dialog(sender, extensions);
            }
        });

        ui.separator();

        if available_images.is_empty() {
            ui.label(tr!("no_images_loaded"));
        } else {
            // Gallery-style grid layout with scrollable area
            bevy_egui::egui::ScrollArea::vertical()
                .id_salt("asset_gallery_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    bevy_egui::egui::Grid::new("asset_gallery_grid")
                        .spacing(bevy_egui::egui::vec2(12.0, 12.0))
                        .max_col_width(120.0)
                        .min_col_width(100.0)
                        .show(ui, |ui| {
                            let images_per_row = 6; // More columns for gallery layout

                            for (index, image_asset) in available_images.iter().enumerate() {
                                if index > 0 && index % images_per_row == 0 {
                                    ui.end_row();
                                }

                                // Gallery-style card with hover effects
                                let thumbnail_size = bevy_egui::egui::vec2(100.0, 100.0);

                                ui.vertical(|ui| {
                                    // Image container with hover effect
                                    let response = ui.group(|ui| {
                                        ui.set_min_size(thumbnail_size);
                                        ui.set_max_size(thumbnail_size);

                                        // Center the image
                                        ui.centered_and_justified(|ui| {
                                            let ctx = ui.ctx().clone();

                                            if let Some(images) = world.get_resource::<Assets<Image>>() {
                                                // Create image preview texture and display it
                                                if let Some(texture_handle) = crate::ui::image_preview::create_image_preview(
                                                    &ctx,
                                                    images,
                                                    &image_asset.handle,
                                                    (thumbnail_size.x as u32, thumbnail_size.y as u32),
                                                ) {
                                                    ui.add(bevy_egui::egui::Image::new(&texture_handle)
                                                        .fit_to_exact_size(thumbnail_size * 0.9));
                                                } else {
                                                    // Fallback placeholder
                                                    ui.colored_label(
                                                        bevy_egui::egui::Color32::from_gray(128),
                                                        "📷",
                                                    );
                                                }
                                            } else {
                                                // Fallback placeholder
                                                ui.colored_label(
                                                    bevy_egui::egui::Color32::from_gray(128),
                                                    "📷",
                                                );
                                            }
                                        });
                                    });

                                    // Hover tooltip with detailed information
                                    response.response.on_hover_ui(|ui| {
                                        ui.vertical(|ui| {
                                            ui.heading(&image_asset.file_name);
                                            ui.separator();
                                            ui.label(format!("{}: {}×{} px",
                                                tr!("size"),
                                                image_asset.size.x as i32,
                                                image_asset.size.y as i32
                                            ));
                                            if let Ok(time) = image_asset.loaded_at.duration_since(std::time::UNIX_EPOCH) {
                                                ui.label(format!("{}: {}s ago", tr!("loaded"), time.as_secs()));
                                            }
                                        });
                                    });

                                    // Optional: Show compact filename below if space allows
                                    if thumbnail_size.x > 110.0 {
                                        let display_name = if image_asset.file_name.len() > 15 {
                                            format!("{}...", &image_asset.file_name[..15])
                                        } else {
                                            image_asset.file_name.clone()
                                        };
                                        ui.label(display_name);
                                    }
                                });
                            }
                        });
                });
        }
    });
}

pub fn pretty_type_name_str(val: &str) -> String {
    format!("{:?}", disqualified::ShortName(val))
}
