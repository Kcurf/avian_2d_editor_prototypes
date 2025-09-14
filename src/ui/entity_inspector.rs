use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_egui::egui::{self, Context};

use crate::collider_tools;
use crate::collider_tools::edit;
use crate::collider_tools::visualization;
use crate::ui::asset_management::{
    ImageAssetChannel, SelectedImageAsset, get_available_images, get_supported_image_extensions,
    open_load_image_dialog,
};
use crate::ui::panel_state::{EntityInspectorMode, EntityInspectorState};
use crate::{EditorSelection, tr};

/// Triangle lock state for angles and sides
#[derive(Resource, Default, Debug)]
pub struct TriangleLockState {
    /// Locked angle index (0=A, 1=B, 2=C), None if no angle locked
    pub locked_angle: Option<usize>,
    /// Locked side index (0=AB, 1=BC, 2=CA), None if no side locked
    pub locked_side: Option<usize>,
}

impl TriangleLockState {
    pub fn lock_angle(&mut self, angle_index: usize) {
        self.locked_angle = Some(angle_index);
        self.locked_side = None; // Clear side lock when locking angle
    }

    pub fn lock_side(&mut self, side_index: usize) {
        self.locked_side = Some(side_index);
        self.locked_angle = None; // Clear angle lock when locking side
    }

    pub fn unlock_angle(&mut self) {
        self.locked_angle = None;
    }

    pub fn unlock_side(&mut self) {
        self.locked_side = None;
    }

    pub fn unlock_all(&mut self) {
        self.locked_angle = None;
        self.locked_side = None;
    }

    pub fn is_angle_locked(&self, angle_index: usize) -> bool {
        self.locked_angle == Some(angle_index)
    }

    pub fn is_side_locked(&self, side_index: usize) -> bool {
        self.locked_side == Some(side_index)
    }
}

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
                        if ui
                            .selectable_label(
                                current_mode == EntityInspectorMode::ShapeEdit,
                                tr!("shape_edit"),
                            )
                            .clicked()
                        {
                            if let Some(mut state) = world.get_resource_mut::<EntityInspectorState>() {
                                state.current_mode = EntityInspectorMode::ShapeEdit;
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
                        EntityInspectorMode::ShapeEdit => {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    shape_edit_ui(ui, world, entity);
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

/// Shape editing UI for direct numerical manipulation of collider properties
fn shape_edit_ui(ui: &mut egui::Ui, world: &mut World, entity: Entity) {
    // Check if the entity has a collider component
    let has_collider = world.get::<Collider>(entity).is_some();

    if !has_collider {
        ui.label(tr!("no_collider_for_shape_edit"));
        return;
    }

    ui.heading(tr!("shape_properties"));
    ui.separator();

    // Extract collider data before borrowing world mutably
    let collider_data = world.get::<Collider>(entity).cloned();
    let transform_data = world.get::<Transform>(entity).cloned();
    let collider_type_data = world
        .get::<crate::collider_tools::ColliderType>(entity)
        .cloned();

    let (collider, transform, collider_type) =
        match (collider_data, transform_data, collider_type_data) {
            (Some(collider), Some(transform), Some(collider_type)) => {
                (collider, transform, collider_type)
            }
            _ => {
                ui.label(tr!("missing_required_components"));
                return;
            }
        };

    // Shape-specific editing interface
    match collider_type {
        crate::collider_tools::ColliderType::Rectangle => {
            rectangle_shape_edit_ui(ui, world, entity, &collider, &transform);
        }
        crate::collider_tools::ColliderType::Circle => {
            circle_shape_edit_ui(ui, world, entity, &collider, &transform);
        }
        crate::collider_tools::ColliderType::Capsule => {
            capsule_shape_edit_ui(ui, world, entity, &collider, &transform);
        }
        crate::collider_tools::ColliderType::Triangle => {
            triangle_shape_edit_ui(ui, world, entity, &collider, &transform);
        }
        crate::collider_tools::ColliderType::Polygon => {
            polygon_shape_edit_ui(ui, world, entity, &collider, &transform);
        }
    }

    ui.separator();

    // Transform editing
    ui.heading(tr!("transform_properties"));
    transform_edit_ui(ui, world, entity, &transform);
}

/// Rectangle shape editing interface
fn rectangle_shape_edit_ui(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    collider: &Collider,
    _transform: &Transform,
) {
    ui.label(tr!("rectangle_properties"));

    // Extract current dimensions
    let (mut width, mut height) = match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::Cuboid(cuboid) => {
            let half_extents = cuboid.half_extents;
            (half_extents.x * 2.0, half_extents.y * 2.0)
        }
        _ => {
            // Fallback to AABB-based dimensions
            let aabb = collider.aabb(avian2d::math::Vector::ZERO, 0.0);
            let size = aabb.max - aabb.min;
            (size.x, size.y)
        }
    };

    // Width editing
    ui.horizontal(|ui| {
        ui.label(tr!("width"));
        if ui
            .add(
                egui::DragValue::new(&mut width)
                    .speed(0.1)
                    .range(0.01..=f32::MAX),
            )
            .changed()
        {
            update_rectangle_collider(world, entity, width, height);
        }
    });

    // Height editing
    ui.horizontal(|ui| {
        ui.label(tr!("height"));
        if ui
            .add(
                egui::DragValue::new(&mut height)
                    .speed(0.1)
                    .range(0.01..=f32::MAX),
            )
            .changed()
        {
            update_rectangle_collider(world, entity, width, height);
        }
    });

    // Preset sizes
    ui.label(tr!("preset_sizes"));
    ui.horizontal(|ui| {
        if ui.button(tr!("square_small")).clicked() {
            update_rectangle_collider(world, entity, 5.0, 5.0);
        }
        if ui.button(tr!("square_medium")).clicked() {
            update_rectangle_collider(world, entity, 10.0, 10.0);
        }
        if ui.button(tr!("square_large")).clicked() {
            update_rectangle_collider(world, entity, 20.0, 20.0);
        }
    });

    ui.horizontal(|ui| {
        if ui.button(tr!("rectangle_wide")).clicked() {
            update_rectangle_collider(world, entity, 20.0, 10.0);
        }
        if ui.button(tr!("rectangle_tall")).clicked() {
            update_rectangle_collider(world, entity, 10.0, 20.0);
        }
        if ui.button(tr!("rectangle_wide_large")).clicked() {
            update_rectangle_collider(world, entity, 40.0, 20.0);
        }
    });
}

/// Circle shape editing interface
fn circle_shape_edit_ui(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    collider: &Collider,
    _transform: &Transform,
) {
    ui.label(tr!("circle_properties"));

    // Extract current radius
    let mut radius = match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::Ball(ball) => ball.radius,
        _ => {
            // Fallback to AABB-based radius
            let aabb = collider.aabb(avian2d::math::Vector::ZERO, 0.0);
            let size = aabb.max - aabb.min;
            (size.x + size.y) * 0.25
        }
    };

    // Radius editing
    ui.horizontal(|ui| {
        ui.label(tr!("radius"));
        if ui
            .add(
                egui::DragValue::new(&mut radius)
                    .speed(0.1)
                    .range(0.01..=f32::MAX),
            )
            .changed()
        {
            update_circle_collider(world, entity, radius);
        }
    });

    // Diameter display
    let diameter = radius * 2.0;
    ui.label(format!("{}: {:.2}", tr!("diameter"), diameter));

    // Preset radii
    ui.label(tr!("preset_radii"));
    ui.horizontal(|ui| {
        if ui.button(tr!("radius_small")).clicked() {
            update_circle_collider(world, entity, 2.5);
        }
        if ui.button(tr!("radius_medium")).clicked() {
            update_circle_collider(world, entity, 5.0);
        }
        if ui.button(tr!("radius_large")).clicked() {
            update_circle_collider(world, entity, 10.0);
        }
    });

    ui.horizontal(|ui| {
        if ui.button(tr!("radius_extra_large")).clicked() {
            update_circle_collider(world, entity, 20.0);
        }
        if ui.button(tr!("radius_huge")).clicked() {
            update_circle_collider(world, entity, 40.0);
        }
    });
}

/// Capsule shape editing interface
fn capsule_shape_edit_ui(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    collider: &Collider,
    transform: &Transform,
) {
    ui.label(tr!("capsule_properties"));

    // Extract current capsule properties
    let (mut radius, half_height, mut rotation) = match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::Capsule(capsule) => {
            let radius = capsule.radius;
            let half_height = capsule.half_height();
            let segment = &capsule.segment;
            let local_start = Vec2::new(segment.a.x, segment.a.y);
            let local_end = Vec2::new(segment.b.x, segment.b.y);
            let direction = (local_end - local_start).normalize_or_zero();
            let rotation = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
            (radius, half_height, rotation)
        }
        _ => {
            // Fallback values
            (0.25, 0.5, 0.0)
        }
    };

    // Add current transform rotation to the capsule rotation
    rotation += transform.rotation.to_euler(EulerRot::XYZ).2;

    // Radius editing
    ui.horizontal(|ui| {
        ui.label(tr!("radius"));
        if ui
            .add(
                egui::DragValue::new(&mut radius)
                    .speed(0.05)
                    .range(0.01..=f32::MAX),
            )
            .changed()
        {
            update_capsule_collider(world, entity, radius, half_height, rotation);
        }
    });

    // Height editing (full height)
    let mut full_height = half_height * 2.0;
    ui.horizontal(|ui| {
        ui.label(tr!("height"));
        if ui
            .add(
                egui::DragValue::new(&mut full_height)
                    .speed(0.1)
                    .range(0.02..=f32::MAX),
            )
            .changed()
        {
            update_capsule_collider(world, entity, radius, full_height / 2.0, rotation);
        }
    });

    // Rotation editing (in degrees for better UX)
    let mut rotation_degrees = rotation.to_degrees();
    ui.horizontal(|ui| {
        ui.label(tr!("rotation"));
        if ui
            .add(egui::DragValue::new(&mut rotation_degrees).speed(1.0))
            .changed()
        {
            update_capsule_collider(
                world,
                entity,
                radius,
                half_height,
                rotation_degrees.to_radians(),
            );
        }
    });

    // Preset capsule configurations
    ui.label(tr!("preset_capsules"));
    ui.horizontal(|ui| {
        if ui.button(tr!("capsule_pill")).clicked() {
            update_capsule_collider(world, entity, 2.5, 5.0, 0.0);
        }
        if ui.button(tr!("capsule_tall")).clicked() {
            update_capsule_collider(world, entity, 2.5, 10.0, 0.0);
        }
        if ui.button(tr!("capsule_wide")).clicked() {
            update_capsule_collider(world, entity, 5.0, 2.5, 0.0);
        }
    });
}

