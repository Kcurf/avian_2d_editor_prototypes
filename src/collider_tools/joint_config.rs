//! Enhanced joint configuration system
//!
//! Provides unified configuration for all joint types with comprehensive property support.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::JointType;

/// 统一的Joint配置结构体，支持所有Joint类型的自定义属性
#[derive(Resource, Clone, Debug)]
pub struct JointConfiguration {
    /// 当前选中的Joint类型
    pub joint_type: JointType,
    /// 通用配置
    pub common: CommonJointConfig,
    /// FixedJoint专用配置
    pub fixed: FixedJointConfig,
    /// DistanceJoint专用配置
    pub distance: DistanceJointConfig,
    /// PrismaticJoint专用配置
    pub prismatic: PrismaticJointConfig,
    /// RevoluteJoint专用配置
    pub revolute: RevoluteJointConfig,
}

/// 枚举类型，包含所有Joint类型的配置
#[derive(Clone, Debug)]
pub enum JointConfigurationEnum {
    /// FixedJoint配置
    Fixed {
        common: CommonJointConfig,
        config: FixedJointConfig,
    },
    /// DistanceJoint配置
    Distance {
        common: CommonJointConfig,
        config: DistanceJointConfig,
    },
    /// PrismaticJoint配置
    Prismatic {
        common: CommonJointConfig,
        config: PrismaticJointConfig,
    },
    /// RevoluteJoint配置
    Revolute {
        common: CommonJointConfig,
        config: RevoluteJointConfig,
    },
}

impl Default for JointConfigurationEnum {
    fn default() -> Self {
        Self::Distance {
            common: CommonJointConfig::default(),
            config: DistanceJointConfig::default(),
        }
    }
}

impl JointConfigurationEnum {
    /// 获取当前配置的通用属性
    pub fn common(&self) -> &CommonJointConfig {
        match self {
            JointConfigurationEnum::Fixed { common, .. } => common,
            JointConfigurationEnum::Distance { common, .. } => common,
            JointConfigurationEnum::Prismatic { common, .. } => common,
            JointConfigurationEnum::Revolute { common, .. } => common,
        }
    }

    /// 创建Joint
    pub fn create_physics_joint(
        &self,
        commands: &mut Commands,
        local_anchor_1: Vec2,
        local_anchor_2: Vec2,
        entity1: Entity,
        entity2: Entity,
    ) -> Entity {
        match self {
            JointConfigurationEnum::Fixed { common, config } => {
                let mut joint = FixedJoint::new(entity1, entity2);
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                if config.point_compliance != 0.0 {
                    joint = joint.with_point_compliance(config.point_compliance);
                }
                if config.angle_compliance != 0.0 {
                    joint = joint.with_angle_compliance(config.angle_compliance);
                }
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    commands
                        .spawn((
                            joint,
                            JointDamping {
                                linear: common.damping_linear,
                                angular: common.damping_angular,
                            },
                        ))
                        .id()
                } else {
                    commands.spawn(joint).id()
                }
            }
            JointConfigurationEnum::Distance { common, config } => {
                let mut joint = DistanceJoint::new(entity1, entity2);
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                if config.compliance != 0.0 {
                    joint = joint.with_compliance(config.compliance);
                }
                if config.rest_length != 100.0 {
                    // Use with_limits instead of deprecated with_rest_length
                    if let Some(max) = config.max_distance {
                        joint = joint.with_limits(0.0, max);
                    }
                }
                if let (Some(min), Some(max)) = (config.min_distance, config.max_distance) {
                    joint = joint.with_limits(min, max);
                }
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    commands
                        .spawn((
                            joint,
                            JointDamping {
                                linear: common.damping_linear,
                                angular: common.damping_angular,
                            },
                        ))
                        .id()
                } else {
                    commands.spawn(joint).id()
                }
            }
            JointConfigurationEnum::Prismatic { common, config } => {
                let mut joint = PrismaticJoint::new(entity1, entity2);
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                if config.free_axis != Vec2::X {
                    joint = joint.with_slider_axis(config.free_axis);
                }
                if config.axis_compliance != 0.0 {
                    joint = joint.with_align_compliance(config.axis_compliance);
                }
                if config.limit_compliance != 0.0 {
                    joint = joint.with_limit_compliance(config.limit_compliance);
                }
                if config.angle_compliance != 0.0 {
                    joint = joint.with_angle_compliance(config.angle_compliance);
                }
                if let (Some(min), Some(max)) = (config.min_distance, config.max_distance) {
                    joint = joint.with_limits(min, max);
                }
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    commands
                        .spawn((
                            joint,
                            JointDamping {
                                linear: common.damping_linear,
                                angular: common.damping_angular,
                            },
                        ))
                        .id()
                } else {
                    commands.spawn(joint).id()
                }
            }
            JointConfigurationEnum::Revolute { common, config } => {
                let mut joint = RevoluteJoint::new(entity1, entity2);

                // Apply local anchors (from the function parameters, provided by anchor system)
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                // Apply basis rotation if specified
                if let Some(basis) = config.basis {
                    joint = joint.with_basis(basis);
                }

                // Apply compliance values
                if config.point_compliance != 0.0 {
                    joint = joint.with_point_compliance(config.point_compliance);
                }
                if config.limit_compliance != 0.0 {
                    joint = joint.with_limit_compliance(config.limit_compliance);
                }

                // Apply angle limits
                if let (Some(min), Some(max)) = (config.min_angle, config.max_angle) {
                    joint = joint.with_angle_limits(min, max);
                }
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    commands
                        .spawn((
                            joint,
                            JointDamping {
                                linear: common.damping_linear,
                                angular: common.damping_angular,
                            },
                        ))
                        .id()
                } else {
                    commands.spawn(joint).id()
                }
            }
        }
    }
}

