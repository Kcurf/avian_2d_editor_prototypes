use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::egui::{self, Context};

use crate::ui::asset_management::{
    ImageAssetChannel, SelectedImageAsset, get_available_images, get_supported_image_extensions,
    open_load_image_dialog,
};
use crate::ui::panel_state::{EntityInspectorMode, EntityInspectorState};
use crate::{EditorSelection, tr};

pub(super) fn ui(ctx: &Context, world: &mut World) {
    egui::SidePanel::right("entity_inspector")
        .resizable(true)
        .default_width(360.0)
        .show(ctx, |ui| {
            let selection = world.resource::<EditorSelection>();
            ui.heading(tr!("entity_inspector"));
            ui.separator();

            // Read current selection
            let selected_primary = selection.primary();

            match selected_primary {
                None => {
                    ui.label(tr!("no_entity_selected_instruction"));
                }
                Some(entity) => {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {:?}", tr!("selected_entity"), entity));
                        // Delete entire entity
                        let delete_entity = ui
                            .button(
                                egui::RichText::new(tr!("delete_entity")).color(egui::Color32::RED),
                            )
                            .on_hover_text(tr!("despawn_entity_tooltip"));
                        if delete_entity.clicked() {
                            world.entity_mut(entity).despawn();
                            return; // entity is gone; skip drawing inspector
                        }
                    });

                    ui.separator();

                    // Get current entity inspector mode
                    let current_mode = world
                        .get_resource::<EntityInspectorState>()
                        .map(|state| state.current_mode)
                        .unwrap_or(EntityInspectorMode::ComponentManagement);

                    // Mode selection
                    ui.heading(tr!("inspector_mode"));
                    ui.horizontal_wrapped(|ui| {
                        if ui
                            .selectable_label(
                                current_mode == EntityInspectorMode::ComponentManagement,
                                tr!("component_management"),
                            )
                            .clicked()
                        {
                            if let Some(mut state) = world.get_resource_mut::<EntityInspectorState>() {
                                state.current_mode = EntityInspectorMode::ComponentManagement;
                            }
                        }
                        if ui
                            .selectable_label(
                                current_mode == EntityInspectorMode::ComponentInspector,
                                tr!("component_inspector"),
                            )
                            .clicked()
                        {
                            if let Some(mut state) = world.get_resource_mut::<EntityInspectorState>() {
                                state.current_mode = EntityInspectorMode::ComponentInspector;
                            }
                        }
                    });

                    ui.separator();

                    // Mode-specific content
                    match current_mode {
                        EntityInspectorMode::ComponentManagement => {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    component_management_ui(ui, world, entity);
                                });
                        }
                        EntityInspectorMode::ComponentInspector => {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    bevy_inspector_egui::bevy_inspector::ui_for_entity_with_children(
                                        world, entity, ui,
                                    );
                                });
                        }
                    }
                }
            }
        });
}

// Component information structure
struct ComponentInfo {
    name: String,
    display_name: String,
    description: String,
}