/// Triangle shape editing interface
fn triangle_shape_edit_ui(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    collider: &Collider,
    transform: &Transform,
) {
    ui.label(tr!("triangle_properties"));

    // Get lock state - extract before UI closures
    let lock_state = world.get_resource::<TriangleLockState>().unwrap();
    let angle_locked = [
        lock_state.is_angle_locked(0),
        lock_state.is_angle_locked(1),
        lock_state.is_angle_locked(2),
    ];
    let side_locked = [
        lock_state.is_side_locked(0),
        lock_state.is_side_locked(1),
        lock_state.is_side_locked(2),
    ];

    ui.separator();

    match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::Triangle(triangle) => {
            let vertices = [
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ];

            // Calculate current side lengths and angles
            let side_a = vertices[0].distance(vertices[1]);
            let side_b = vertices[1].distance(vertices[2]);
            let side_c = vertices[2].distance(vertices[0]);
            let angles = calculate_triangle_angles(&vertices);

            // Side length controls with new algorithm
            ui.label(tr!("triangle_side_lengths"));

            // Side AB lockable label
            ui.horizontal(|ui| {
                let is_locked = side_locked[0];
                let label_text = if is_locked {
                    format!("🔒 {}", tr!("side_ab"))
                } else {
                    tr!("side_ab")
                };
                if ui.selectable_label(is_locked, label_text).clicked() {
                    let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                    if is_locked {
                        lock_state.unlock_side();
                    } else {
                        lock_state.lock_side(0);
                    }
                }
                let mut new_length = side_a;
                let drag_value = egui::DragValue::new(&mut new_length)
                    .speed(0.1)
                    .range(0.001..=f32::MAX);
                if ui.add_enabled(!is_locked, drag_value).changed() {
                    update_triangle_side_length(world, entity, 0, new_length, transform);
                }
                ui.label(format!("(当前: {:.1})", side_a));
            });

            // Side BC lockable label
            ui.horizontal(|ui| {
                let is_locked = side_locked[1];
                let label_text = if is_locked {
                    format!("🔒 {}", tr!("side_bc"))
                } else {
                    tr!("side_bc")
                };
                if ui.selectable_label(is_locked, label_text).clicked() {
                    let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                    if is_locked {
                        lock_state.unlock_side();
                    } else {
                        lock_state.lock_side(1);
                    }
                }
                let mut new_length = side_b;
                let drag_value = egui::DragValue::new(&mut new_length)
                    .speed(0.1)
                    .range(0.001..=f32::MAX);
                if ui.add_enabled(!is_locked, drag_value).changed() {
                    update_triangle_side_length(world, entity, 1, new_length, transform);
                }
                ui.label(format!("(当前: {:.1})", side_b));
            });

            // Side CA lockable label
            ui.horizontal(|ui| {
                let is_locked = side_locked[2];
                let label_text = if is_locked {
                    format!("🔒 {}", tr!("side_ca"))
                } else {
                    tr!("side_ca")
                };
                if ui.selectable_label(is_locked, label_text).clicked() {
                    let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                    if is_locked {
                        lock_state.unlock_side();
                    } else {
                        lock_state.lock_side(2);
                    }
                }
                let mut new_length = side_c;
                let drag_value = egui::DragValue::new(&mut new_length)
                    .speed(0.1)
                    .range(0.001..=f32::MAX);
                if ui.add_enabled(!is_locked, drag_value).changed() {
                    update_triangle_side_length(world, entity, 2, new_length, transform);
                }
                ui.label(format!("(当前: {:.1})", side_c));
            });

            ui.separator();

            // Triangle radius control (circumradius)
            ui.label(tr!("triangle_radius"));
            let circumradius = calculate_triangle_circumradius(&vertices);
            ui.horizontal(|ui| {
                let mut new_radius = circumradius;
                if ui
                    .add(
                        egui::DragValue::new(&mut new_radius)
                            .speed(0.1)
                            .range(0.001..=f32::MAX),
                    )
                    .changed()
                {
                    scale_triangle_from_circumradius(world, entity, new_radius, transform);
                }
                ui.label(format!("(当前: {:.1})", circumradius));
            });

            ui.separator();

            // Angle controls with new algorithm - minimum 0.1 degree increments
            ui.label(tr!("triangle_angles"));

            // Angle A lockable label
            ui.horizontal(|ui| {
                let is_locked = angle_locked[0];
                let label_text = if is_locked {
                    format!("🔒 {}", tr!("angle_at_a"))
                } else {
                    tr!("angle_at_a")
                };
                if ui.selectable_label(is_locked, label_text).clicked() {
                    let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                    if is_locked {
                        lock_state.unlock_angle();
                    } else {
                        lock_state.lock_angle(0);
                    }
                }
                let mut new_angle = angles[0];
                let drag_value = egui::DragValue::new(&mut new_angle)
                    .speed(0.1)
                    .range(2.5..=175.0);
                if ui.add_enabled(!is_locked, drag_value).changed() {
                    update_triangle_angle_new(world, entity, 0, new_angle, transform);
                }
                ui.label(format!("(当前: {:.1}°)", angles[0]));
            });

            // Angle B lockable label
            ui.horizontal(|ui| {
                let is_locked = angle_locked[1];
                let label_text = if is_locked {
                    format!("🔒 {}", tr!("angle_at_b"))
                } else {
                    tr!("angle_at_b")
                };
                if ui.selectable_label(is_locked, label_text).clicked() {
                    let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                    if is_locked {
                        lock_state.unlock_angle();
                    } else {
                        lock_state.lock_angle(1);
                    }
                }
                let mut new_angle = angles[1];
                let drag_value = egui::DragValue::new(&mut new_angle)
                    .speed(0.1)
                    .range(2.5..=175.0);
                if ui.add_enabled(!is_locked, drag_value).changed() {
                    update_triangle_angle_new(world, entity, 1, new_angle, transform);
                }
                ui.label(format!("(当前: {:.1}°)", angles[1]));
            });

            // Angle C lockable label
            ui.horizontal(|ui| {
                let is_locked = angle_locked[2];
                let label_text = if is_locked {
                    format!("🔒 {}", tr!("angle_at_c"))
                } else {
                    tr!("angle_at_c")
                };
                if ui.selectable_label(is_locked, label_text).clicked() {
                    let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                    if is_locked {
                        lock_state.unlock_angle();
                    } else {
                        lock_state.lock_angle(2);
                    }
                }
                let mut new_angle = angles[2];
                let drag_value = egui::DragValue::new(&mut new_angle)
                    .speed(0.1)
                    .range(2.5..=175.0);
                if ui.add_enabled(!is_locked, drag_value).changed() {
                    update_triangle_angle_new(world, entity, 2, new_angle, transform);
                }
                ui.label(format!("(当前: {:.1}°)", angles[2]));
            });

            ui.separator();

            // Clear all locks button
            if ui.button(tr!("clear_all_locks")).clicked() {
                let mut lock_state = world.get_resource_mut::<TriangleLockState>().unwrap();
                lock_state.unlock_all();
            }

            ui.separator();

            // Preset triangle types
            ui.label(tr!("preset_triangles"));
            ui.horizontal(|ui| {
                if ui.button(tr!("equilateral_triangle")).clicked() {
                    let preset_vertices =
                        preset_equilateral_triangle(Vec2::ZERO, 10.0, Quat::IDENTITY);
                    update_triangle_collider(world, entity, &preset_vertices, transform);
                }
                if ui
                    .button(tr!("equilateral_triangle").to_string() + " (20)")
                    .clicked()
                {
                    let preset_vertices =
                        preset_equilateral_triangle(Vec2::ZERO, 20.0, Quat::IDENTITY);
                    update_triangle_collider(world, entity, &preset_vertices, transform);
                }
            });
            ui.horizontal(|ui| {
                if ui.button(tr!("right_triangle")).clicked() {
                    let preset_vertices = preset_right_triangle(Vec2::ZERO, 10.0, Quat::IDENTITY);
                    update_triangle_collider(world, entity, &preset_vertices, transform);
                }
                if ui
                    .button(tr!("right_triangle").to_string() + " (20)")
                    .clicked()
                {
                    let preset_vertices = preset_right_triangle(Vec2::ZERO, 20.0, Quat::IDENTITY);
                    update_triangle_collider(world, entity, &preset_vertices, transform);
                }
            });

            ui.separator();

            // Advanced vertex editing (collapsible)
            ui.collapsing(tr!("triangle_vertices"), |ui| {
                let center = transform.translation.truncate();
                let rotation = transform.rotation;

                let mut world_vertices: Vec<Vec2> = vertices
                    .iter()
                    .map(|vertex| center + (rotation * vertex.extend(0.0)).truncate())
                    .collect();

                for i in 0..world_vertices.len() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} {}", tr!("vertex"), i + 1));
                        let mut x = world_vertices[i].x;
                        let mut y = world_vertices[i].y;

                        let x_changed = ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                        let y_changed = ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();

                        if x_changed || y_changed {
                            world_vertices[i] = Vec2::new(x, y);
                        }
                    });
                }

                if ui.button(tr!("apply_triangle_changes")).clicked() {
                    let rotation_inv = rotation.inverse();
                    let local_vertices: Vec<Vec2> = world_vertices
                        .iter()
                        .map(|world_vertex| {
                            let offset = *world_vertex - center;
                            (rotation_inv * offset.extend(0.0)).truncate()
                        })
                        .collect();
                    if local_vertices.len() == 3 {
                        let triangle_array: [Vec2; 3] =
                            [local_vertices[0], local_vertices[1], local_vertices[2]];
                        update_triangle_collider(world, entity, &triangle_array, transform);
                    }
                }
            });
        }
        _ => {
            ui.label(tr!("invalid_triangle_shape"));
        }
    }
}