impl Default for JointConfiguration {
    fn default() -> Self {
        Self {
            joint_type: JointType::Distance,
            common: CommonJointConfig {
                damping_linear: 0.1,
                damping_angular: 1.0,
                disable_collision: false,
            },
            fixed: FixedJointConfig {
                point_compliance: 0.0,
                angle_compliance: 0.0,
            },
            distance: DistanceJointConfig {
                compliance: 0.00000001,
                rest_length: 100.0,
                min_distance: None,
                max_distance: None,
            },
            prismatic: PrismaticJointConfig {
                free_axis: Vec2::X,
                axis_compliance: 0.0,
                limit_compliance: 0.0,
                angle_compliance: 0.0,
                min_distance: None,
                max_distance: None,
            },
            revolute: RevoluteJointConfig {
                point_compliance: 0.0,
                limit_compliance: 0.0,
                basis: None,
                min_angle: None,
                max_angle: None,
            },
        }
    }
}

/// 所有Joint通用的配置
#[derive(Clone, Debug, Default)]
pub struct CommonJointConfig {
    /// 线性速度阻尼
    pub damping_linear: f32,
    /// 角速度阻尼
    pub damping_angular: f32,
    /// 禁用碰撞
    pub disable_collision: bool,
}

/// FixedJoint配置
#[derive(Clone, Debug, Default)]
pub struct FixedJointConfig {
    /// 位置约束柔度（刚度的倒数，m/N）
    pub point_compliance: f32,
    /// 角度约束柔度（刚度的倒数，N⋅m/rad）
    pub angle_compliance: f32,
}

/// DistanceJoint配置
#[derive(Clone, Debug, Default)]
pub struct DistanceJointConfig {
    /// 柔度（刚度的倒数，m/N）
    pub compliance: f32,
    /// 目标静止长度
    pub rest_length: f32,
    /// 最小距离限制
    pub min_distance: Option<f32>,
    /// 最大距离限制
    pub max_distance: Option<f32>,
}

/// PrismaticJoint配置
#[derive(Clone, Debug, Default)]
pub struct PrismaticJointConfig {
    /// 自由移动轴
    pub free_axis: Vec2,
    /// 轴对齐柔度
    pub axis_compliance: f32,
    /// 限制柔度
    pub limit_compliance: f32,
    /// 角度约束柔度
    pub angle_compliance: f32,
    /// 最小移动距离
    pub min_distance: Option<f32>,
    /// 最大移动距离
    pub max_distance: Option<f32>,
}

/// RevoluteJoint配置
#[derive(Clone, Debug, Default)]
pub struct RevoluteJointConfig {
    /// 位置约束柔度 (m/N)
    pub point_compliance: f32,
    /// 限制柔度 (N*m/rad)
    pub limit_compliance: f32,
    /// 基础旋转角度 (弧度)
    pub basis: Option<f32>,
    /// 最小角度限制（弧度）
    pub min_angle: Option<f32>,
    /// 最大角度限制（弧度）
    pub max_angle: Option<f32>,
}

impl JointConfiguration {
    /// 创建默认配置
    pub fn new(joint_type: JointType) -> Self {
        let mut config = Self::default();
        config.joint_type = joint_type;
        config
    }

    /// 转换为枚举类型配置
    pub fn to_enum(&self) -> JointConfigurationEnum {
        match self.joint_type {
            JointType::Fixed => JointConfigurationEnum::Fixed {
                common: self.common.clone(),
                config: self.fixed.clone(),
            },
            JointType::Distance => JointConfigurationEnum::Distance {
                common: self.common.clone(),
                config: self.distance.clone(),
            },
            JointType::Prismatic => JointConfigurationEnum::Prismatic {
                common: self.common.clone(),
                config: self.prismatic.clone(),
            },
            JointType::Revolute => JointConfigurationEnum::Revolute {
                common: self.common.clone(),
                config: self.revolute.clone(),
            },
        }
    }

    /// 重置为当前Joint类型的默认配置
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// 获取刚性连接的预设
    pub fn rigid_connection(&mut self) {
        self.joint_type = JointType::Fixed;
        self.fixed.point_compliance = 0.0;
        self.fixed.angle_compliance = 0.0;
    }

    /// 获取弹性连接的预设
    pub fn spring_connection(&mut self) {
        self.joint_type = JointType::Distance;
        self.distance.compliance = 0.001;
        self.distance.rest_length = 50.0;
        self.common.damping_linear = 0.5;
        self.common.damping_angular = 0.5;
    }

    /// 获取滑动门的预设
    pub fn sliding_door(&mut self) {
        self.joint_type = JointType::Prismatic;
        self.prismatic.free_axis = Vec2::X;
        self.prismatic.min_distance = Some(0.0);
        self.prismatic.max_distance = Some(200.0);
        self.prismatic.axis_compliance = 0.0;
        self.prismatic.angle_compliance = 0.0;
    }

    /// 获取铰链的预设
    pub fn hinge(&mut self) {
        self.joint_type = JointType::Revolute;
        self.revolute.min_angle = Some(-std::f32::consts::PI / 4.0); // -45度
        self.revolute.max_angle = Some(std::f32::consts::PI / 4.0); // +45度
        self.revolute.point_compliance = 0.0;
        self.revolute.limit_compliance = 0.0;
    }
}
