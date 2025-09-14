use avian2d::prelude::{LockedAxes, RigidBody};
use bevy::prelude::*;
use bevy_egui::egui::{self, Context};

use crate::{
    AnchorCreationState, AnchorPoint, ColliderEditState, ColliderType, CreationProperties,
    EditorSelection, GizmoMode, GizmoTransformable, JointCreationState, JointType, ToolMode,
    TransformGizmoSettings, joint_config::JointConfiguration, tr,
};

/// Event for duplicating an entity
#[derive(Event)]
pub struct DuplicateEntityEvent {
    pub original: Entity,
}

pub(super) fn ui(
    ctx: &mut Context,
    world: &mut World,
    current_mode: ToolMode,
    selected_entity: Option<Entity>,
) {
    egui::SidePanel::left("tool_panel")
        .default_width(250.0)
        .show(ctx, |ui| {
            // Tool Mode Selection
            ui.heading(tr!("tool_mode"));

            ui.horizontal_wrapped(|ui| {
                if ui
                    .selectable_label(current_mode == ToolMode::Select, tr!("mode_select"))
                    .clicked()
                {
                    if let Some(mut next_state) = world.get_resource_mut::<NextState<ToolMode>>() {
                        next_state.set(ToolMode::Select);
                    }
                }
                if ui
                    .selectable_label(current_mode == ToolMode::Create, tr!("mode_create"))
                    .clicked()
                {
                    if let Some(mut next_state) = world.get_resource_mut::<NextState<ToolMode>>() {
                        next_state.set(ToolMode::Create);
                    }
                }
                if ui
                    .selectable_label(current_mode == ToolMode::Edit, tr!("mode_edit"))
                    .clicked()
                {
                    if let Some(mut next_state) = world.get_resource_mut::<NextState<ToolMode>>() {
                        next_state.set(ToolMode::Edit);
                    }
                }
                if ui
                    .selectable_label(current_mode == ToolMode::Anchor, tr!("mode_anchor"))
                    .clicked()
                {
                    if let Some(mut next_state) = world.get_resource_mut::<NextState<ToolMode>>() {
                        next_state.set(ToolMode::Anchor);
                    }
                }
                if ui
                    .selectable_label(current_mode == ToolMode::Joint, tr!("mode_joint"))
                    .clicked()
                {
                    if let Some(mut next_state) = world.get_resource_mut::<NextState<ToolMode>>() {
                        next_state.set(ToolMode::Joint);
                    }
                }
            });

            // Mode-specific controls
            ui.separator();
            match current_mode {
                ToolMode::Create => {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        world.resource_scope::<CreationProperties, ()>(|world, mut properties| {
                            let mut changed = false;

                            // === 基础配置 ===
                            changed = create_basic_ui(ui, &mut properties) || changed;

                            ui.separator();

                            // === 预设配置 ===
                            ui.collapsing(tr!("presets_config"), |ui| {
                                changed = create_presets_ui(ui, &mut properties) || changed;
                            });

                            ui.separator();

                            // === 质量属性 ===
                            ui.collapsing(tr!("mass_properties"), |ui| {
                                changed = create_mass_properties_ui(ui, &mut properties) || changed;
                            });

                            ui.separator();

                            // === 材料属性 ===
                            ui.collapsing(tr!("material_properties"), |ui| {
                                changed =
                                    create_material_properties_ui(ui, &mut properties) || changed;
                            });

                            ui.separator();

                            // === 运动控制 ===
                            ui.collapsing(tr!("motion_control"), |ui| {
                                changed =
                                    create_motion_properties_ui(ui, &mut properties) || changed;
                            });

                            ui.separator();

                            // === 碰撞检测 ===
                            ui.collapsing(tr!("collision_detection"), |ui| {
                                changed =
                                    create_collision_properties_ui(ui, &mut properties, world)
                                        || changed;
                            });

                            ui.separator();

                            // === 性能优化 ===
                            ui.collapsing(tr!("performance_optimization"), |ui| {
                                changed = create_performance_properties_ui(ui, &mut properties)
                                    || changed;
                            });

                            ui.separator();

                            // === 高级物理 ===
                            ui.collapsing(tr!("advanced_physics"), |ui| {
                                changed =
                                    create_advanced_physics_ui(ui, &mut properties) || changed;
                            });

                            if changed {
                                ui.label(tr!("config_updated"));
                            }
                        });

                        // Instructions
                        re_ui::Help::new_without_title()
                            .markdown(tr!("creation_controls"))
                            .markdown(tr!("drawing_controls"))
                            .control(tr!("rectangle"), ("Left Click", "Drag"))
                            .control(tr!("circle"), ("Right Click", "Drag"))
                            .markdown(tr!("shape_configuration"))
                            .control(tr!("collider_type"), "Dropdown")
                            .control(tr!("create"), "Enter")
                            .markdown(tr!("advanced_shapes"))
                            .markdown("- **Capsule**: Define with two points")
                            .markdown("- **Polygon**: Define with multiple clicks")
                            .markdown("- **Triangle**: Define with three points")
                            .ui(ui);
                    });
                }
                ToolMode::Edit => {
                    ui.heading(tr!("collider_editor"));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(edit_state) = world.get_resource::<ColliderEditState>() {
                            // Editing Status
                            ui.label(format!(
                                "{}: {}",
                                tr!("control_points"),
                                edit_state.control_points.len()
                            ));

                            if let Some(dragging_idx) = edit_state.dragging_point {
                                ui.label(format!("{}: {}", tr!("dragging_point"), dragging_idx));
                            } else {
                                ui.label(tr!("no_point_selected"));
                            }

                            if let Some(entity) = edit_state.editing_entity {
                                ui.label(format!("{}: {:?}", tr!("editing_entity"), entity));
                            }

                            ui.separator();

                            // Instructions
                            re_ui::Help::new_without_title()
                                .markdown(tr!("edit_controls"))
                                .markdown(tr!("point_editing"))
                                .control(tr!("modify"), "Drag")
                                .control(tr!("select"), "Click")
                                .control(tr!("remove"), "Delete")
                                .markdown(tr!("history_management"))
                                .control(tr!("undo"), ("Ctrl +", "Z"))
                                .control(tr!("redo"), ("Ctrl +", "Y"))
                                .markdown(tr!("shape_operations"))
                                .control(tr!("add_vertex"), "Click")
                                .control(tr!("move"), "Drag")
                                .control(tr!("cancel"), "Escape")
                                .ui(ui);
                        } else {
                            ui.label(tr!("edit_state_unavailable"));
                        }
                    });
                }
                ToolMode::Anchor => {
                    ui.heading(tr!("anchor_tools"));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Extract anchor state first to avoid borrowing conflicts
                        let anchor_count = world.query::<&AnchorPoint>().iter(world).count();
                        let anchor_preview_mode = world
                            .get_resource::<AnchorCreationState>()
                            .map(|s| s.preview_mode)
                            .unwrap_or(false);
                        let anchor_shift_pressed = world
                            .get_resource::<AnchorCreationState>()
                            .map(|s| s.shift_pressed)
                            .unwrap_or(false);
                        let anchor_preview_pos = world
                            .get_resource::<AnchorCreationState>()
                            .and_then(|s| s.preview_position);
                        let anchor_selected_entity = world
                            .get_resource::<AnchorCreationState>()
                            .and_then(|s| s.selected_anchor);

                        if world.get_resource::<AnchorCreationState>().is_some() {
                            // Anchor Status
                            ui.label(format!("{}: {}", tr!("created_anchors"), anchor_count));

                            if let Some(pos) = anchor_preview_pos {
                                ui.label(format!(
                                    "{}: ({:.1}, {:.1})",
                                    tr!("preview_position"),
                                    pos.x,
                                    pos.y
                                ));
                            }

                            if let Some(entity) = anchor_selected_entity {
                                ui.label(format!("{}: {:?}", tr!("selected_anchor"), entity));
                            }

                            ui.label(format!("{}: {}", tr!("preview_mode"), anchor_preview_mode));
                            ui.label(format!(
                                "{}: {}",
                                tr!("shift_pressed"),
                                anchor_shift_pressed
                            ));

                            ui.separator();

                            // Quick Actions
                            ui.label(tr!("quick_actions"));
                            ui.horizontal(|ui| {
                                if ui.button(tr!("clear_all")).clicked() {
                                    // Collect all anchor entities to despawn
                                    let entities_to_despawn: Vec<Entity> = world
                                        .query::<(Entity, &AnchorPoint)>()
                                        .iter(world)
                                        .map(|(entity, _)| entity)
                                        .collect();

                                    // Despawn entities
                                    for entity in entities_to_despawn {
                                        if let Ok(entity_mut) = world.get_entity_mut(entity) {
                                            entity_mut.despawn();
                                        }
                                    }

                                    // Clear state
                                    if let Some(mut anchor_state) =
                                        world.get_resource_mut::<AnchorCreationState>()
                                    {
                                        anchor_state.selected_anchor = None;
                                        anchor_state.preview_position = None;
                                        anchor_state.preview_collider = None;
                                    }
                                }
                            });

                            ui.separator();

                            ui.separator();

                            re_ui::Help::new_without_title()
                                .markdown(tr!("anchor_controls"))
                                .markdown(tr!("anchor_creation"))
                                .control(tr!("create"), "Click")
                                .control(tr!("multiple"), ("Shift +", "Click"))
                                .control(tr!("remove"), "Delete")
                                .control(tr!("clear_all"), "Clear")
                                .markdown(tr!("positioning"))
                                .control(tr!("preview"), "Right Click")
                                .control(tr!("snap"), "Shift")
                                .control(tr!("precise"), "Ctrl")
                                .markdown(tr!("workflow"))
                                .control(tr!("switch_mode"), "Tab")
                                .control(tr!("connect"), "Drag")
                                .control(tr!("confirm"), "Enter")
                                .ui(ui);
                        } else {
                            ui.label(tr!("anchor_state_unavailable"));
                        }
                    });
                }
                ToolMode::Joint => {
                    ui.heading(tr!("joint_settings"));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(joint_state) =
                            world.get_resource::<JointCreationState>().cloned()
                        {
                            if let Some(mut joint_config) =
                                world.get_resource_mut::<JointConfiguration>()
                            {
                                let mut config_changed = false;

                                // Joint Type Selection
                                ui.label(tr!("select_joint_type"));
                                ui.horizontal_wrapped(|ui| {
                                    config_changed = ui
                                        .selectable_value(
                                            &mut joint_config.joint_type,
                                            JointType::Distance,
                                            tr!("distance_joint"),
                                        )
                                        .changed()
                                        || config_changed;
                                    config_changed = ui
                                        .selectable_value(
                                            &mut joint_config.joint_type,
                                            JointType::Revolute,
                                            tr!("revolute_joint"),
                                        )
                                        .changed()
                                        || config_changed;
                                    config_changed = ui
                                        .selectable_value(
                                            &mut joint_config.joint_type,
                                            JointType::Prismatic,
                                            tr!("prismatic_joint"),
                                        )
                                        .changed()
                                        || config_changed;
                                    config_changed = ui
                                        .selectable_value(
                                            &mut joint_config.joint_type,
                                            JointType::Fixed,
                                            tr!("fixed_joint"),
                                        )
                                        .changed()
                                        || config_changed;
                                });

                                ui.separator();

                                // Quick Presets
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button(tr!("rigid_preset")).clicked() {
                                        joint_config.rigid_connection();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("spring_preset")).clicked() {
                                        joint_config.spring_connection();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("sliding_preset")).clicked() {
                                        joint_config.sliding_door();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("hinge_preset")).clicked() {
                                        joint_config.hinge();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("breakable_preset")).clicked() {
                                        joint_config.breakable_connection();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("motorized_preset")).clicked() {
                                        joint_config.motorized_hinge();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("suspension_preset")).clicked() {
                                        joint_config.suspension();
                                        config_changed = true;
                                    }
                                    if ui.button(tr!("rope_preset")).clicked() {
                                        joint_config.rope_constraint();
                                        config_changed = true;
                                    }
                                });

                                ui.separator();

                                // Common Properties
                                // Damping
                                ui.vertical(|ui| {
                                    ui.label(tr!("linear_damping"));
                                    config_changed = ui
                                        .add(
                                            egui::Slider::new(
                                                &mut joint_config.common.damping_linear,
                                                0.0..=5.0,
                                            )
                                            .text(tr!("linear")),
                                        )
                                        .changed()
                                        || config_changed;
                                });

                                ui.vertical(|ui| {
                                    ui.label(tr!("angular_damping"));
                                    config_changed = ui
                                        .add(
                                            egui::Slider::new(
                                                &mut joint_config.common.damping_angular,
                                                0.0..=5.0,
                                            )
                                            .text(tr!("angular")),
                                        )
                                        .changed()
                                        || config_changed;
                                });

                                // Collision Disable
                                config_changed = ui
                                    .checkbox(
                                        &mut joint_config.common.disable_collision,
                                        tr!("disable_collision"),
                                    )
                                    .changed()
                                    || config_changed;

                                ui.separator();

                                // Type-Specific Properties
                                match joint_config.joint_type {
                                    JointType::Fixed => {
                                        ui.label(tr!("fixed_joint_properties"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("point_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.fixed.point_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("point")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("angle_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.fixed.angle_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("angle")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                    }
                                    JointType::Distance => {
                                        ui.label(tr!("distance_joint_properties"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.distance.compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("compliance")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("rest_length"));
                                            config_changed = ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut joint_config.distance.rest_length,
                                                    )
                                                    .speed(1.0)
                                                    .range(0.0..=1000.0),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("distance_limits"));
                                            ui.horizontal_wrapped(|ui| {
                                                let mut min_enabled =
                                                    joint_config.distance.min_distance.is_some();
                                                let mut max_enabled =
                                                    joint_config.distance.max_distance.is_some();

                                                if ui
                                                    .checkbox(&mut min_enabled, tr!("min"))
                                                    .changed()
                                                {
                                                    if min_enabled {
                                                        joint_config.distance.min_distance =
                                                            Some(0.0);
                                                    } else {
                                                        joint_config.distance.min_distance = None;
                                                    }
                                                    config_changed = true;
                                                }
                                                if ui
                                                    .checkbox(&mut max_enabled, tr!("max"))
                                                    .changed()
                                                {
                                                    if max_enabled {
                                                        joint_config.distance.max_distance =
                                                            Some(100.0);
                                                    } else {
                                                        joint_config.distance.max_distance = None;
                                                    }
                                                    config_changed = true;
                                                }
                                            });

                                            if let Some(ref mut min_dist) =
                                                joint_config.distance.min_distance
                                            {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(min_dist)
                                                            .speed(1.0)
                                                            .range(0.0..=1000.0),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                            }
                                            if let Some(ref mut max_dist) =
                                                joint_config.distance.max_distance
                                            {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(max_dist)
                                                            .speed(1.0)
                                                            .range(0.0..=1000.0),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                            }
                                        });
                                    }
                                    JointType::Prismatic => {
                                        ui.label(tr!("prismatic_joint_properties"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("free_axis"));
                                            ui.horizontal_wrapped(|ui| {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(
                                                            &mut joint_config.prismatic.free_axis.x,
                                                        )
                                                        .speed(0.1),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                                ui.add(
                                                    egui::DragValue::new(
                                                        &mut joint_config.prismatic.free_axis.y,
                                                    )
                                                    .speed(0.1),
                                                );
                                            });
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("axis_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.prismatic.axis_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("axis")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("limit_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config
                                                            .prismatic
                                                            .limit_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("limit")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("distance_limits"));
                                            ui.horizontal_wrapped(|ui| {
                                                let mut min_enabled =
                                                    joint_config.prismatic.min_distance.is_some();
                                                let mut max_enabled =
                                                    joint_config.prismatic.max_distance.is_some();

                                                if ui
                                                    .checkbox(&mut min_enabled, tr!("min"))
                                                    .changed()
                                                {
                                                    if min_enabled {
                                                        joint_config.prismatic.min_distance =
                                                            Some(0.0);
                                                    } else {
                                                        joint_config.prismatic.min_distance = None;
                                                    }
                                                    config_changed = true;
                                                }
                                                if ui
                                                    .checkbox(&mut max_enabled, tr!("max"))
                                                    .changed()
                                                {
                                                    if max_enabled {
                                                        joint_config.prismatic.max_distance =
                                                            Some(200.0);
                                                    } else {
                                                        joint_config.prismatic.max_distance = None;
                                                    }
                                                    config_changed = true;
                                                }
                                            });

                                            if let Some(ref mut min_dist) =
                                                joint_config.prismatic.min_distance
                                            {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(min_dist)
                                                            .speed(1.0)
                                                            .range(-1000.0..=1000.0),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                            }
                                            if let Some(ref mut max_dist) =
                                                joint_config.prismatic.max_distance
                                            {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(max_dist)
                                                            .speed(1.0)
                                                            .range(-1000.0..=1000.0),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                            }
                                        });
                                    }
                                    JointType::Revolute => {
                                        ui.label(tr!("revolute_joint_properties"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("point_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.revolute.point_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("point")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("point_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.revolute.point_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("point")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("limit_compliance"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.revolute.limit_compliance,
                                                        0.0..=0.1,
                                                    )
                                                    .text(tr!("limit")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("angle_limits"));
                                            ui.horizontal_wrapped(|ui| {
                                                let mut min_enabled =
                                                    joint_config.revolute.min_angle.is_some();
                                                let mut max_enabled =
                                                    joint_config.revolute.max_angle.is_some();

                                                if ui
                                                    .checkbox(&mut min_enabled, tr!("min"))
                                                    .changed()
                                                {
                                                    if min_enabled {
                                                        joint_config.revolute.min_angle =
                                                            Some(-std::f32::consts::PI / 4.0);
                                                    } else {
                                                        joint_config.revolute.min_angle = None;
                                                    }
                                                    config_changed = true;
                                                }
                                                if ui
                                                    .checkbox(&mut max_enabled, tr!("max"))
                                                    .changed()
                                                {
                                                    if max_enabled {
                                                        joint_config.revolute.max_angle =
                                                            Some(std::f32::consts::PI / 4.0);
                                                    } else {
                                                        joint_config.revolute.max_angle = None;
                                                    }
                                                    config_changed = true;
                                                }
                                            });

                                            if let Some(ref mut min_angle) =
                                                joint_config.revolute.min_angle
                                            {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(min_angle)
                                                            .speed(0.1)
                                                            .range(
                                                                -std::f32::consts::PI
                                                                    ..=std::f32::consts::PI,
                                                            ),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                            }
                                            if let Some(ref mut max_angle) =
                                                joint_config.revolute.max_angle
                                            {
                                                config_changed = ui
                                                    .add(
                                                        egui::DragValue::new(max_angle)
                                                            .speed(0.1)
                                                            .range(
                                                                -std::f32::consts::PI
                                                                    ..=std::f32::consts::PI,
                                                            ),
                                                    )
                                                    .changed()
                                                    || config_changed;
                                            }
                                        });
                                    }
                                }

                                // Advanced Properties
                                ui.collapsing(tr!("advanced_properties"), |ui| {
                                    // Breakable Joint Settings
                                    ui.collapsing(tr!("breakable_joint"), |ui| {
                                        ui.label(tr!("breakable_settings"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("breakable"));
                                            config_changed = ui
                                                .checkbox(
                                                    &mut joint_config.advanced.breakable,
                                                    tr!("enable"),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("break_force"));
                                            config_changed = ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut joint_config.advanced.break_force,
                                                    )
                                                    .speed(10.0)
                                                    .range(0.0..=10000.0),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("break_torque"));
                                            config_changed = ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut joint_config.advanced.break_torque,
                                                    )
                                                    .speed(10.0)
                                                    .range(0.0..=10000.0),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.label(tr!("breakable_description"));
                                    });

                                    // Motor Settings
                                    ui.collapsing(tr!("joint_motor"), |ui| {
                                        ui.label(tr!("motor_settings"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("motor_enabled"));
                                            config_changed = ui
                                                .checkbox(
                                                    &mut joint_config.advanced.motor_enabled,
                                                    tr!("enable"),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("target_velocity"));
                                            config_changed = ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut joint_config.advanced.motor_target_velocity,
                                                    )
                                                    .speed(0.1)
                                                    .range(-100.0..=100.0),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("max_force"));
                                            config_changed = ui
                                                .add(
                                                    egui::DragValue::new(
                                                        &mut joint_config.advanced.motor_max_force,
                                                    )
                                                    .speed(1.0)
                                                    .range(0.0..=1000.0),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("motor_stiffness"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.advanced.motor_stiffness,
                                                        0.0..=100.0,
                                                    )
                                                    .text(tr!("stiffness")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("motor_damping"));
                                            config_changed = ui
                                                .add(
                                                    egui::Slider::new(
                                                        &mut joint_config.advanced.motor_damping,
                                                        0.0..=10.0,
                                                    )
                                                    .text(tr!("damping")),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.label(tr!("motor_description"));
                                    });

                                    // Advanced Physics
                                    ui.collapsing(tr!("advanced_physics"), |ui| {
                                        ui.label(tr!("joint_disable_settings"));
                                        ui.vertical(|ui| {
                                            ui.label(tr!("disabled"));
                                            config_changed = ui
                                                .checkbox(
                                                    &mut joint_config.advanced.disabled,
                                                    tr!("disable_joint"),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.vertical(|ui| {
                                            ui.label(tr!("track_forces"));
                                            config_changed = ui
                                                .checkbox(
                                                    &mut joint_config.advanced.track_forces,
                                                    tr!("track_forces"),
                                                )
                                                .changed()
                                                || config_changed;
                                        });
                                        ui.label(tr!("advanced_physics_description"));
                                    });
                                });

                                ui.separator();

                                // Actions
                                ui.horizontal_wrapped(|ui| {
                                    if ui.button(tr!("reset_defaults")).clicked() {
                                        joint_config.reset_to_defaults();
                                        config_changed = true;
                                    }
                                });

                                if config_changed {
                                    ui.label(tr!("configuration_updated"));
                                }

                                ui.separator();

                                re_ui::Help::new_without_title()
                                    .markdown(tr!("joint_creation_instructions"))
                                    .markdown(tr!("joint_creation"))
                                    .control(tr!("create_joint"), "Drag")
                                    .markdown(tr!("configuration"))
                                    .control(tr!("select_type"), "Dropdown")
                                    .markdown(tr!("joint_types"))
                                    .markdown("- **Distance**: Fixed length joint")
                                    .markdown("- **Revolute**: Hinge-type rotation")
                                    .markdown("- **Prismatic**: Linear sliding motion")
                                    .markdown("- **Fixed**: Rigid connection")
                                    .markdown(tr!("presets"))
                                    .markdown("- **Rigid**: Stiff, low compliance connection")
                                    .markdown("- **Spring**: Flexible, bouncy connection")
                                    .markdown("- **Sliding**: Prismatic with linear limits")
                                    .markdown("- **Hinge**: Revolute with angular limits")
                                    .markdown("- **Breakable**: Joint that breaks under force")
                                    .markdown("- **Motorized**: Joint with automatic motor")
                                    .markdown("- **Suspension**: Spring with damping")
                                    .markdown("- **Rope**: Distance with soft limits")
                                    .markdown(tr!("properties"))
                                    .markdown("- **Compliance**: Adjust flexibility")
                                    .markdown("- **Distance Limits**: Restrict motion range")
                                    .markdown("- **Collision**: Toggle between bodies")
                                    .markdown(tr!("advanced_features"))
                                    .markdown("- **Breakable Joints**: Set force/torque thresholds")
                                    .markdown("- **Joint Motors**: Automatic velocity control")
                                    .markdown("- **Force Tracking**: Monitor joint forces")
                                    .markdown("- **Advanced Physics**: Collision and break settings")
                                    .ui(ui);

                                // Current state
                                if joint_state.is_dragging {
                                    ui.label(tr!("currently_dragging"));
                                    if let (Some(start), Some(current)) =
                                        (joint_state.drag_start_pos, joint_state.drag_current_pos)
                                    {
                                        let distance = start.distance(current);
                                        ui.label(format!("{}: {:.1}", tr!("distance"), distance));
                                    }
                                }
                            } else {
                                ui.label(tr!("joint_configuration_unavailable"));
                            }
                        } else {
                            ui.label(tr!("joint_state_unavailable"));
                        }
                    });
                }
                ToolMode::Select => {
                    ui.heading(tr!("transform_gizmo"));
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Selection Info
                        ui.vertical(|ui| {
                            if let Some(entity) = selected_entity {
                                ui.label(format!("{}: {:?}", tr!("selected"), entity));
                            } else {
                                ui.label(tr!("no_entity_selected"));
                            }
                        });

                        ui.separator();

                        // Extract gizmo settings first
                        let mut gizmo_mode = GizmoMode::Translate;
                        let mut snap_enabled = false;
                        let mut angle_snap = 15.0;
                        let mut scale_snap = 0.1;

                        world.resource_scope(
                            |_world, gizmo_settings: Mut<TransformGizmoSettings>| {
                                gizmo_mode = gizmo_settings.mode;
                                snap_enabled = gizmo_settings.snap_enabled;
                                angle_snap = gizmo_settings.angle_snap;
                                scale_snap = gizmo_settings.scale_snap;
                            },
                        );

                        ui.label(tr!("gizmo_mode"));
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    gizmo_mode == GizmoMode::Translate,
                                    tr!("translate_mode"),
                                )
                                .clicked()
                            {
                                world.resource_scope(
                                    |_world, mut gizmo_settings: Mut<TransformGizmoSettings>| {
                                        gizmo_settings.mode = GizmoMode::Translate;
                                    },
                                );
                            }
                            if ui
                                .selectable_label(
                                    gizmo_mode == GizmoMode::Rotate,
                                    tr!("rotate_mode"),
                                )
                                .clicked()
                            {
                                world.resource_scope(
                                    |_world, mut gizmo_settings: Mut<TransformGizmoSettings>| {
                                        gizmo_settings.mode = GizmoMode::Rotate;
                                    },
                                );
                            }
                            if ui
                                .selectable_label(gizmo_mode == GizmoMode::Scale, tr!("scale_mode"))
                                .clicked()
                            {
                                world.resource_scope(
                                    |_world, mut gizmo_settings: Mut<TransformGizmoSettings>| {
                                        gizmo_settings.mode = GizmoMode::Scale;
                                    },
                                );
                            }
                        });

                        ui.separator();

                        // Snap Settings
                        let mut snap_enabled_local = snap_enabled;
                        ui.checkbox(&mut snap_enabled_local, tr!("enable_snapping"));
                        if snap_enabled_local != snap_enabled {
                            world.resource_scope(
                                |_world, mut gizmo_settings: Mut<TransformGizmoSettings>| {
                                    gizmo_settings.snap_enabled = snap_enabled_local;
                                },
                            );
                        }

                        if snap_enabled_local {
                            ui.horizontal(|ui| {
                                ui.label(tr!("angle_snap"));
                                let mut angle_snap_local = angle_snap;
                                ui.add(egui::DragValue::new(&mut angle_snap_local).speed(1.0));
                                if angle_snap_local != angle_snap {
                                    world.resource_scope(
                                    |_world, mut gizmo_settings: Mut<TransformGizmoSettings>| {
                                        gizmo_settings.angle_snap = angle_snap_local;
                                    },
                                );
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(tr!("scale_snap"));
                                let mut scale_snap_local = scale_snap;
                                ui.add(egui::DragValue::new(&mut scale_snap_local).speed(0.01));
                                if scale_snap_local != scale_snap {
                                    world.resource_scope(
                                    |_world, mut gizmo_settings: Mut<TransformGizmoSettings>| {
                                        gizmo_settings.scale_snap = scale_snap_local;
                                    },
                                );
                                }
                            });
                        }

                        ui.separator();

                        // Quick Actions
                        let center_clicked = ui.button(tr!("center_to_origin")).clicked();
                        let duplicate_clicked = ui.button(tr!("duplicate")).clicked();

                        if center_clicked {
                            if let Some(entity) = selected_entity {
                                if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                                    transform.translation = Vec3::ZERO;
                                }
                            }
                        }

                        if duplicate_clicked {
                            if let Some(entity) = selected_entity {
                                // Send a duplication event
                                world.send_event(DuplicateEntityEvent { original: entity });
                            }
                        }

                        ui.separator();

                        // Selection Controls
                        ui.label(tr!("selection_controls"));
                        ui.horizontal(|ui| {
                            if ui.button(tr!("clear_selection")).clicked() {
                                if let Some(mut selection) =
                                    world.get_resource_mut::<EditorSelection>()
                                {
                                    selection.clear();
                                }
                            }
                            if ui.button(tr!("select_all")).clicked() {
                                // Collect all transformable entities first
                                let mut entities_to_select = Vec::new();
                                let mut query = world.query::<(Entity, &GizmoTransformable)>();
                                for (entity, _) in query.iter(world) {
                                    entities_to_select.push(entity);
                                }

                                // Then update selection
                                if let Some(mut selection) =
                                    world.get_resource_mut::<EditorSelection>()
                                {
                                    selection.clear();
                                    for entity in entities_to_select {
                                        selection.add(entity);
                                    }
                                }
                            }
                        });

                        ui.separator();

                        // Instructions
                        re_ui::Help::new_without_title()
                            .markdown(tr!("selection_controls"))
                            .markdown(tr!("transform_gizmo_controls"))
                            .control(tr!("select_entity"), "Left Click")
                            .control(tr!("multi_select"), "Shift + Click")
                            .control(tr!("translate_mode"), "W")
                            .control(tr!("rotate_mode"), "E")
                            .control(tr!("scale_mode"), "R")
                            .markdown(tr!("gizmo_operations"))
                            .control(tr!("move_axis"), "Drag Arrow")
                            .control(tr!("rotate"), "Drag Ring")
                            .control(tr!("scale"), "Drag Handle")
                            .control(tr!("snapping"), "Ctrl")
                            .ui(ui);
                    });
                }
            }
        });
}