/// Polygon shape editing interface
fn polygon_shape_edit_ui(
    ui: &mut egui::Ui,
    world: &mut World,
    entity: Entity,
    collider: &Collider,
    transform: &Transform,
) {
    ui.label(tr!("polygon_properties"));

    match collider.shape_scaled().as_typed_shape() {
        avian2d::parry::shape::TypedShape::ConvexPolygon(poly) => {
            let vertices: Vec<Vec2> = poly
                .points()
                .iter()
                .map(|point| Vec2::new(point.x, point.y))
                .collect();

            ui.label(format!("{}: {}", tr!("vertex_count"), vertices.len()));

            // Calculate polygon properties for ergonomic editing
            let center = vertices.iter().sum::<Vec2>() / vertices.len() as f32;
            let avg_radius =
                vertices.iter().map(|v| v.distance(center)).sum::<f32>() / vertices.len() as f32;

            // Calculate average side length
            let mut side_lengths = Vec::new();
            for i in 0..vertices.len() {
                let next_i = (i + 1) % vertices.len();
                side_lengths.push(vertices[i].distance(vertices[next_i]));
            }
            let avg_side_length = side_lengths.iter().sum::<f32>() / side_lengths.len() as f32;

            // Main controls: radius and side length
            ui.label(tr!("polygon_main_controls"));

            ui.horizontal(|ui| {
                ui.label(tr!("radius"));
                let mut radius = avg_radius;
                if ui
                    .add(
                        egui::DragValue::new(&mut radius)
                            .speed(0.1)
                            .range(0.1..=100.0),
                    )
                    .changed()
                {
                    scale_polygon_from_radius(world, entity, radius, transform);
                }
            });

            ui.horizontal(|ui| {
                ui.label(tr!("avg_side_length"));
                let mut side_length = avg_side_length;
                if ui
                    .add(
                        egui::DragValue::new(&mut side_length)
                            .speed(0.1)
                            .range(0.1..=100.0),
                    )
                    .changed()
                {
                    scale_polygon_from_side_length(world, entity, side_length, transform);
                }
            });

            // Advanced controls for simple polygons
            if vertices.len() <= 8 {
                ui.collapsing(tr!("advanced_polygon_controls"), |ui| {
                    // Individual side length controls
                    ui.label(tr!("individual_side_lengths"));
                    for (i, &side_len) in side_lengths.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("Side {}: ", i + 1));
                            let mut new_len = side_len;
                            if ui
                                .add(
                                    egui::DragValue::new(&mut new_len)
                                        .speed(0.05)
                                        .range(0.001..=f32::MAX),
                                )
                                .changed()
                            {
                                adjust_polygon_side(world, entity, i, new_len, transform);
                            }
                        });
                    }

                    // Individual vertex controls
                    ui.label(tr!("individual_vertices"));
                    let rotation = transform.rotation;
                    let world_vertices: Vec<Vec2> = vertices
                        .iter()
                        .map(|vertex| {
                            let world_pos = transform.translation.truncate()
                                + (rotation * vertex.extend(0.0)).truncate();
                            world_pos
                        })
                        .collect();

                    for (i, world_vertex) in world_vertices.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("V{}: ", i + 1));
                            let mut x = world_vertex.x;
                            let mut y = world_vertex.y;

                            let x_changed =
                                ui.add(egui::DragValue::new(&mut x).speed(0.1)).changed();
                            let y_changed =
                                ui.add(egui::DragValue::new(&mut y).speed(0.1)).changed();

                            if x_changed || y_changed {
                                let new_world_vertices: Vec<Vec2> = world_vertices
                                    .iter()
                                    .enumerate()
                                    .map(|(j, &v)| if j == i { Vec2::new(x, y) } else { v })
                                    .collect();
                                update_polygon_from_world_vertices(
                                    world,
                                    entity,
                                    &new_world_vertices,
                                    transform,
                                );
                            }
                        });
                    }
                });
            } else {
                ui.label(tr!("polygon_too_complex_for_detailed_editing"));
            }

            // Preset polygons
            ui.label(tr!("preset_polygons"));
            ui.horizontal(|ui| {
                if ui.button(tr!("pentagon_10")).clicked() {
                    create_preset_polygon(world, entity, 5, 10.0, transform);
                }
                if ui.button(tr!("hexagon_10")).clicked() {
                    create_preset_polygon(world, entity, 6, 10.0, transform);
                }
                if ui.button(tr!("octagon_10")).clicked() {
                    create_preset_polygon(world, entity, 8, 10.0, transform);
                }
            });
            ui.horizontal(|ui| {
                if ui.button(tr!("pentagon_20")).clicked() {
                    create_preset_polygon(world, entity, 5, 20.0, transform);
                }
                if ui.button(tr!("hexagon_20")).clicked() {
                    create_preset_polygon(world, entity, 6, 20.0, transform);
                }
                if ui.button(tr!("octagon_20")).clicked() {
                    create_preset_polygon(world, entity, 8, 20.0, transform);
                }
            });
        }
        _ => {
            ui.label(tr!("invalid_polygon_shape"));
        }
    }
}