// Get available component categories with comprehensive component support
fn get_available_component_categories() -> Vec<(&'static str, Vec<ComponentInfo>)> {
    vec![
        (
            "Physics",
            vec![
                ComponentInfo {
                    name: "RigidBody".to_string(),
                    display_name: tr!("rigid_body"),
                    description: tr!("rigid_body_desc"),
                },
                ComponentInfo {
                    name: "Collider".to_string(),
                    display_name: tr!("collider"),
                    description: tr!("collider_desc"),
                },
                ComponentInfo {
                    name: "Mass".to_string(),
                    display_name: tr!("mass"),
                    description: tr!("mass_desc"),
                },
                ComponentInfo {
                    name: "AngularInertia".to_string(),
                    display_name: tr!("angular_inertia"),
                    description: tr!("angular_inertia_desc"),
                },
                ComponentInfo {
                    name: "CenterOfMass".to_string(),
                    display_name: tr!("center_of_mass"),
                    description: tr!("center_of_mass_desc"),
                },
            ],
        ),
        (
            "Material",
            vec![
                ComponentInfo {
                    name: "Friction".to_string(),
                    display_name: tr!("friction"),
                    description: tr!("friction_desc"),
                },
                ComponentInfo {
                    name: "Restitution".to_string(),
                    display_name: tr!("restitution"),
                    description: tr!("restitution_desc"),
                },
            ],
        ),
        (
            "Motion",
            vec![
                ComponentInfo {
                    name: "LinearDamping".to_string(),
                    display_name: tr!("linear_damping"),
                    description: tr!("linear_damping_desc"),
                },
                ComponentInfo {
                    name: "AngularDamping".to_string(),
                    display_name: tr!("angular_damping"),
                    description: tr!("angular_damping_desc"),
                },
                ComponentInfo {
                    name: "GravityScale".to_string(),
                    display_name: tr!("gravity_scale"),
                    description: tr!("gravity_scale_desc"),
                },
                ComponentInfo {
                    name: "MaxLinearSpeed".to_string(),
                    display_name: tr!("max_linear_speed"),
                    description: tr!("max_linear_speed_desc"),
                },
                ComponentInfo {
                    name: "MaxAngularSpeed".to_string(),
                    display_name: tr!("max_angular_speed"),
                    description: tr!("max_angular_speed_desc"),
                },
                ComponentInfo {
                    name: "LockedAxes".to_string(),
                    display_name: tr!("locked_axes"),
                    description: tr!("locked_axes_desc"),
                },
                ComponentInfo {
                    name: "Dominance".to_string(),
                    display_name: tr!("dominance"),
                    description: tr!("dominance_desc"),
                },
            ],
        ),
        (
            "Collision",
            vec![
                ComponentInfo {
                    name: "Sensor".to_string(),
                    display_name: tr!("sensor"),
                    description: tr!("sensor_desc"),
                },
                ComponentInfo {
                    name: "CollisionMargin".to_string(),
                    display_name: tr!("collision_margin"),
                    description: tr!("collision_margin_desc"),
                },
                ComponentInfo {
                    name: "SpeculativeMargin".to_string(),
                    display_name: tr!("speculative_margin"),
                    description: tr!("speculative_margin_desc"),
                },
                ComponentInfo {
                    name: "SweptCcd".to_string(),
                    display_name: tr!("swept_ccd"),
                    description: tr!("swept_ccd_desc"),
                },
                ComponentInfo {
                    name: "CollisionEventsEnabled".to_string(),
                    display_name: tr!("collision_events"),
                    description: tr!("collision_events_desc"),
                },
                ComponentInfo {
                    name: "ColliderDisabled".to_string(),
                    display_name: tr!("collider_disabled"),
                    description: tr!("collider_disabled_desc"),
                },
            ],
        ),
        (
            "Performance",
            vec![
                ComponentInfo {
                    name: "SleepingDisabled".to_string(),
                    display_name: tr!("sleeping_disabled"),
                    description: tr!("sleeping_disabled_desc"),
                },
                ComponentInfo {
                    name: "RigidBodyDisabled".to_string(),
                    display_name: tr!("rigid_body_disabled"),
                    description: tr!("rigid_body_disabled_desc"),
                },
                ComponentInfo {
                    name: "TransformInterpolation".to_string(),
                    display_name: tr!("transform_interpolation"),
                    description: tr!("transform_interpolation_desc"),
                },
            ],
        ),
        (
            "Advanced Physics",
            vec![
                ComponentInfo {
                    name: "ConstantForce".to_string(),
                    display_name: tr!("constant_force"),
                    description: tr!("constant_force_desc"),
                },
                ComponentInfo {
                    name: "ConstantLocalForce".to_string(),
                    display_name: tr!("constant_local_force"),
                    description: tr!("constant_local_force_desc"),
                },
                ComponentInfo {
                    name: "ConstantTorque".to_string(),
                    display_name: tr!("constant_torque"),
                    description: tr!("constant_torque_desc"),
                },
                ComponentInfo {
                    name: "ConstantLinearAcceleration".to_string(),
                    display_name: tr!("constant_linear_acceleration"),
                    description: tr!("constant_linear_acceleration_desc"),
                },
                ComponentInfo {
                    name: "ConstantLocalLinearAcceleration".to_string(),
                    display_name: tr!("constant_local_linear_acceleration"),
                    description: tr!("constant_local_linear_acceleration_desc"),
                },
                ComponentInfo {
                    name: "ConstantAngularAcceleration".to_string(),
                    display_name: tr!("constant_angular_acceleration"),
                    description: tr!("constant_angular_acceleration_desc"),
                },
            ],
        ),
        (
            "Rendering",
            vec![ComponentInfo {
                name: "Sprite".to_string(),
                display_name: tr!("sprite"),
                description: tr!("sprite_desc"),
            }],
        ),
        (
            "Mass Control",
            vec![
                ComponentInfo {
                    name: "NoAutoMass".to_string(),
                    display_name: tr!("no_auto_mass"),
                    description: tr!("no_auto_mass_desc"),
                },
                ComponentInfo {
                    name: "NoAutoAngularInertia".to_string(),
                    display_name: tr!("no_auto_angular_inertia"),
                    description: tr!("no_auto_angular_inertia_desc"),
                },
                ComponentInfo {
                    name: "NoAutoCenterOfMass".to_string(),
                    display_name: tr!("no_auto_center_of_mass"),
                    description: tr!("no_auto_center_of_mass_desc"),
                },
            ],
        ),
    ]
}