// === UI 辅助函数 ===

/// 基础配置 UI
fn create_basic_ui(ui: &mut egui::Ui, properties: &mut CreationProperties) -> bool {
    let mut changed = false;

    // 碰撞体类型
    ui.vertical(|ui| {
        ui.label(tr!("collider_type"));
        ui.horizontal_wrapped(|ui| {
            changed = ui
                .selectable_value(
                    &mut properties.collider_type,
                    ColliderType::Rectangle,
                    tr!("rectangle"),
                )
                .changed();
            changed = ui
                .selectable_value(
                    &mut properties.collider_type,
                    ColliderType::Circle,
                    tr!("circle"),
                )
                .changed()
                || changed;
            changed = ui
                .selectable_value(
                    &mut properties.collider_type,
                    ColliderType::Capsule,
                    tr!("capsule"),
                )
                .changed()
                || changed;
            changed = ui
                .selectable_value(
                    &mut properties.collider_type,
                    ColliderType::Triangle,
                    tr!("triangle"),
                )
                .changed()
                || changed;
            changed = ui
                .selectable_value(
                    &mut properties.collider_type,
                    ColliderType::Polygon,
                    tr!("polygon"),
                )
                .changed()
                || changed;
        });
    });

    // 物理体类型
    ui.vertical(|ui| {
        ui.label(tr!("body_type"));
        ui.horizontal_wrapped(|ui| {
            changed = ui
                .selectable_value(&mut properties.body_type, RigidBody::Static, tr!("static"))
                .changed();
            changed = ui
                .selectable_value(
                    &mut properties.body_type,
                    RigidBody::Dynamic,
                    tr!("dynamic"),
                )
                .changed()
                || changed;
            changed = ui
                .selectable_value(
                    &mut properties.body_type,
                    RigidBody::Kinematic,
                    tr!("kinematic"),
                )
                .changed()
                || changed;
        });
    });

    // 颜色
    ui.label(tr!("color"));
    let [r, g, b, a] = properties.color.to_srgba().to_f32_array();
    let mut egui_color = egui::ecolor::Color32::from_rgba_unmultiplied(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
        (a * 255.0) as u8,
    );

    let color_changed = ui.color_edit_button_srgba(&mut egui_color).changed();
    if color_changed {
        let [r, g, b, a] = egui_color.to_srgba_unmultiplied();
        properties.color = Color::srgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        changed = true;
    }

    changed
}