/// Transform editing interface
fn transform_edit_ui(ui: &mut egui::Ui, world: &mut World, entity: Entity, transform: &Transform) {
    let mut pos = transform.translation;
    let rot = transform.rotation.to_euler(EulerRot::XYZ).2; // Z-axis rotation
    let mut scale = transform.scale;

    // Position editing
    ui.horizontal(|ui| {
        ui.label(tr!("position"));
        let x_changed = ui
            .add(egui::DragValue::new(&mut pos.x).speed(0.1))
            .changed();
        let y_changed = ui
            .add(egui::DragValue::new(&mut pos.y).speed(0.1))
            .changed();

        if x_changed || y_changed {
            let mut new_transform = *transform;
            new_transform.translation = pos;
            update_entity_transform(world, entity, new_transform);
        }
    });

    // Rotation editing (in degrees for better UX)
    let mut rot_degrees = rot.to_degrees();
    ui.horizontal(|ui| {
        ui.label(tr!("rotation"));
        if ui
            .add(egui::DragValue::new(&mut rot_degrees).speed(1.0))
            .changed()
        {
            let mut new_transform = *transform;
            let rotation = Quat::from_rotation_z(rot_degrees.to_radians());
            new_transform.rotation = rotation;
            update_entity_transform(world, entity, new_transform);
        }
    });

    // Scale editing
    ui.horizontal(|ui| {
        ui.label(tr!("scale"));
        let x_changed = ui
            .add(
                egui::DragValue::new(&mut scale.x)
                    .speed(0.1)
                    .range(0.01..=f32::MAX),
            )
            .changed();
        let y_changed = ui
            .add(
                egui::DragValue::new(&mut scale.y)
                    .speed(0.1)
                    .range(0.01..=f32::MAX),
            )
            .changed();

        if x_changed || y_changed {
            let mut new_transform = *transform;
            new_transform.scale = scale;
            update_entity_transform(world, entity, new_transform);
        }
    });

    // Reset transform button
    if ui.button(tr!("reset_transform")).clicked() {
        let reset_transform = Transform::from_xyz(pos.x, pos.y, pos.z);
        update_entity_transform(world, entity, reset_transform);
    }
}

// Helper functions to update colliders
fn update_rectangle_collider(world: &mut World, entity: Entity, width: f32, height: f32) {
    let mut commands = world.commands();
    commands
        .entity(entity)
        .insert(Collider::rectangle(width, height));

    // Update edit points to sync with shape editing
    sync_edit_points(world, entity);
}

fn update_circle_collider(world: &mut World, entity: Entity, radius: f32) {
    let mut commands = world.commands();
    commands.entity(entity).insert(Collider::circle(radius));

    // Update edit points to sync with shape editing
    sync_edit_points(world, entity);
}

fn update_capsule_collider(
    world: &mut World,
    entity: Entity,
    radius: f32,
    half_height: f32,
    rotation: f32,
) {
    let mut commands = world.commands();
    let relative_start = Vec2::new(0.0, -half_height);
    let relative_end = Vec2::new(0.0, half_height);

    let collider = Collider::capsule_endpoints(radius, relative_start, relative_end);
    commands.entity(entity).insert(collider);

    // Also update the rotation
    if let Some(mut transform) = world.get_mut::<Transform>(entity) {
        transform.rotation = Quat::from_rotation_z(rotation);
    }

    // Update edit points to sync with shape editing
    sync_edit_points(world, entity);
}

fn update_triangle_collider(
    world: &mut World,
    entity: Entity,
    vertices: &[Vec2; 3],
    _transform: &Transform,
) {
    let mut commands = world.commands();

    // Debug info: log vertex positions before creating collider
    info!(
        "更新三角形碰撞体 - 顶点位置: A=({:.1}, {:.1}), B=({:.1}, {:.1}), C=({:.1}, {:.1})",
        vertices[0].x, vertices[0].y, vertices[1].x, vertices[1].y, vertices[2].x, vertices[2].y
    );

    // Use triangle_unchecked to avoid automatic vertex reordering
    // Always maintain the original vertex order to prevent switching
    commands.entity(entity).insert(Collider::triangle_unchecked(
        avian2d::math::Vector::new(vertices[0].x, vertices[0].y),
        avian2d::math::Vector::new(vertices[1].x, vertices[1].y),
        avian2d::math::Vector::new(vertices[2].x, vertices[2].y),
    ));
    info!("保持顶点顺序: A→B→C");

    // Update edit points to sync with shape editing
    sync_edit_points(world, entity);
}