fn component_management_ui(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Get entity's current components
    let current_components = get_current_components(world, entity);

    // Get available component categories
    let component_categories = get_available_component_categories();

    // Get available image assets
    let image_channel = world.resource::<ImageAssetChannel>();
    let available_images = get_available_images(image_channel);
    let image_sender = image_channel.send.clone();

    // Show component count
    ui.label(format!(
        "{}: {}",
        tr!("total_components"),
        current_components.len()
    ));

    // Unified component management section
    for (category_name, components) in component_categories {
        ui.collapsing(category_name, |ui| {
            for component_info in components {
                let is_already_added = current_components.contains(&component_info.name);

                ui.horizontal(|ui| {
                    // Component name with description tooltip
                    let label = ui.label(&component_info.display_name);
                    label.on_hover_text(&component_info.description);

                    // Add space between name and button
                    ui.add_space(10.0);

                    // Action button (Add or Remove)
                    if is_already_added {
                        // Show remove button for already added components
                        if ui
                            .button(tr!("remove"))
                            .on_hover_text(tr!("remove_component_tooltip"))
                            .clicked()
                        {
                            remove_component_from_entity(
                                world,
                                entity,
                                &component_info.name,
                            );
                        }
                    } else {
                        // Show add button for available components
                        if component_info.name == "Sprite" {
                            // Special handling for Sprite component - show asset selection panel
                            ui.vertical(|ui| {
                                ui.add_space(5.0);

                                // Image asset selection UI
                                if available_images.is_empty() {
                                    ui.label(tr!("no_assets_available"));
                                    if ui.button(tr!("import_image")).clicked() {
                                        let extensions = get_supported_image_extensions();
                                        open_load_image_dialog(
                                            image_sender.clone(),
                                            extensions,
                                        );
                                    }
                                } else {
                                    // Show available images in a grid or list
                                    ui.label(tr!("available_images"));

                                    world.resource_scope(|world,mut selected_asset: Mut<SelectedImageAsset>| {
                                        let is_no_asset_selected = selected_asset.handle.is_none();

                                        // Image selection combo box
                                        egui::ComboBox::from_label(tr!("no_asset_selected"))
                                            .selected_text(&selected_asset.display_name)
                                            .show_ui(ui, |ui| {
                                                // Add "None" option to clear selection
                                                if ui
                                                    .selectable_label(
                                                        is_no_asset_selected,
                                                        tr!("no_asset_selected"),
                                                    )
                                                    .clicked()
                                                {
                                                    // Clear the persistent resource selection
                                                    selected_asset.handle = None;
                                                    selected_asset.display_name = tr!("no_asset_selected");
                                                }

                                                ui.separator();

                                                for image_asset in &available_images {
                                                    if ui.selectable_label(selected_asset.handle.as_ref().map(|handle|handle.eq(&image_asset.handle)).unwrap_or(false),&image_asset.file_name)

                                                        .on_hover_ui(|ui| {
                                                            let thumbnail_size = egui::vec2(64.0, 64.0);

                                                            // Extract context before mutable operations
                                                            let ctx = ui.ctx().clone();

                                                            // Image preview using our utility function
                                                            if let Some(images) = world.get_resource::<Assets<Image>>() {
                                                                crate::ui::image_preview::show_image_preview_with_info(
                                                                    ui,
                                                                    &ctx,
                                                                    images,
                                                                    image_asset,
                                                                    thumbnail_size,
                                                                );
                                                            } else {
                                                                // Fallback placeholder
                                                                ui.centered_and_justified(|ui| {
                                                                    ui.colored_label(
                                                                        egui::Color32::from_gray(128),
                                                                        tr!("unavailable")
                                                                    );
                                                                });
                                                                ui.label(&image_asset.file_name);
                                                                ui.label(format!("{}×{}", image_asset.size.x, image_asset.size.y));
                                                            }
                                                        })
                                                        .clicked() {

                                                        // Update the persistent resource
                                                        selected_asset.handle = Some(image_asset.handle.clone());
                                                        selected_asset.display_name = image_asset.file_name.clone();
                                                    }

                                                }
                                            });

                                    });


                                    // Import more images button
                                    if ui.button(tr!("import_more_images")).clicked() {
                                        let extensions = get_supported_image_extensions();
                                        open_load_image_dialog(
                                            image_sender.clone(),
                                            extensions,
                                        );
                                    }

                                    // Add sprite button with selected image
                                    if ui
                                        .button(tr!("add"))
                                        .on_hover_text(tr!("add_sprite_tooltip"))
                                        .clicked()
                                    {
                                        let selected_image_handle = world
                                            .get_resource::<SelectedImageAsset>()
                                            .and_then(|asset| asset.handle.clone());

                                        add_component_to_entity(
                                            world,
                                            entity,
                                            &component_info.name,
                                            selected_image_handle,
                                        );
                                    }
                                }
                            });
                        } else {
                            // Normal add button for other components
                            if ui
                                .button(tr!("add"))
                                .on_hover_text(tr!("add_component_tooltip"))
                                .clicked()
                            {
                                add_component_to_entity(
                                    world,
                                    entity,
                                    &component_info.name,
                                    None,
                                );
                            }
                        }
                    }
                });
            }
        });
    }
}