/// 预设配置 UI
fn create_presets_ui(ui: &mut egui::Ui, properties: &mut CreationProperties) -> bool {
    let mut changed = false;

    ui.label(tr!("quick_presets"));
    ui.horizontal_wrapped(|ui| {
        if ui.button(tr!("character_controller")).clicked() {
            properties.character_controller();
            changed = true;
        }
        if ui.button(tr!("high_speed_object")).clicked() {
            properties.high_speed_object();
            changed = true;
        }
        if ui.button(tr!("bouncy_ball")).clicked() {
            properties.bouncy_ball();
            changed = true;
        }
        if ui.button(tr!("static_platform")).clicked() {
            properties.static_platform();
            changed = true;
        }
        if ui.button(tr!("trigger_zone")).clicked() {
            properties.trigger_zone();
            changed = true;
        }
        if ui.button(tr!("physics_prop")).clicked() {
            properties.physics_prop();
            changed = true;
        }
        if ui.button(tr!("vehicle")).clicked() {
            properties.vehicle();
            changed = true;
        }
        if ui.button(tr!("anti_gravity")).clicked() {
            properties.anti_gravity();
            changed = true;
        }
        if ui.button(tr!("destructible")).clicked() {
            properties.destructible();
            changed = true;
        }
    });

    changed
}