fn create_preset_polygon(
    world: &mut World,
    entity: Entity,
    sides: usize,
    radius: f32,
    _transform: &Transform,
) {
    let mut commands = world.commands();

    // Generate regular polygon vertices
    let vertices: Vec<Vec2> = (0..sides)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (sides as f32);
            Vec2::new(radius * angle.cos(), radius * angle.sin())
        })
        .collect();

    // Convert to avian2d vectors
    let avian_vertices: Vec<avian2d::math::Vector> = vertices
        .iter()
        .map(|v| avian2d::math::Vector::new(v.x, v.y))
        .collect();

    if let Some(collider) = Collider::convex_hull(avian_vertices) {
        commands.entity(entity).insert(collider);
        sync_edit_points(world, entity);
    }
}

fn update_entity_transform(world: &mut World, entity: Entity, transform: Transform) {
    let mut commands = world.commands();
    commands.entity(entity).insert(transform);
}

// Preset generation functions
fn preset_equilateral_triangle(center: Vec2, size: f32, rotation: Quat) -> [Vec2; 3] {
    let height = size * (1.73205080757 / 2.0); // SQRT_3 ≈ 1.73205080757
    let vertices = [
        Vec2::new(0.0, height * 2.0 / 3.0),    // Top
        Vec2::new(-size / 2.0, -height / 3.0), // Bottom left
        Vec2::new(size / 2.0, -height / 3.0),  // Bottom right
    ];

    vertices.map(|v| center + (rotation * v.extend(0.0)).truncate())
}

fn preset_right_triangle(center: Vec2, size: f32, rotation: Quat) -> [Vec2; 3] {
    let vertices = [
        Vec2::new(0.0, size / 2.0),          // Top
        Vec2::new(-size / 2.0, -size / 2.0), // Bottom left
        Vec2::new(size / 2.0, -size / 2.0),  // Bottom right
    ];

    vertices.map(|v| center + (rotation * v.extend(0.0)).truncate())
}

/// Update triangle side length while keeping the opposite vertex fixed
/// For side B-C (opposite vertex A), this keeps vertex A fixed and adjusts B and C positions
fn update_triangle_side_length(
    world: &mut World,
    entity: Entity,
    side_index: usize,
    new_length: f32,
    transform: &Transform,
) {
    if new_length <= 0.001 {
        return;
    }

    // Check for locked sides and handle constraint solving
    let lock_state = world.get_resource::<TriangleLockState>().unwrap();

    // If another side is locked, we need to adjust this side indirectly
    if let Some(locked_side_idx) = lock_state.locked_side {
        if locked_side_idx != side_index {
            // Another side is locked, so we need to solve the triangle constraint
            solve_triangle_with_locked_side(
                world,
                entity,
                locked_side_idx,
                side_index,
                new_length,
                transform,
            );
            return;
        }
    }

    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::Triangle(triangle) =
            collider.shape_scaled().as_typed_shape()
        {
            // Get local vertices (these are already in local space relative to entity origin)
            let vertices = [
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ];

            let mut new_vertices = vertices.clone();

            match side_index {
                // Side A-B (opposite vertex C) - keep vertex C fixed
                0 => {
                    let fixed_vertex = vertices[2]; // C is fixed in local space

                    // Current vectors from C to A and C to B in local space
                    let vec_ca = vertices[0] - fixed_vertex;
                    let vec_cb = vertices[1] - fixed_vertex;

                    // Current length of A-B
                    let current_ab = vertices[0].distance(vertices[1]);

                    if current_ab > 0.0 {
                        let dir_ca = vec_ca.normalize_or_zero();
                        let dir_cb = vec_cb.normalize_or_zero();

                        let ca_len = vec_ca.length();
                        let cb_len = vec_cb.length();
                        let angle_c = dir_ca.angle_to(dir_cb);
                        let cos_c = angle_c.cos();

                        let denominator =
                            ca_len * ca_len + cb_len * cb_len - 2.0 * ca_len * cb_len * cos_c;

                        if denominator > 0.0 {
                            let k_squared = new_length * new_length / denominator;
                            let k = k_squared.sqrt();

                            // Calculate new positions in local space
                            new_vertices[0] = fixed_vertex + dir_ca * (ca_len * k);
                            new_vertices[1] = fixed_vertex + dir_cb * (cb_len * k);
                        }
                    }
                }
                // Side B-C (opposite vertex A) - keep vertex A fixed
                1 => {
                    let fixed_vertex = vertices[0]; // A is fixed in local space

                    let vec_ab = vertices[1] - fixed_vertex;
                    let vec_ac = vertices[2] - fixed_vertex;

                    let current_bc = vertices[1].distance(vertices[2]);

                    if current_bc > 0.0 {
                        let dir_ab = vec_ab.normalize_or_zero();
                        let dir_ac = vec_ac.normalize_or_zero();

                        let ab_len = vec_ab.length();
                        let ac_len = vec_ac.length();
                        let angle_a = dir_ab.angle_to(dir_ac);
                        let cos_a = angle_a.cos();

                        let denominator =
                            ab_len * ab_len + ac_len * ac_len - 2.0 * ab_len * ac_len * cos_a;

                        if denominator > 0.0 {
                            let k_squared = new_length * new_length / denominator;
                            let k = k_squared.sqrt();

                            new_vertices[1] = fixed_vertex + dir_ab * (ab_len * k);
                            new_vertices[2] = fixed_vertex + dir_ac * (ac_len * k);
                        }
                    }
                }
                // Side C-A (opposite vertex B) - keep vertex B fixed
                2 => {
                    let fixed_vertex = vertices[1]; // B is fixed in local space

                    let vec_bc = vertices[2] - fixed_vertex;
                    let vec_ba = vertices[0] - fixed_vertex;

                    let current_ca = vertices[2].distance(vertices[0]);

                    if current_ca > 0.0 {
                        let dir_bc = vec_bc.normalize_or_zero();
                        let dir_ba = vec_ba.normalize_or_zero();

                        let bc_len = vec_bc.length();
                        let ba_len = vec_ba.length();
                        let angle_b = dir_bc.angle_to(dir_ba);
                        let cos_b = angle_b.cos();

                        let denominator =
                            bc_len * bc_len + ba_len * ba_len - 2.0 * bc_len * ba_len * cos_b;

                        if denominator > 0.0 {
                            let k_squared = new_length * new_length / denominator;
                            let k = k_squared.sqrt();

                            new_vertices[2] = fixed_vertex + dir_bc * (bc_len * k);
                            new_vertices[0] = fixed_vertex + dir_ba * (ba_len * k);
                        }
                    }
                }
                _ => return,
            }

            // Pass the local vertices directly to update_triangle_collider
            update_triangle_collider(world, entity, &new_vertices, transform);
        }
    }
}