fn get_current_components(world: &World, entity: Entity) -> Vec<String> {
    let mut components = Vec::new();

    // Physics components
    if world.get::<RigidBody>(entity).is_some() {
        components.push("RigidBody".to_string());
    }
    if world.get::<Collider>(entity).is_some() {
        components.push("Collider".to_string());
    }
    if world.get::<Mass>(entity).is_some() {
        components.push("Mass".to_string());
    }
    if world.get::<AngularInertia>(entity).is_some() {
        components.push("AngularInertia".to_string());
    }
    if world.get::<CenterOfMass>(entity).is_some() {
        components.push("CenterOfMass".to_string());
    }

    // Material components
    if world.get::<Friction>(entity).is_some() {
        components.push("Friction".to_string());
    }
    if world.get::<Restitution>(entity).is_some() {
        components.push("Restitution".to_string());
    }

    // Motion components
    if world.get::<LinearDamping>(entity).is_some() {
        components.push("LinearDamping".to_string());
    }
    if world.get::<AngularDamping>(entity).is_some() {
        components.push("AngularDamping".to_string());
    }
    if world.get::<GravityScale>(entity).is_some() {
        components.push("GravityScale".to_string());
    }
    if world.get::<MaxLinearSpeed>(entity).is_some() {
        components.push("MaxLinearSpeed".to_string());
    }
    if world.get::<MaxAngularSpeed>(entity).is_some() {
        components.push("MaxAngularSpeed".to_string());
    }
    if world.get::<LockedAxes>(entity).is_some() {
        components.push("LockedAxes".to_string());
    }
    if world.get::<Dominance>(entity).is_some() {
        components.push("Dominance".to_string());
    }

    // Collision components
    if world.get::<Sensor>(entity).is_some() {
        components.push("Sensor".to_string());
    }

    if world.get::<CollisionMargin>(entity).is_some() {
        components.push("CollisionMargin".to_string());
    }
    if world.get::<SpeculativeMargin>(entity).is_some() {
        components.push("SpeculativeMargin".to_string());
    }
    if world.get::<SweptCcd>(entity).is_some() {
        components.push("SweptCcd".to_string());
    }
    if world.get::<CollisionEventsEnabled>(entity).is_some() {
        components.push("CollisionEventsEnabled".to_string());
    }
    if world.get::<ColliderDisabled>(entity).is_some() {
        components.push("ColliderDisabled".to_string());
    }

    // Performance components
    if world.get::<SleepingDisabled>(entity).is_some() {
        components.push("SleepingDisabled".to_string());
    }
    if world.get::<RigidBodyDisabled>(entity).is_some() {
        components.push("RigidBodyDisabled".to_string());
    }
    if world.get::<TransformInterpolation>(entity).is_some() {
        components.push("TransformInterpolation".to_string());
    }

    // Advanced physics components
    if world.get::<ConstantForce>(entity).is_some() {
        components.push("ConstantForce".to_string());
    }
    if world.get::<ConstantLocalForce>(entity).is_some() {
        components.push("ConstantLocalForce".to_string());
    }
    if world.get::<ConstantTorque>(entity).is_some() {
        components.push("ConstantTorque".to_string());
    }
    if world.get::<ConstantLinearAcceleration>(entity).is_some() {
        components.push("ConstantLinearAcceleration".to_string());
    }
    if world
        .get::<ConstantLocalLinearAcceleration>(entity)
        .is_some()
    {
        components.push("ConstantLocalLinearAcceleration".to_string());
    }
    if world.get::<ConstantAngularAcceleration>(entity).is_some() {
        components.push("ConstantAngularAcceleration".to_string());
    }

    // Rendering components
    if world.get::<Sprite>(entity).is_some() {
        components.push("Sprite".to_string());
    }

    // Mass control components
    if world.get::<NoAutoMass>(entity).is_some() {
        components.push("NoAutoMass".to_string());
    }
    if world.get::<NoAutoAngularInertia>(entity).is_some() {
        components.push("NoAutoAngularInertia".to_string());
    }
    if world.get::<NoAutoCenterOfMass>(entity).is_some() {
        components.push("NoAutoCenterOfMass".to_string());
    }

    components
}

