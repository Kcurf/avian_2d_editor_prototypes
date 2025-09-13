use std::f32::consts::TAU;

use super::{
    InteractionKind, InternalGizmoCamera, ScaleGizmo, TransformGizmo, TransformGizmoSettings,
    TranslationGizmo,
};
use bevy::{
    core_pipeline::core_3d::Camera3dDepthLoadOp, pbr::NotShadowCaster, prelude::*,
    render::view::RenderLayers,
};

#[derive(Component)]
pub struct RotationGizmo;

#[derive(Component)]
pub struct ViewTranslateGizmo;

/// Startup system that builds the procedural mesh and materials of the gizmo.
pub fn build_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<TransformGizmoSettings>,
) {
    let axis_length = 1.5;
    let arc_radius = TAU / 4.0;
    let plane_size = 0.3;
    let plane_offset = 0.4;

    // Define improved gizmo meshes with better proportions
    let arrow_tail_mesh = meshes.add(Capsule3d {
        radius: 0.03, // Slightly thinner for precision
        half_length: axis_length * 0.45,
    });

    let cone_mesh = meshes.add(Cone {
        height: 0.2,
        radius: 0.08, // Smaller, more precise arrow heads
    });

    // Plane handles for multi-axis translation
    let plane_mesh = meshes.add(Plane3d::default().mesh().size(plane_size, plane_size));

    // Center sphere for free movement
    let sphere_mesh = meshes.add(Sphere { radius: 0.15 });

    // Scale gizmo handles - small cubes at the end of axes
    let scale_handle_mesh = meshes.add(Cuboid::new(0.12, 0.12, 0.12));

    // Rotation rings with better visibility
    let rotation_mesh = meshes.add(Mesh::from(
        Torus {
            major_radius: 1.1,
            minor_radius: 0.03,
        }
        .mesh()
        .angle_range(0f32..=arc_radius * 0.8), // Partial arcs for cleaner look
    ));

    /// Helper function to create a material with a specific color
    fn material(color: Color) -> StandardMaterial {
        StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            ..default()
        }
    }

    // Editor color scheme - matching CSS specification
    let gizmo_matl_x = materials.add(material(Color::srgba(0.8, 0.25, 0.32, 0.9))); // Red X-axis: #CC3F51
    let gizmo_matl_y = materials.add(material(Color::srgba(0.36, 0.7, 0.05, 0.9))); // Green Y-axis: #5CB20D
    let gizmo_matl_z = materials.add(material(Color::srgba(0.13, 0.5, 0.8, 0.9))); // Blue Z-axis: #2180CC

    // Brighter versions for selected/hovered state
    let gizmo_matl_x_sel = materials.add(material(Color::srgba(1.0, 0.4, 0.45, 1.0))); // Bright red
    let gizmo_matl_y_sel = materials.add(material(Color::srgba(0.5, 0.9, 0.2, 1.0))); // Bright green
    let gizmo_matl_z_sel = materials.add(material(Color::srgba(0.25, 0.65, 1.0, 1.0))); // Bright blue

    // View gizmo - neutral white/gray
    let gizmo_matl_v_sel = materials.add(material(Color::srgba(0.9, 0.9, 0.9, 0.8)));

    // Build the gizmo using the variables above.
    commands
        .spawn(TransformGizmo::default())
        .with_children(|parent| {
            // 通用组件
            let base_components = |mesh: Mesh3d,
                                   material: MeshMaterial3d<StandardMaterial>,
                                   transform: Transform,
                                   interaction: InteractionKind| {
                (
                    mesh,
                    material,
                    transform,
                    interaction,
                    TranslationGizmo,
                    NotShadowCaster,
                    RenderLayers::layer(12),
                )
            };

            // 平移轴（X和Y轴始终启用）
            parent.spawn(base_components(
                Mesh3d(arrow_tail_mesh.clone()),
                MeshMaterial3d(gizmo_matl_x.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / 2.0),
                    Vec3::new(axis_length / 2.0, 0.0, 0.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
            ));

            parent.spawn(base_components(
                Mesh3d(arrow_tail_mesh.clone()),
                MeshMaterial3d(gizmo_matl_y.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_y(std::f32::consts::PI / 2.0),
                    Vec3::new(0.0, axis_length / 2.0, 0.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
            ));

            // Z轴相关组件（仅在启用Z轴时创建）
            if settings.enable_z_axis {
                // Z轴平移轴
                parent.spawn(base_components(
                    Mesh3d(arrow_tail_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, axis_length / 2.0),
                    )),
                    InteractionKind::TranslateAxis {
                        original: Vec3::Z,
                        axis: Vec3::Z,
                    },
                ));

                // Z轴平移手柄
                parent.spawn(base_components(
                    Mesh3d(cone_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z_sel.clone()),
                    Transform::from_matrix(Mat4::from_rotation_translation(
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                        Vec3::new(0.0, 0.0, axis_length),
                    )),
                    InteractionKind::TranslateAxis {
                        original: Vec3::Z,
                        axis: Vec3::Z,
                    },
                ));

                // 平面平移手柄（仅在同时启用平面gizmo时）
                if settings.enable_plane_gizmos {
                    // X轴平面
                    parent.spawn((
                        Mesh3d(plane_mesh.clone()),
                        MeshMaterial3d(gizmo_matl_x_sel.clone()),
                        Transform::from_matrix(Mat4::from_rotation_translation(
                            Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                            Vec3::new(0., plane_offset, plane_offset),
                        )),
                        InteractionKind::TranslatePlane {
                            original: Vec3::X,
                            normal: Vec3::X,
                        },
                        TranslationGizmo,
                        NotShadowCaster,
                        RenderLayers::layer(12),
                    ));

                    // Y轴平面
                    parent.spawn((
                        Mesh3d(plane_mesh.clone()),
                        MeshMaterial3d(gizmo_matl_y_sel.clone()),
                        Transform::from_translation(Vec3::new(plane_offset, 0.0, plane_offset)),
                        InteractionKind::TranslatePlane {
                            original: Vec3::Y,
                            normal: Vec3::Y,
                        },
                        TranslationGizmo,
                        NotShadowCaster,
                        RenderLayers::layer(12),
                    ));

                    // Z轴平面
                    parent.spawn((
                        Mesh3d(plane_mesh.clone()),
                        MeshMaterial3d(gizmo_matl_z_sel.clone()),
                        Transform::from_matrix(Mat4::from_rotation_translation(
                            Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                            Vec3::new(plane_offset, plane_offset, 0.0),
                        )),
                        InteractionKind::TranslatePlane {
                            original: Vec3::Z,
                            normal: Vec3::Z,
                        },
                        TranslationGizmo,
                        NotShadowCaster,
                        RenderLayers::layer(12),
                    ));
                }
            }

            // X和Y轴平移手柄（始终启用）
            parent.spawn(base_components(
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(gizmo_matl_x_sel.clone()),
                Transform::from_matrix(Mat4::from_rotation_translation(
                    Quat::from_rotation_z(std::f32::consts::PI / -2.0),
                    Vec3::new(axis_length, 0.0, 0.0),
                )),
                InteractionKind::TranslateAxis {
                    original: Vec3::X,
                    axis: Vec3::X,
                },
            ));

            parent.spawn(base_components(
                Mesh3d(cone_mesh.clone()),
                MeshMaterial3d(gizmo_matl_y_sel.clone()),
                Transform::from_translation(Vec3::new(0.0, axis_length, 0.0)),
                InteractionKind::TranslateAxis {
                    original: Vec3::Y,
                    axis: Vec3::Y,
                },
            ));

            // 视图平移gizmo（XY平面移动，适用于2D和3D模式）
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(gizmo_matl_v_sel.clone()),
                Transform::default(),
                InteractionKind::TranslatePlane {
                    original: Vec3::ZERO,
                    normal: Vec3::Z,
                },
                ViewTranslateGizmo,
                TranslationGizmo,
                NotShadowCaster,
                RenderLayers::layer(12),
            ));

            // 旋转弧（仅在启用旋转gizmo时）
            if settings.enable_rotation_gizmos {
                // Z轴旋转（始终启用，适用于2D和3D模式）
                parent.spawn((
                    Mesh3d(rotation_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_z.clone()),
                    Transform::from_rotation(
                        Quat::from_axis_angle(Vec3::Z, f32::to_radians(90.0))
                            * Quat::from_axis_angle(Vec3::X, f32::to_radians(90.0)),
                    ),
                    RotationGizmo,
                    InteractionKind::RotateAxis {
                        original: Vec3::Z,
                        axis: Vec3::Z,
                    },
                    NotShadowCaster,
                    RenderLayers::layer(12),
                ));

                // X和Y轴旋转（仅在启用Z轴时，用于3D模式）
                if settings.enable_z_axis {
                    parent.spawn((
                        Mesh3d(rotation_mesh.clone()),
                        MeshMaterial3d(gizmo_matl_x.clone()),
                        Transform::from_rotation(Quat::from_axis_angle(
                            Vec3::Z,
                            f32::to_radians(90.0),
                        )),
                        RotationGizmo,
                        InteractionKind::RotateAxis {
                            original: Vec3::X,
                            axis: Vec3::X,
                        },
                        NotShadowCaster,
                        RenderLayers::layer(12),
                    ));

                    parent.spawn((
                        Mesh3d(rotation_mesh.clone()),
                        MeshMaterial3d(gizmo_matl_y.clone()),
                        Transform::default(),
                        RotationGizmo,
                        InteractionKind::RotateAxis {
                            original: Vec3::Y,
                            axis: Vec3::Y,
                        },
                        NotShadowCaster,
                        RenderLayers::layer(12),
                    ));
                }
            }

            // 缩放手柄（仅在启用缩放gizmo时）
            if settings.enable_scale_gizmos {
                // X和Y轴缩放手柄（始终启用）
                parent.spawn((
                    Mesh3d(scale_handle_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_x_sel.clone()),
                    Transform::from_translation(Vec3::new(axis_length + 0.15, 0.0, 0.0)),
                    InteractionKind::ScaleAxis {
                        original: Vec3::X,
                        axis: Vec3::X,
                    },
                    ScaleGizmo,
                    NotShadowCaster,
                    RenderLayers::layer(12),
                ));

                parent.spawn((
                    Mesh3d(scale_handle_mesh.clone()),
                    MeshMaterial3d(gizmo_matl_y_sel.clone()),
                    Transform::from_translation(Vec3::new(0.0, axis_length + 0.15, 0.0)),
                    InteractionKind::ScaleAxis {
                        original: Vec3::Y,
                        axis: Vec3::Y,
                    },
                    ScaleGizmo,
                    NotShadowCaster,
                    RenderLayers::layer(12),
                ));

                // Z轴缩放手柄（仅在启用Z轴时）
                if settings.enable_z_axis {
                    parent.spawn((
                        Mesh3d(scale_handle_mesh.clone()),
                        MeshMaterial3d(gizmo_matl_z_sel.clone()),
                        Transform::from_translation(Vec3::new(0.0, 0.0, axis_length + 0.15)),
                        InteractionKind::ScaleAxis {
                            original: Vec3::Z,
                            axis: Vec3::Z,
                        },
                        ScaleGizmo,
                        NotShadowCaster,
                        RenderLayers::layer(12),
                    ));
                }

                // 均匀缩放手柄 - 中心的大立方体
                parent.spawn((
                    Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
                    MeshMaterial3d(materials.add(material(Color::srgba(0.9, 0.9, 0.9, 0.7)))),
                    Transform::from_translation(Vec3::ZERO),
                    InteractionKind::ScaleUniform {
                        original: Vec3::ONE,
                    },
                    ScaleGizmo,
                    NotShadowCaster,
                    RenderLayers::layer(12),
                ));
            }
        });

    commands.spawn((
        Camera3d {
            depth_load_op: Camera3dDepthLoadOp::Clear(0.),
            ..default()
        },
        Camera {
            clear_color: ClearColorConfig::None,
            ..default()
        },
        InternalGizmoCamera,
        RenderLayers::layer(12),
    ));
}