/// Update triangle angle with lock support
fn update_triangle_angle_new(
    world: &mut World,
    entity: Entity,
    vertex_index: usize,
    new_angle_degrees: f32,
    transform: &Transform,
) {
    // Clamp angle to valid range (2.5 to 175 degrees)
    let new_angle_degrees = new_angle_degrees.clamp(2.5, 175.0);
    let new_angle_radians = new_angle_degrees.to_radians();

    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::Triangle(triangle) =
            collider.shape_scaled().as_typed_shape()
        {
            // Get local vertices (these are already in local space relative to entity origin)
            let vertices = [
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ];

            let mut new_vertices = vertices.clone();

            // Helper function to find intersection of two lines
            let line_intersection = |p1: Vec2, d1: Vec2, p2: Vec2, d2: Vec2| -> Option<Vec2> {
                let det = d1.x * d2.y - d1.y * d2.x;
                if det.abs() < 1e-10 {
                    return None; // Lines are parallel
                }
                let t = ((p2.x - p1.x) * d2.y - (p2.y - p1.y) * d2.x) / det;
                Some(p1 + d1 * t)
            };

            // Fix the specified vertex position and adjust the other two vertices
            // based on the new angle at that vertex, using the opposite side as reference
            match vertex_index {
                // Angle at vertex A (between AB and AC), opposite side is BC
                0 => {
                    let fixed_vertex = vertices[0]; // A is fixed
                    let b = vertices[1];
                    let c = vertices[2];

                    // Get vectors from fixed vertex to the other two vertices
                    let ab = b - fixed_vertex;
                    let ac = c - fixed_vertex;

                    // Calculate lengths of AB and AC
                    let ab_len = ab.length();
                    let ac_len = ac.length();

                    if ab_len <= 0.001 || ac_len <= 0.001 {
                        return;
                    }

                    // Calculate direction vectors
                    let ab_dir = ab.normalize();
                    let ac_dir = ac.normalize();

                    // Calculate the angle bisector
                    // For a correct angle bisector, we need to ensure it points in the "interior" of the angle
                    let angle_bisector = (ab_dir + ac_dir).normalize();

                    // Calculate perpendicular to the angle bisector
                    // We need to be careful about the direction to ensure correct angle orientation
                    let perpendicular = Vec2::new(-angle_bisector.y, angle_bisector.x);

                    // Create new direction vectors at the target angle
                    // These should be symmetric around the angle bisector
                    let half_angle = new_angle_radians / 2.0;
                    let new_ab_dir =
                        angle_bisector * half_angle.cos() - perpendicular * half_angle.sin();
                    let new_ac_dir =
                        angle_bisector * half_angle.cos() + perpendicular * half_angle.sin();

                    // Calculate the opposite side (BC) line direction
                    let bc_start = b;
                    let bc_end = c;
                    let bc_dir = (bc_end - bc_start).normalize_or_zero();

                    // Extend the BC line significantly in both directions
                    let line_extension = 1000.0;
                    let extended_bc_start = bc_start - bc_dir * line_extension;
                    let extended_bc_end = bc_end + bc_dir * line_extension;
                    let extended_bc_dir = (extended_bc_end - extended_bc_start).normalize();

                    // Find intersection points of new AB and AC rays with the extended BC line
                    if let Some(intersection1) = line_intersection(
                        fixed_vertex,
                        new_ab_dir,
                        extended_bc_start,
                        extended_bc_dir,
                    ) {
                        if let Some(intersection2) = line_intersection(
                            fixed_vertex,
                            new_ac_dir,
                            extended_bc_start,
                            extended_bc_dir,
                        ) {
                            // Determine which intersection point corresponds to B and which to C
                            // by checking which one is closer to the original B and C positions
                            let dist1_to_b = intersection1.distance(b);
                            let dist1_to_c = intersection1.distance(c);
                            let dist2_to_b = intersection2.distance(b);
                            let dist2_to_c = intersection2.distance(c);

                            let mut new_b = intersection1;
                            let mut new_c = intersection2;

                            // If intersection1 is closer to C and intersection2 is closer to B,
                            // we need to swap them to maintain vertex identity
                            if dist1_to_c < dist1_to_b && dist2_to_b < dist2_to_c {
                                new_b = intersection2;
                                new_c = intersection1;
                            }

                            // Check if the new points are valid (not too far)
                            let bc_original_length = bc_start.distance(bc_end);
                            let new_bc_length = new_b.distance(new_c);

                            // Only update if the new triangle is reasonable
                            if new_bc_length > 0.001 && new_bc_length < bc_original_length * 100.0 {
                                new_vertices[1] = new_b;
                                new_vertices[2] = new_c;
                            }
                        }
                    }
                }
                // Angle at vertex B (between BA and BC), opposite side is AC
                1 => {
                    let fixed_vertex = vertices[1]; // B is fixed
                    let a = vertices[0];
                    let c = vertices[2];

                    // Get vectors from fixed vertex to the other two vertices
                    let ba = a - fixed_vertex;
                    let bc = c - fixed_vertex;

                    // Calculate lengths of BA and BC
                    let ba_len = ba.length();
                    let bc_len = bc.length();

                    if ba_len <= 0.001 || bc_len <= 0.001 {
                        return;
                    }

                    // Calculate direction vectors
                    let ba_dir = ba.normalize();
                    let bc_dir = bc.normalize();

                    // Calculate the angle bisector
                    let angle_bisector = (ba_dir + bc_dir).normalize();

                    // Calculate perpendicular to the angle bisector
                    let perpendicular = Vec2::new(-angle_bisector.y, angle_bisector.x);

                    // Create new direction vectors at the target angle
                    let half_angle = new_angle_radians / 2.0;
                    let new_ba_dir =
                        angle_bisector * half_angle.cos() - perpendicular * half_angle.sin();
                    let new_bc_dir =
                        angle_bisector * half_angle.cos() + perpendicular * half_angle.sin();

                    // Calculate the opposite side (AC) line direction
                    let ac_start = a;
                    let ac_end = c;
                    let ac_dir = (ac_end - ac_start).normalize_or_zero();

                    // Extend the AC line significantly in both directions
                    let line_extension = 1000.0;
                    let extended_ac_start = ac_start - ac_dir * line_extension;
                    let extended_ac_end = ac_end + ac_dir * line_extension;
                    let extended_ac_dir = (extended_ac_end - extended_ac_start).normalize();

                    // Find intersection points of new BA and BC rays with the extended AC line
                    if let Some(intersection1) = line_intersection(
                        fixed_vertex,
                        new_ba_dir,
                        extended_ac_start,
                        extended_ac_dir,
                    ) {
                        if let Some(intersection2) = line_intersection(
                            fixed_vertex,
                            new_bc_dir,
                            extended_ac_start,
                            extended_ac_dir,
                        ) {
                            // Determine which intersection point corresponds to A and which to C
                            let dist1_to_a = intersection1.distance(a);
                            let dist1_to_c = intersection1.distance(c);
                            let dist2_to_a = intersection2.distance(a);
                            let dist2_to_c = intersection2.distance(c);

                            let mut new_a = intersection1;
                            let mut new_c = intersection2;

                            // If intersection1 is closer to C and intersection2 is closer to A,
                            // we need to swap them to maintain vertex identity
                            if dist1_to_c < dist1_to_a && dist2_to_a < dist2_to_c {
                                new_a = intersection2;
                                new_c = intersection1;
                            }

                            // Check if the new points are valid
                            let ac_original_length = ac_start.distance(ac_end);
                            let new_ac_length = new_a.distance(new_c);

                            // Only update if the new triangle is reasonable
                            if new_ac_length > 0.001 && new_ac_length < ac_original_length * 100.0 {
                                new_vertices[0] = new_a;
                                new_vertices[2] = new_c;
                            }
                        }
                    }
                }
                // Angle at vertex C (between CA and CB), opposite side is AB
                2 => {
                    let fixed_vertex = vertices[2]; // C is fixed
                    let a = vertices[0];
                    let b = vertices[1];

                    // Get vectors from fixed vertex to the other two vertices
                    let ca = a - fixed_vertex;
                    let cb = b - fixed_vertex;

                    // Calculate lengths of CA and CB
                    let ca_len = ca.length();
                    let cb_len = cb.length();

                    if ca_len <= 0.001 || cb_len <= 0.001 {
                        return;
                    }

                    // Calculate direction vectors
                    let ca_dir = ca.normalize();
                    let cb_dir = cb.normalize();

                    // Calculate the angle bisector
                    let angle_bisector = (ca_dir + cb_dir).normalize();

                    // Calculate perpendicular to the angle bisector
                    let perpendicular = Vec2::new(-angle_bisector.y, angle_bisector.x);

                    // Create new direction vectors at the target angle
                    let half_angle = new_angle_radians / 2.0;
                    let new_ca_dir =
                        angle_bisector * half_angle.cos() - perpendicular * half_angle.sin();
                    let new_cb_dir =
                        angle_bisector * half_angle.cos() + perpendicular * half_angle.sin();

                    // Calculate the opposite side (AB) line direction
                    let ab_start = a;
                    let ab_end = b;
                    let ab_dir = (ab_end - ab_start).normalize_or_zero();

                    // Extend the AB line significantly in both directions
                    let line_extension = 1000.0;
                    let extended_ab_start = ab_start - ab_dir * line_extension;
                    let extended_ab_end = ab_end + ab_dir * line_extension;
                    let extended_ab_dir = (extended_ab_end - extended_ab_start).normalize();

                    // Find intersection points of new CA and CB rays with the extended AB line
                    if let Some(intersection1) = line_intersection(
                        fixed_vertex,
                        new_ca_dir,
                        extended_ab_start,
                        extended_ab_dir,
                    ) {
                        if let Some(intersection2) = line_intersection(
                            fixed_vertex,
                            new_cb_dir,
                            extended_ab_start,
                            extended_ab_dir,
                        ) {
                            // Determine which intersection point corresponds to A and which to B
                            let dist1_to_a = intersection1.distance(a);
                            let dist1_to_b = intersection1.distance(b);
                            let dist2_to_a = intersection2.distance(a);
                            let dist2_to_b = intersection2.distance(b);

                            let mut new_a = intersection1;
                            let mut new_b = intersection2;

                            // If intersection1 is closer to B and intersection2 is closer to A,
                            // we need to swap them to maintain vertex identity
                            if dist1_to_b < dist1_to_a && dist2_to_a < dist2_to_b {
                                new_a = intersection2;
                                new_b = intersection1;
                            }

                            // Check if the new points are valid
                            let ab_original_length = ab_start.distance(ab_end);
                            let new_ab_length = new_a.distance(new_b);

                            // Only update if the new triangle is reasonable
                            if new_ab_length > 0.001 && new_ab_length < ab_original_length * 100.0 {
                                new_vertices[0] = new_a;
                                new_vertices[1] = new_b;
                            }
                        }
                    }
                }
                _ => return, // Invalid vertex index
            }

            // Update the triangle with new vertices
            update_triangle_collider(world, entity, &new_vertices, transform);
        }
    }
}