/// 质量属性 UI
fn create_mass_properties_ui(ui: &mut egui::Ui, properties: &mut CreationProperties) -> bool {
    let mut changed = false;
    let mass = &mut properties.mass_properties;

    // 显式质量 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("explicit_mass"));
    ui.horizontal(|ui| {
        let mut use_explicit_mass = mass.mass.is_some();
        changed = ui
            .checkbox(&mut use_explicit_mass, tr!("use_explicit_mass"))
            .changed()
            || changed;

        if use_explicit_mass {
            if mass.mass.is_none() {
                mass.mass = Some(1.0);
            }
            changed = ui
                .add(
                    egui::DragValue::new(mass.mass.as_mut().unwrap())
                        .speed(0.1)
                        .range(0.0..=1000.0),
                )
                .changed()
                || changed;
        } else {
            mass.mass = None;
            ui.label(tr!("auto_calculated"));
        }
    });

    // 密度
    ui.label(tr!("density"));
    changed = ui
        .add(egui::Slider::new(&mut mass.density, 0.1..=20.0))
        .changed()
        || changed;

    // 显式转动惯量 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("explicit_angular_inertia"));
    ui.horizontal(|ui| {
        let mut use_explicit_inertia = mass.angular_inertia.is_some();
        changed = ui
            .checkbox(
                &mut use_explicit_inertia,
                tr!("use_explicit_angular_inertia"),
            )
            .changed()
            || changed;

        if use_explicit_inertia {
            if mass.angular_inertia.is_none() {
                mass.angular_inertia = Some(1.0);
            }
            changed = ui
                .add(egui::DragValue::new(mass.angular_inertia.as_mut().unwrap()).speed(0.01))
                .changed()
                || changed;
        } else {
            mass.angular_inertia = None;
            ui.label(tr!("auto_calculated"));
        }
    });

    // 显式质心 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("explicit_center_of_mass"));
    ui.horizontal(|ui| {
        let mut use_explicit_com = mass.center_of_mass.is_some();
        changed = ui
            .checkbox(&mut use_explicit_com, tr!("use_explicit_center_of_mass"))
            .changed()
            || changed;

        if use_explicit_com {
            if mass.center_of_mass.is_none() {
                mass.center_of_mass = Some(Vec2::ZERO);
            }
            ui.horizontal(|ui| {
                changed = ui
                    .add(
                        egui::DragValue::new(&mut mass.center_of_mass.as_mut().unwrap().x)
                            .speed(0.1),
                    )
                    .changed()
                    || changed;
                changed = ui
                    .add(
                        egui::DragValue::new(&mut mass.center_of_mass.as_mut().unwrap().y)
                            .speed(0.1),
                    )
                    .changed()
                    || changed;
            });
        } else {
            mass.center_of_mass = None;
            ui.label(tr!("auto_calculated"));
        }
    });

    // 质量属性控制
    ui.label(tr!("mass_properties_control"));
    changed = ui
        .checkbox(&mut mass.no_auto_mass, tr!("disable_auto_mass"))
        .changed()
        || changed;
    changed = ui
        .checkbox(
            &mut mass.no_auto_angular_inertia,
            tr!("disable_auto_angular_inertia"),
        )
        .changed()
        || changed;
    changed = ui
        .checkbox(
            &mut mass.no_auto_center_of_mass,
            tr!("disable_auto_center_of_mass"),
        )
        .changed()
        || changed;

    changed
}