fn add_component_to_entity(
    world: &mut World,
    entity: Entity,
    component_name: &str,
    selected_image_handle: Option<Handle<Image>>,
) {
    let mut commands = world.commands();

    match component_name {
        // Physics components
        "RigidBody" => {
            commands.entity(entity).insert(RigidBody::Dynamic);
        }
        "Collider" => {
            commands
                .entity(entity)
                .insert(Collider::rectangle(1.0, 1.0));
        }
        "Mass" => {
            commands.entity(entity).insert(Mass(1.0));
        }
        "AngularInertia" => {
            commands.entity(entity).insert(AngularInertia(1.0));
        }
        "CenterOfMass" => {
            commands.entity(entity).insert(CenterOfMass(Vec2::ZERO));
        }

        // Material components
        "Friction" => {
            commands.entity(entity).insert(Friction::new(0.5));
        }
        "Restitution" => {
            commands.entity(entity).insert(Restitution::new(0.0));
        }

        // Motion components
        "LinearDamping" => {
            commands.entity(entity).insert(LinearDamping(0.0));
        }
        "AngularDamping" => {
            commands.entity(entity).insert(AngularDamping(0.0));
        }
        "GravityScale" => {
            commands.entity(entity).insert(GravityScale(1.0));
        }
        "MaxLinearSpeed" => {
            commands.entity(entity).insert(MaxLinearSpeed(100.0));
        }
        "MaxAngularSpeed" => {
            commands.entity(entity).insert(MaxAngularSpeed(10.0));
        }
        "LockedAxes" => {
            commands
                .entity(entity)
                .insert(LockedAxes::new().lock_rotation());
        }
        "Dominance" => {
            commands.entity(entity).insert(Dominance(0));
        }

        // Collision components
        "Sensor" => {
            commands.entity(entity).insert(Sensor);
        }
        "CollisionMargin" => {
            commands.entity(entity).insert(CollisionMargin(0.01));
        }
        "SpeculativeMargin" => {
            commands.entity(entity).insert(SpeculativeMargin(0.01));
        }
        "SweptCcd" => {
            commands.entity(entity).insert(SweptCcd::default());
        }
        "CollisionEventsEnabled" => {
            commands.entity(entity).insert(CollisionEventsEnabled);
        }
        "ColliderDisabled" => {
            commands.entity(entity).insert(ColliderDisabled);
        }

        // Performance components
        "SleepingDisabled" => {
            commands.entity(entity).insert(SleepingDisabled);
        }
        "RigidBodyDisabled" => {
            commands.entity(entity).insert(RigidBodyDisabled);
        }
        "TransformInterpolation" => {
            commands.entity(entity).insert(TransformInterpolation);
        }

        // Advanced physics components
        "ConstantForce" => {
            commands.entity(entity).insert(ConstantForce(Vec2::ZERO));
        }
        "ConstantLocalForce" => {
            commands
                .entity(entity)
                .insert(ConstantLocalForce(Vec2::ZERO));
        }
        "ConstantTorque" => {
            commands.entity(entity).insert(ConstantTorque(0.0));
        }
        "ConstantLinearAcceleration" => {
            commands
                .entity(entity)
                .insert(ConstantLinearAcceleration(Vec2::ZERO));
        }
        "ConstantLocalLinearAcceleration" => {
            commands
                .entity(entity)
                .insert(ConstantLocalLinearAcceleration(Vec2::ZERO));
        }
        "ConstantAngularAcceleration" => {
            commands
                .entity(entity)
                .insert(ConstantAngularAcceleration(0.0));
        }

        // Rendering components - Sprite with image support
        "Sprite" => {
            if let Some(image_handle) = selected_image_handle {
                commands.entity(entity).insert(Sprite {
                    image: image_handle,
                    ..default()
                });
            } else {
                // Create a default colored sprite if no image is selected
                commands.entity(entity).insert(Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                });
            }
        }

        // Mass control components
        "NoAutoMass" => {
            commands.entity(entity).insert(NoAutoMass);
        }
        "NoAutoAngularInertia" => {
            commands.entity(entity).insert(NoAutoAngularInertia);
        }
        "NoAutoCenterOfMass" => {
            commands.entity(entity).insert(NoAutoCenterOfMass);
        }

        _ => {}
    }
}