/// Solve triangle when one side is locked and another is being modified
/// locked_side_idx: the index of the locked side (0=AB, 1=BC, 2=CA)
/// modified_side_idx: the index of the side being modified
/// new_length: the desired new length for the modified side
fn solve_triangle_with_locked_side(
    world: &mut World,
    entity: Entity,
    locked_side_idx: usize,
    modified_side_idx: usize,
    new_length: f32,
    transform: &Transform,
) {
    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::Triangle(triangle) =
            collider.shape_scaled().as_typed_shape()
        {
            // Get current vertices
            let vertices = [
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ];

            // Calculate current side lengths
            let sides = [
                vertices[0].distance(vertices[1]), // AB
                vertices[1].distance(vertices[2]), // BC
                vertices[2].distance(vertices[0]), // CA
            ];

            let locked_length = sides[locked_side_idx];

            // Find the third side index
            let third_side_idx = 3 - locked_side_idx - modified_side_idx;
            let third_length = sides[third_side_idx];

            // Check triangle inequality
            if new_length + locked_length <= third_length
                || new_length + third_length <= locked_length
                || locked_length + third_length <= new_length
            {
                return; // Invalid triangle
            }

            // Use law of cosines to find angles and then adjust the third side
            // For simplicity, we'll keep the angles as close as possible while satisfying the side constraints

            let mut new_vertices = vertices.clone();

            match modified_side_idx {
                // Side AB being modified
                0 => {
                    // Scale AB to new length, keeping angles similar
                    let scale_factor = new_length / sides[0];
                    let center = (vertices[0] + vertices[1]) * 0.5;
                    new_vertices[0] = center + (vertices[0] - center) * scale_factor;
                    new_vertices[1] = center + (vertices[1] - center) * scale_factor;
                }
                // Side BC being modified
                1 => {
                    let scale_factor = new_length / sides[1];
                    let center = (vertices[1] + vertices[2]) * 0.5;
                    new_vertices[1] = center + (vertices[1] - center) * scale_factor;
                    new_vertices[2] = center + (vertices[2] - center) * scale_factor;
                }
                // Side CA being modified
                2 => {
                    let scale_factor = new_length / sides[2];
                    let center = (vertices[2] + vertices[0]) * 0.5;
                    new_vertices[2] = center + (vertices[2] - center) * scale_factor;
                    new_vertices[0] = center + (vertices[0] - center) * scale_factor;
                }
                _ => return,
            }

            // Ensure the locked side maintains its length
            match locked_side_idx {
                0 => {
                    // AB should maintain locked length
                    adjust_side_to_length(&mut new_vertices, 0, 1, locked_length);
                }
                1 => {
                    // BC should maintain locked length
                    adjust_side_to_length(&mut new_vertices, 1, 2, locked_length);
                }
                2 => {
                    // CA should maintain locked length
                    adjust_side_to_length(&mut new_vertices, 2, 0, locked_length);
                }
                _ => return,
            }

            // Update the triangle with new vertices
            update_triangle_collider(world, entity, &new_vertices, transform);
        }
    }
}

/// Adjust the distance between two vertices to a specific length
fn adjust_side_to_length(vertices: &mut [Vec2; 3], idx1: usize, idx2: usize, target_length: f32) {
    let current_length = vertices[idx1].distance(vertices[idx2]);
    if current_length > 0.0 {
        let scale_factor = target_length / current_length;
        let center = (vertices[idx1] + vertices[idx2]) * 0.5;
        vertices[idx1] = center + (vertices[idx1] - center) * scale_factor;
        vertices[idx2] = center + (vertices[idx2] - center) * scale_factor;
    }
}

/// Calculate triangle angles in degrees for display
fn calculate_triangle_angles(vertices: &[Vec2; 3]) -> [f32; 3] {
    let side_a = vertices[0].distance(vertices[1]);
    let side_b = vertices[1].distance(vertices[2]);
    let side_c = vertices[2].distance(vertices[0]);

    // Calculate angles using law of cosines
    let angle_a =
        ((side_b * side_b + side_c * side_c - side_a * side_a) / (2.0 * side_b * side_c)).acos();
    let angle_b =
        ((side_a * side_a + side_c * side_c - side_b * side_b) / (2.0 * side_a * side_c)).acos();
    let angle_c =
        ((side_a * side_a + side_b * side_b - side_c * side_c) / (2.0 * side_a * side_b)).acos();

    [
        angle_a.to_degrees(),
        angle_b.to_degrees(),
        angle_c.to_degrees(),
    ]
}