/// 材料属性 UI
fn create_material_properties_ui(ui: &mut egui::Ui, properties: &mut CreationProperties) -> bool {
    let mut changed = false;
    let material = &mut properties.material;

    // 摩擦系数 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("friction_coefficient"));
    ui.horizontal(|ui| {
        let mut use_friction = material.friction.is_some();
        changed = ui
            .checkbox(&mut use_friction, tr!("use_explicit_friction"))
            .changed()
            || changed;

        if use_friction {
            if material.friction.is_none() {
                material.friction = Some(0.5);
            }
            changed = ui
                .add(egui::Slider::new(
                    material.friction.as_mut().unwrap(),
                    0.0..=2.0,
                ))
                .changed()
                || changed;
        } else {
            material.friction = None;
            ui.label(tr!("global_default_0_5"));
        }
    });

    // 静摩擦系数 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("static_friction_coefficient"));
    ui.horizontal(|ui| {
        let mut use_static_friction = material.static_friction.is_some();
        changed = ui
            .checkbox(
                &mut use_static_friction,
                tr!("use_explicit_static_friction"),
            )
            .changed()
            || changed;

        if use_static_friction {
            if material.static_friction.is_none() {
                material.static_friction = Some(0.5);
            }
            changed = ui
                .add(egui::Slider::new(
                    material.static_friction.as_mut().unwrap(),
                    0.0..=2.0,
                ))
                .changed()
                || changed;
        } else {
            material.static_friction = None;
            ui.label(tr!("use_dynamic_friction"));
        }
    });

    // 弹性系数 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("restitution_coefficient"));
    ui.horizontal(|ui| {
        let mut use_restitution = material.restitution.is_some();
        changed = ui
            .checkbox(&mut use_restitution, tr!("use_explicit_restitution"))
            .changed()
            || changed;

        if use_restitution {
            if material.restitution.is_none() {
                material.restitution = Some(0.0);
            }
            changed = ui
                .add(egui::Slider::new(
                    material.restitution.as_mut().unwrap(),
                    0.0..=1.0,
                ))
                .changed()
                || changed;
        } else {
            material.restitution = None;
            ui.label(tr!("global_default_0_0"));
        }
    });

    changed
}