fn remove_component_from_entity(world: &mut World, entity: Entity, component_name: &str) {
    let mut commands = world.commands();

    match component_name {
        // Physics components
        "RigidBody" => {
            commands.entity(entity).remove::<RigidBody>();
        }
        "Collider" => {
            commands.entity(entity).remove::<Collider>();
        }
        "Mass" => {
            commands.entity(entity).remove::<Mass>();
        }
        "AngularInertia" => {
            commands.entity(entity).remove::<AngularInertia>();
        }
        "CenterOfMass" => {
            commands.entity(entity).remove::<CenterOfMass>();
        }

        // Material components
        "Friction" => {
            commands.entity(entity).remove::<Friction>();
        }
        "Restitution" => {
            commands.entity(entity).remove::<Restitution>();
        }

        // Motion components
        "LinearDamping" => {
            commands.entity(entity).remove::<LinearDamping>();
        }
        "AngularDamping" => {
            commands.entity(entity).remove::<AngularDamping>();
        }
        "GravityScale" => {
            commands.entity(entity).remove::<GravityScale>();
        }
        "MaxLinearSpeed" => {
            commands.entity(entity).remove::<MaxLinearSpeed>();
        }
        "MaxAngularSpeed" => {
            commands.entity(entity).remove::<MaxAngularSpeed>();
        }
        "LockedAxes" => {
            commands.entity(entity).remove::<LockedAxes>();
        }
        "Dominance" => {
            commands.entity(entity).remove::<Dominance>();
        }

        // Collision components
        "Sensor" => {
            commands.entity(entity).remove::<Sensor>();
        }
        "CollisionMargin" => {
            commands.entity(entity).remove::<CollisionMargin>();
        }
        "SpeculativeMargin" => {
            commands.entity(entity).remove::<SpeculativeMargin>();
        }
        "SweptCcd" => {
            commands.entity(entity).remove::<SweptCcd>();
        }
        "CollisionEventsEnabled" => {
            commands.entity(entity).remove::<CollisionEventsEnabled>();
        }
        "ColliderDisabled" => {
            commands.entity(entity).remove::<ColliderDisabled>();
        }

        // Performance components
        "SleepingDisabled" => {
            commands.entity(entity).remove::<SleepingDisabled>();
        }
        "RigidBodyDisabled" => {
            commands.entity(entity).remove::<RigidBodyDisabled>();
        }
        "TransformInterpolation" => {
            commands.entity(entity).remove::<TransformInterpolation>();
        }

        // Advanced physics components
        "ConstantForce" => {
            commands.entity(entity).remove::<ConstantForce>();
        }
        "ConstantLocalForce" => {
            commands.entity(entity).remove::<ConstantLocalForce>();
        }
        "ConstantTorque" => {
            commands.entity(entity).remove::<ConstantTorque>();
        }
        "ConstantLinearAcceleration" => {
            commands
                .entity(entity)
                .remove::<ConstantLinearAcceleration>();
        }
        "ConstantLocalLinearAcceleration" => {
            commands
                .entity(entity)
                .remove::<ConstantLocalLinearAcceleration>();
        }
        "ConstantAngularAcceleration" => {
            commands
                .entity(entity)
                .remove::<ConstantAngularAcceleration>();
        }

        // Rendering components
        "Sprite" => {
            commands.entity(entity).remove::<Sprite>();
        }

        // Mass control components
        "NoAutoMass" => {
            commands.entity(entity).remove::<NoAutoMass>();
        }
        "NoAutoAngularInertia" => {
            commands.entity(entity).remove::<NoAutoAngularInertia>();
        }
        "NoAutoCenterOfMass" => {
            commands.entity(entity).remove::<NoAutoCenterOfMass>();
        }

        _ => {}
    }
}