/// Calculate triangle circumradius (R = abc / 4K where K is area)
fn calculate_triangle_circumradius(vertices: &[Vec2; 3]) -> f32 {
    let side_a = vertices[0].distance(vertices[1]);
    let side_b = vertices[1].distance(vertices[2]);
    let side_c = vertices[2].distance(vertices[0]);

    // Calculate area using cross product
    let ab = vertices[1] - vertices[0];
    let ac = vertices[2] - vertices[0];
    let area = 0.5 * (ab.x * ac.y - ab.y * ac.x).abs();

    if area > 0.0 {
        (side_a * side_b * side_c) / (4.0 * area)
    } else {
        0.0
    }
}

/// Scale triangle from circumradius while preserving shape
fn scale_triangle_from_circumradius(
    world: &mut World,
    entity: Entity,
    new_radius: f32,
    transform: &Transform,
) {
    if new_radius <= 0.0 {
        return;
    }

    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::Triangle(triangle) =
            collider.shape_scaled().as_typed_shape()
        {
            // Get local vertices
            let vertices = [
                Vec2::new(triangle.a.x, triangle.a.y),
                Vec2::new(triangle.b.x, triangle.b.y),
                Vec2::new(triangle.c.x, triangle.c.y),
            ];

            // Calculate current circumradius
            let current_radius = calculate_triangle_circumradius(&vertices);

            if current_radius > 0.0 {
                let scale_factor = new_radius / current_radius;

                // Calculate centroid as scaling center
                let centroid = Vec2::new(
                    (vertices[0].x + vertices[1].x + vertices[2].x) / 3.0,
                    (vertices[0].y + vertices[1].y + vertices[2].y) / 3.0,
                );

                // Scale all vertices from centroid
                let new_vertices: [Vec2; 3] =
                    vertices.map(|v| centroid + (v - centroid) * scale_factor);

                update_triangle_collider(world, entity, &new_vertices, transform);
            }
        }
    }
}

/// Scale polygon uniformly from radius
fn scale_polygon_from_radius(
    world: &mut World,
    entity: Entity,
    new_radius: f32,
    transform: &Transform,
) {
    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::ConvexPolygon(poly) =
            collider.shape_scaled().as_typed_shape()
        {
            let vertices: Vec<Vec2> = poly
                .points()
                .iter()
                .map(|point| Vec2::new(point.x, point.y))
                .collect();

            let center = vertices.iter().sum::<Vec2>() / vertices.len() as f32;
            let avg_radius =
                vertices.iter().map(|v| v.distance(center)).sum::<f32>() / vertices.len() as f32;

            if avg_radius > 0.0 {
                let scale_factor = new_radius / avg_radius;
                let scaled_vertices: Vec<Vec2> = vertices
                    .iter()
                    .map(|v| center + (*v - center) * scale_factor)
                    .collect();
                update_polygon_from_vertices(world, entity, &scaled_vertices, transform);
            }
        }
    }
}

/// Scale polygon uniformly from side length
fn scale_polygon_from_side_length(
    world: &mut World,
    entity: Entity,
    new_side_length: f32,
    transform: &Transform,
) {
    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::ConvexPolygon(poly) =
            collider.shape_scaled().as_typed_shape()
        {
            let vertices: Vec<Vec2> = poly
                .points()
                .iter()
                .map(|point| Vec2::new(point.x, point.y))
                .collect();

            // Calculate current average side length
            let mut side_lengths = Vec::new();
            for i in 0..vertices.len() {
                let next_i = (i + 1) % vertices.len();
                side_lengths.push(vertices[i].distance(vertices[next_i]));
            }
            let avg_side_length = side_lengths.iter().sum::<f32>() / side_lengths.len() as f32;

            if avg_side_length > 0.0 {
                let scale_factor = new_side_length / avg_side_length;
                let center = vertices.iter().sum::<Vec2>() / vertices.len() as f32;
                let scaled_vertices: Vec<Vec2> = vertices
                    .iter()
                    .map(|v| center + (*v - center) * scale_factor)
                    .collect();
                update_polygon_from_vertices(world, entity, &scaled_vertices, transform);
            }
        }
    }
}

/// Adjust individual polygon side length
fn adjust_polygon_side(
    world: &mut World,
    entity: Entity,
    side_index: usize,
    new_length: f32,
    transform: &Transform,
) {
    if let Ok((_, _, collider, _)) = world
        .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
        .get_mut(world, entity)
    {
        if let avian2d::parry::shape::TypedShape::ConvexPolygon(poly) =
            collider.shape_scaled().as_typed_shape()
        {
            let vertices: Vec<Vec2> = poly
                .points()
                .iter()
                .map(|point| Vec2::new(point.x, point.y))
                .collect();

            if side_index < vertices.len() {
                let next_index = (side_index + 1) % vertices.len();
                let current_length = vertices[side_index].distance(vertices[next_index]);

                if current_length > 0.0 {
                    // Calculate direction and scale factor
                    let direction =
                        (vertices[next_index] - vertices[side_index]).normalize_or_zero();
                    let _scale_factor = new_length / current_length;

                    // Move the next vertex to achieve the desired side length
                    let mut new_vertices = vertices.clone();
                    new_vertices[next_index] = vertices[side_index] + direction * new_length;

                    update_polygon_from_vertices(world, entity, &new_vertices, transform);
                }
            }
        }
    }
}

/// Update polygon from local vertices
fn update_polygon_from_vertices(
    world: &mut World,
    entity: Entity,
    vertices: &[Vec2],
    _transform: &Transform,
) {
    let mut commands = world.commands();

    let avian_vertices: Vec<avian2d::math::Vector> = vertices
        .iter()
        .map(|v| avian2d::math::Vector::new(v.x, v.y))
        .collect();

    if let Some(collider) = Collider::convex_hull(avian_vertices) {
        commands.entity(entity).insert(collider);
        sync_edit_points(world, entity);
    }
}

/// Update polygon from world vertices
fn update_polygon_from_world_vertices(
    world: &mut World,
    entity: Entity,
    world_vertices: &[Vec2],
    transform: &Transform,
) {
    let center = transform.translation.truncate();
    let rotation_inv = transform.rotation.inverse();

    let local_vertices: Vec<Vec2> = world_vertices
        .iter()
        .map(|world_vertex| {
            let offset = *world_vertex - center;
            (rotation_inv * offset.extend(0.0)).truncate()
        })
        .collect();

    update_polygon_from_vertices(world, entity, &local_vertices, transform);
}

/// Synchronize edit points with shape editing changes
/// This ensures that the visual edit points in Edit mode stay in sync with shape changes made in ShapeEdit mode
fn sync_edit_points(world: &mut World, entity: Entity) {
    // Extract component data first to avoid borrowing conflicts
    let (transform, collider, collider_type) = {
        if let Ok((_, transform, collider, collider_type)) = world
            .query::<(Entity, &Transform, &Collider, &collider_tools::ColliderType)>()
            .get(world, entity)
        {
            (transform.clone(), collider.clone(), *collider_type)
        } else {
            return;
        }
    };

    // Now update the edit state
    if let Some(mut edit_state) = world.get_resource_mut::<edit::ColliderEditState>() {
        visualization::generate_control_points(
            &mut edit_state,
            &transform,
            &collider,
            &collider_type,
        );
    }
}