/// 运动属性 UI
fn create_motion_properties_ui(ui: &mut egui::Ui, properties: &mut CreationProperties) -> bool {
    let mut changed = false;
    let motion = &mut properties.motion;

    // 线性阻尼 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("linear_damping"));
    ui.horizontal(|ui| {
        let mut use_linear_damping = motion.linear_damping.is_some();
        changed = ui
            .checkbox(&mut use_linear_damping, tr!("use_explicit_linear_damping"))
            .changed()
            || changed;

        if use_linear_damping {
            if motion.linear_damping.is_none() {
                motion.linear_damping = Some(0.0);
            }
            changed = ui
                .add(egui::Slider::new(
                    motion.linear_damping.as_mut().unwrap(),
                    0.0..=5.0,
                ))
                .changed()
                || changed;
        } else {
            motion.linear_damping = None;
            ui.label(tr!("global_default_0_0"));
        }
    });

    // 角度阻尼 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("angular_damping"));
    ui.horizontal(|ui| {
        let mut use_angular_damping = motion.angular_damping.is_some();
        changed = ui
            .checkbox(
                &mut use_angular_damping,
                tr!("use_explicit_angular_damping"),
            )
            .changed()
            || changed;

        if use_angular_damping {
            if motion.angular_damping.is_none() {
                motion.angular_damping = Some(0.0);
            }
            changed = ui
                .add(egui::Slider::new(
                    motion.angular_damping.as_mut().unwrap(),
                    0.0..=5.0,
                ))
                .changed()
                || changed;
        } else {
            motion.angular_damping = None;
            ui.label(tr!("global_default_0_0"));
        }
    });

    // 重力缩放 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("gravity_scale"));
    ui.horizontal(|ui| {
        let mut use_gravity_scale = motion.gravity_scale.is_some();
        changed = ui
            .checkbox(&mut use_gravity_scale, tr!("use_explicit_gravity_scale"))
            .changed()
            || changed;

        if use_gravity_scale {
            if motion.gravity_scale.is_none() {
                motion.gravity_scale = Some(1.0);
            }
            changed = ui
                .add(egui::Slider::new(
                    motion.gravity_scale.as_mut().unwrap(),
                    -2.0..=2.0,
                ))
                .changed()
                || changed;
        } else {
            motion.gravity_scale = None;
            ui.label(tr!("global_default_1_0"));
        }
    });

    // 最大速度 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("max_linear_speed"));
    ui.horizontal(|ui| {
        let mut use_max_linear_speed = motion.max_linear_speed.is_some();
        changed = ui
            .checkbox(&mut use_max_linear_speed, tr!("limit_linear_speed"))
            .changed()
            || changed;

        if use_max_linear_speed {
            if motion.max_linear_speed.is_none() {
                motion.max_linear_speed = Some(50.0);
            }
            changed = ui
                .add(egui::Slider::new(
                    motion.max_linear_speed.as_mut().unwrap(),
                    0.0..=100.0,
                ))
                .changed()
                || changed;
        } else {
            motion.max_linear_speed = None;
            ui.label(tr!("no_limit"));
        }
    });

    ui.label(tr!("max_angular_speed"));
    ui.horizontal(|ui| {
        let mut use_max_angular_speed = motion.max_angular_speed.is_some();
        changed = ui
            .checkbox(&mut use_max_angular_speed, tr!("limit_angular_speed"))
            .changed()
            || changed;

        if use_max_angular_speed {
            if motion.max_angular_speed.is_none() {
                motion.max_angular_speed = Some(5.0);
            }
            changed = ui
                .add(egui::Slider::new(
                    motion.max_angular_speed.as_mut().unwrap(),
                    0.0..=10.0,
                ))
                .changed()
                || changed;
        } else {
            motion.max_angular_speed = None;
            ui.label(tr!("no_limit"));
        }
    });

    // 锁定轴
    ui.label(tr!("lock_axes"));
    ui.horizontal(|ui| {
        let mut lock_x = motion
            .locked_axes
            .as_ref()
            .map_or(false, |axes| axes.is_translation_x_locked());
        let mut lock_y = motion
            .locked_axes
            .as_ref()
            .map_or(false, |axes| axes.is_translation_y_locked());
        let mut lock_rotation = motion
            .locked_axes
            .as_ref()
            .map_or(false, |axes| axes.is_rotation_locked());

        changed = ui.checkbox(&mut lock_x, tr!("lock_x")).changed() || changed;
        changed = ui.checkbox(&mut lock_y, tr!("lock_y")).changed() || changed;
        changed = ui
            .checkbox(&mut lock_rotation, tr!("lock_rotation"))
            .changed()
            || changed;

        // 更新锁定轴状态
        let mut new_locked_axes = LockedAxes::new();
        if lock_x {
            new_locked_axes = new_locked_axes.lock_translation_x();
        }
        if lock_y {
            new_locked_axes = new_locked_axes.lock_translation_y();
        }
        if lock_rotation {
            new_locked_axes = new_locked_axes.lock_rotation();
        }

        // 只有当有轴被锁定时才设置 locked_axes
        if new_locked_axes.is_translation_x_locked()
            || new_locked_axes.is_translation_y_locked()
            || new_locked_axes.is_rotation_locked()
        {
            motion.locked_axes = Some(new_locked_axes);
        } else {
            motion.locked_axes = None;
        }
    });

    // 优势值 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("dominance_value"));
    ui.horizontal(|ui| {
        let mut use_dominance = motion.dominance.is_some();
        changed = ui
            .checkbox(&mut use_dominance, tr!("use_dominance_value"))
            .changed()
            || changed;

        if use_dominance {
            if motion.dominance.is_none() {
                motion.dominance = Some(0);
            }
            changed = ui
                .add(egui::Slider::new(
                    motion.dominance.as_mut().unwrap(),
                    -127..=127,
                ))
                .changed()
                || changed;
        } else {
            motion.dominance = None;
            ui.label(tr!("default"));
        }
    });

    changed
}

/// 碰撞属性 UI
fn create_collision_properties_ui(
    ui: &mut egui::Ui,
    properties: &mut CreationProperties,
    world: &mut World,
) -> bool {
    let mut changed = false;
    let collision = &mut properties.collision;

    // 传感器
    changed = ui
        .checkbox(
            &mut collision.is_sensor,
            tr!("sensor_no_collision_response"),
        )
        .changed()
        || changed;

    // 碰撞层配置 - 简化版本，使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("collision_layers"));
    ui.vertical(|ui| {
        let mut use_collision_layers = collision.collision_layers.is_some();
        changed = ui
            .checkbox(&mut use_collision_layers, tr!("use_collision_layers"))
            .changed()
            || changed;

        if use_collision_layers {
            if collision.collision_layers.is_none() {
                collision.collision_layers = Some(avian2d::prelude::CollisionLayers::default());
            }

            // 显示简化的碰撞层配置信息
            if let Some(ref mut collision_layers) = collision.collision_layers {
                crate::ui::collision_layer_ui::UnifiedCollisionLayerUI::render(
                    ui,
                    world,
                    collision_layers,
                );
            }
        } else {
            collision.collision_layers = None;
            ui.label(tr!("default_collision_all"));
        }
    });

    // 碰撞边距
    ui.label(tr!("collision_margin"));
    changed = ui
        .add(egui::DragValue::new(&mut collision.collision_margin).speed(0.001))
        .changed()
        || changed;

    // 推测接触边距 (CCD) - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("speculative_margin_ccd"));
    ui.horizontal(|ui| {
        let mut use_speculative_margin = collision.speculative_margin.is_some();
        changed = ui
            .checkbox(&mut use_speculative_margin, tr!("use_speculative_margin"))
            .changed()
            || changed;

        if use_speculative_margin {
            if collision.speculative_margin.is_none() {
                collision.speculative_margin = Some(0.1);
            }
            changed = ui
                .add(
                    egui::DragValue::new(collision.speculative_margin.as_mut().unwrap())
                        .speed(0.01),
                )
                .changed()
                || changed;
        } else {
            collision.speculative_margin = None;
            ui.label(tr!("no_limit"));
        }
    });

    // 扫描CCD
    changed = ui
        .checkbox(&mut collision.swept_ccd, tr!("enable_swept_ccd"))
        .changed()
        || changed;

    // 碰撞事件
    changed = ui
        .checkbox(
            &mut collision.collision_events,
            tr!("enable_collision_events"),
        )
        .changed()
        || changed;

    // 禁用碰撞体
    changed = ui
        .checkbox(&mut collision.collider_disabled, tr!("disable_collider"))
        .changed()
        || changed;

    changed
}

/// 性能属性 UI
fn create_performance_properties_ui(
    ui: &mut egui::Ui,
    properties: &mut CreationProperties,
) -> bool {
    let mut changed = false;
    let performance = &mut properties.performance;

    // 禁用睡眠
    changed = ui
        .checkbox(&mut performance.disable_sleeping, tr!("disable_sleeping"))
        .changed()
        || changed;

    // 禁用物理
    changed = ui
        .checkbox(
            &mut performance.physics_disabled,
            tr!("disable_physics_simulation"),
        )
        .changed()
        || changed;

    // 变换插值
    changed = ui
        .checkbox(
            &mut performance.transform_interpolation,
            tr!("enable_transform_interpolation"),
        )
        .changed()
        || changed;

    changed
}

/// 高级物理 UI
fn create_advanced_physics_ui(ui: &mut egui::Ui, properties: &mut CreationProperties) -> bool {
    let mut changed = false;
    let advanced = &mut properties.advanced;

    // 常力 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("constant_force_world_space"));
    ui.horizontal(|ui| {
        let mut use_constant_force = advanced.constant_force.is_some();
        changed = ui
            .checkbox(&mut use_constant_force, tr!("use_constant_force"))
            .changed()
            || changed;

        if use_constant_force {
            if advanced.constant_force.is_none() {
                advanced.constant_force = Some(Vec2::ZERO);
            }
            ui.horizontal(|ui| {
                changed = ui
                    .add(
                        egui::DragValue::new(&mut advanced.constant_force.as_mut().unwrap().x)
                            .speed(0.1),
                    )
                    .changed()
                    || changed;
                changed = ui
                    .add(
                        egui::DragValue::new(&mut advanced.constant_force.as_mut().unwrap().y)
                            .speed(0.1),
                    )
                    .changed()
                    || changed;
            });
        } else {
            advanced.constant_force = None;
            ui.label(tr!("none"));
        }
    });

    // 常本地力 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("constant_force_local_space"));
    ui.horizontal(|ui| {
        let mut use_constant_local_force = advanced.constant_local_force.is_some();
        changed = ui
            .checkbox(
                &mut use_constant_local_force,
                tr!("use_local_constant_force"),
            )
            .changed()
            || changed;

        if use_constant_local_force {
            if advanced.constant_local_force.is_none() {
                advanced.constant_local_force = Some(Vec2::ZERO);
            }
            ui.horizontal(|ui| {
                changed = ui
                    .add(
                        egui::DragValue::new(
                            &mut advanced.constant_local_force.as_mut().unwrap().x,
                        )
                        .speed(0.1),
                    )
                    .changed()
                    || changed;
                changed = ui
                    .add(
                        egui::DragValue::new(
                            &mut advanced.constant_local_force.as_mut().unwrap().y,
                        )
                        .speed(0.1),
                    )
                    .changed()
                    || changed;
            });
        } else {
            advanced.constant_local_force = None;
            ui.label(tr!("none"));
        }
    });

    // 常扭矩 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("constant_torque"));
    ui.horizontal(|ui| {
        let mut use_constant_torque = advanced.constant_torque.is_some();
        changed = ui
            .checkbox(&mut use_constant_torque, tr!("use_constant_torque"))
            .changed()
            || changed;

        if use_constant_torque {
            if advanced.constant_torque.is_none() {
                advanced.constant_torque = Some(0.0);
            }
            changed = ui
                .add(egui::DragValue::new(advanced.constant_torque.as_mut().unwrap()).speed(0.1))
                .changed()
                || changed;
        } else {
            advanced.constant_torque = None;
            ui.label(tr!("none"));
        }
    });

    // 常线性加速度 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("constant_linear_acceleration_world_space"));
    ui.horizontal(|ui| {
        let mut use_constant_linear_acceleration = advanced.constant_linear_acceleration.is_some();
        changed = ui
            .checkbox(
                &mut use_constant_linear_acceleration,
                tr!("use_constant_linear_acceleration"),
            )
            .changed()
            || changed;

        if use_constant_linear_acceleration {
            if advanced.constant_linear_acceleration.is_none() {
                advanced.constant_linear_acceleration = Some(Vec2::ZERO);
            }
            ui.horizontal(|ui| {
                changed = ui
                    .add(
                        egui::DragValue::new(
                            &mut advanced.constant_linear_acceleration.as_mut().unwrap().x,
                        )
                        .speed(0.1),
                    )
                    .changed()
                    || changed;
                changed = ui
                    .add(
                        egui::DragValue::new(
                            &mut advanced.constant_linear_acceleration.as_mut().unwrap().y,
                        )
                        .speed(0.1),
                    )
                    .changed()
                    || changed;
            });
        } else {
            advanced.constant_linear_acceleration = None;
            ui.label(tr!("none"));
        }
    });

    // 常本地线性加速度 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("constant_linear_acceleration_local_space"));
    ui.horizontal(|ui| {
        let mut use_constant_local_linear_acceleration =
            advanced.constant_local_linear_acceleration.is_some();
        changed = ui
            .checkbox(
                &mut use_constant_local_linear_acceleration,
                tr!("use_local_constant_linear_acceleration"),
            )
            .changed()
            || changed;

        if use_constant_local_linear_acceleration {
            if advanced.constant_local_linear_acceleration.is_none() {
                advanced.constant_local_linear_acceleration = Some(Vec2::ZERO);
            }
            ui.horizontal(|ui| {
                changed = ui
                    .add(
                        egui::DragValue::new(
                            &mut advanced
                                .constant_local_linear_acceleration
                                .as_mut()
                                .unwrap()
                                .x,
                        )
                        .speed(0.1),
                    )
                    .changed()
                    || changed;
                changed = ui
                    .add(
                        egui::DragValue::new(
                            &mut advanced
                                .constant_local_linear_acceleration
                                .as_mut()
                                .unwrap()
                                .y,
                        )
                        .speed(0.1),
                    )
                    .changed()
                    || changed;
            });
        } else {
            advanced.constant_local_linear_acceleration = None;
            ui.label(tr!("none"));
        }
    });

    // 常角加速度 - 使用 InspectorOptions 风格的 Option 处理
    ui.label(tr!("constant_angular_acceleration"));
    ui.horizontal(|ui| {
        let mut use_constant_angular_acceleration =
            advanced.constant_angular_acceleration.is_some();
        changed = ui
            .checkbox(
                &mut use_constant_angular_acceleration,
                tr!("use_constant_angular_acceleration"),
            )
            .changed()
            || changed;

        if use_constant_angular_acceleration {
            if advanced.constant_angular_acceleration.is_none() {
                advanced.constant_angular_acceleration = Some(0.0);
            }
            changed = ui
                .add(
                    egui::DragValue::new(advanced.constant_angular_acceleration.as_mut().unwrap())
                        .speed(0.1),
                )
                .changed()
                || changed;
        } else {
            advanced.constant_angular_acceleration = None;
            ui.label(tr!("none"));
        }
    });

    changed
}
