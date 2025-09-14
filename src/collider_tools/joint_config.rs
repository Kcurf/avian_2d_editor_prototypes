//! Enhanced joint configuration system
//!
//! Provides comprehensive configuration for all Avian 2D joint types with full support for
//! the new Joint system including advanced features like force feedback, breakable joints,
//! collision control, and sophisticated constraint configuration.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::JointType;

/// Advanced joint components for breakable joints and force tracking
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct BreakableJoint {
    /// Force threshold at which joint breaks (Newtons)
    pub break_force: f32,
    /// Torque threshold at which joint breaks (Newton-meters)
    pub break_torque: f32,
}

/// Motor component for powered joints
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct JointMotor {
    /// Target velocity for the motor
    pub target_velocity: f32,
    /// Maximum force the motor can apply
    pub max_force: f32,
    /// Motor stiffness (spring constant)
    pub stiffness: f32,
    /// Motor damping coefficient
    pub damping: f32,
}

/// Comprehensive joint configuration structure supporting all customizable properties
/// for Avian 2D joints with advanced features and full API coverage
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct JointConfiguration {
    /// Current selected joint type
    pub joint_type: JointType,
    /// Common configuration shared by all joint types
    pub common: CommonJointConfig,
    /// FixedJoint specific configuration
    pub fixed: FixedJointConfig,
    /// DistanceJoint specific configuration
    pub distance: DistanceJointConfig,
    /// PrismaticJoint specific configuration
    pub prismatic: PrismaticJointConfig,
    /// RevoluteJoint specific configuration
    pub revolute: RevoluteJointConfig,
    /// Advanced joint components and features
    pub advanced: AdvancedJointConfig,
}

/// Enum containing all joint type configurations with full property support
#[derive(Clone, Debug, Reflect)]
pub enum JointConfigurationEnum {
    /// FixedJoint configuration with full XPBD constraint support
    Fixed {
        common: CommonJointConfig,
        config: FixedJointConfig,
        advanced: AdvancedJointConfig,
    },
    /// DistanceJoint configuration with limits and compliance
    Distance {
        common: CommonJointConfig,
        config: DistanceJointConfig,
        advanced: AdvancedJointConfig,
    },
    /// PrismaticJoint configuration with sliding axis and limits
    Prismatic {
        common: CommonJointConfig,
        config: PrismaticJointConfig,
        advanced: AdvancedJointConfig,
    },
    /// RevoluteJoint configuration with rotation limits and basis
    Revolute {
        common: CommonJointConfig,
        config: RevoluteJointConfig,
        advanced: AdvancedJointConfig,
    },
}

impl Default for JointConfigurationEnum {
    fn default() -> Self {
        Self::Distance {
            common: CommonJointConfig::default(),
            config: DistanceJointConfig::default(),
            advanced: AdvancedJointConfig::default(),
        }
    }
}

impl JointConfigurationEnum {
    /// Get common properties for current configuration
    pub fn common(&self) -> &CommonJointConfig {
        match self {
            JointConfigurationEnum::Fixed { common, .. } => common,
            JointConfigurationEnum::Distance { common, .. } => common,
            JointConfigurationEnum::Prismatic { common, .. } => common,
            JointConfigurationEnum::Revolute { common, .. } => common,
        }
    }

    /// Get advanced configuration for current joint type
    pub fn advanced(&self) -> &AdvancedJointConfig {
        match self {
            JointConfigurationEnum::Fixed { advanced, .. } => advanced,
            JointConfigurationEnum::Distance { advanced, .. } => advanced,
            JointConfigurationEnum::Prismatic { advanced, .. } => advanced,
            JointConfigurationEnum::Revolute { advanced, .. } => advanced,
        }
    }

    /// Create physics joint with full configuration support using the new Avian API
    pub fn create_physics_joint(
        &self,
        commands: &mut Commands,
        local_anchor_1: Vec2,
        local_anchor_2: Vec2,
        entity1: Entity,
        entity2: Entity,
    ) -> Entity {
        match self {
            JointConfigurationEnum::Fixed {
                common,
                config,
                advanced,
            } => {
                let mut joint = FixedJoint::new(entity1, entity2);

                // Apply anchor configuration
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                // Apply compliance settings for XPBD constraints
                if config.point_compliance != 0.0 {
                    joint = joint.with_point_compliance(config.point_compliance);
                }
                if config.angle_compliance != 0.0 {
                    joint = joint.with_angle_compliance(config.angle_compliance);
                }

                // Build entity with all components
                let mut entity_commands = commands.spawn(joint);

                // Add damping if configured
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    entity_commands.insert(JointDamping {
                        linear: common.damping_linear,
                        angular: common.damping_angular,
                    });
                }

                // Add advanced components
                Self::add_advanced_components(&mut entity_commands, common, advanced);

                entity_commands.id()
            }

            JointConfigurationEnum::Distance {
                common,
                config,
                advanced,
            } => {
                let mut joint = DistanceJoint::new(entity1, entity2);

                // Apply anchor configuration
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                // Apply compliance for distance constraint
                if config.compliance != 0.0 {
                    joint = joint.with_compliance(config.compliance);
                }

                // Apply distance limits - use new API
                if let Some(min) = config.min_distance {
                    joint = joint.with_min_distance(min);
                }
                if let Some(max) = config.max_distance {
                    joint = joint.with_max_distance(max);
                }
                // Use combined limits for more precise control
                if let (Some(min), Some(max)) = (config.min_distance, config.max_distance) {
                    joint = joint.with_limits(min, max);
                }

                // Build entity with all components
                let mut entity_commands = commands.spawn(joint);

                // Add damping if configured
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    entity_commands.insert(JointDamping {
                        linear: common.damping_linear,
                        angular: common.damping_angular,
                    });
                }

                // Add advanced components
                Self::add_advanced_components(&mut entity_commands, common, advanced);

                entity_commands.id()
            }

            JointConfigurationEnum::Prismatic {
                common,
                config,
                advanced,
            } => {
                let mut joint = PrismaticJoint::new(entity1, entity2);

                // Apply anchor configuration
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                // Apply sliding axis
                if config.free_axis != Vec2::X {
                    joint = joint.with_slider_axis(config.free_axis);
                }

                // Apply compliance settings for different constraint types
                if config.axis_compliance != 0.0 {
                    joint = joint.with_align_compliance(config.axis_compliance);
                }
                if config.limit_compliance != 0.0 {
                    joint = joint.with_limit_compliance(config.limit_compliance);
                }
                if config.angle_compliance != 0.0 {
                    joint = joint.with_angle_compliance(config.angle_compliance);
                }

                // Apply translation limits
                if let (Some(min), Some(max)) = (config.min_distance, config.max_distance) {
                    joint = joint.with_limits(min, max);
                }

                // Build entity with all components
                let mut entity_commands = commands.spawn(joint);

                // Add damping if configured
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    entity_commands.insert(JointDamping {
                        linear: common.damping_linear,
                        angular: common.damping_angular,
                    });
                }

                // Add advanced components
                Self::add_advanced_components(&mut entity_commands, common, advanced);

                entity_commands.id()
            }

            JointConfigurationEnum::Revolute {
                common,
                config,
                advanced,
            } => {
                let mut joint = RevoluteJoint::new(entity1, entity2);

                // Apply anchor configuration
                joint = joint.with_local_anchor1(local_anchor_1);
                joint = joint.with_local_anchor2(local_anchor_2);

                // Apply basis rotation for orientation
                if let Some(basis) = config.basis {
                    joint = joint.with_basis(basis);
                }

                // Apply compliance settings
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

                // Build entity with all components
                let mut entity_commands = commands.spawn(joint);

                // Add damping if configured
                if common.damping_linear != 1.0 || common.damping_angular != 1.0 {
                    entity_commands.insert(JointDamping {
                        linear: common.damping_linear,
                        angular: common.damping_angular,
                    });
                }

                // Add advanced components
                Self::add_advanced_components(&mut entity_commands, common, advanced);

                entity_commands.id()
            }
        }
    }

    /// Add advanced joint components based on configuration
    fn add_advanced_components(
        entity_commands: &mut EntityCommands,
        common: &CommonJointConfig,
        advanced: &AdvancedJointConfig,
    ) {
        // Add collision disabling if configured
        if common.disable_collision {
            entity_commands.insert(JointCollisionDisabled);
        }

        // Add joint disabling component if needed
        if advanced.disabled {
            entity_commands.insert(JointDisabled);
        }

        // Add force tracking for breakable joints
        if advanced.track_forces {
            entity_commands.insert(JointForces::default());
        }

        // Add breakable joint configuration
        if advanced.breakable {
            entity_commands.insert(BreakableJoint {
                break_force: advanced.break_force,
                break_torque: advanced.break_torque,
            });
        }

        // Add motor if configured
        if advanced.motor_enabled {
            entity_commands.insert(JointMotor {
                target_velocity: advanced.motor_target_velocity,
                max_force: advanced.motor_max_force,
                stiffness: advanced.motor_stiffness,
                damping: advanced.motor_damping,
            });
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
            advanced: AdvancedJointConfig::default(),
        }
    }
}

/// Common configuration shared by all joint types
#[derive(Clone, Debug, Default, Reflect)]
pub struct CommonJointConfig {
    /// Linear velocity damping coefficient
    pub damping_linear: f32,
    /// Angular velocity damping coefficient
    pub damping_angular: f32,
    /// Disable collision between connected bodies
    pub disable_collision: bool,
}

/// Advanced joint configuration for modern Avian features
#[derive(Clone, Debug, Default, Reflect)]
pub struct AdvancedJointConfig {
    /// Whether the joint is temporarily disabled
    pub disabled: bool,
    /// Track forces applied to joint for breakable joints
    pub track_forces: bool,
    /// Enable breakable joint functionality
    pub breakable: bool,
    /// Force threshold for breaking the joint (Newtons)
    pub break_force: f32,
    /// Torque threshold for breaking the joint (Newton-meters)
    pub break_torque: f32,
    /// Enable motor functionality
    pub motor_enabled: bool,
    /// Target velocity for motor control
    pub motor_target_velocity: f32,
    /// Maximum force motor can apply
    pub motor_max_force: f32,
    /// Motor stiffness (spring constant)
    pub motor_stiffness: f32,
    /// Motor damping coefficient
    pub motor_damping: f32,
}

/// FixedJoint configuration - locks relative position and rotation
#[derive(Clone, Debug, Default, Reflect)]
pub struct FixedJointConfig {
    /// Position constraint compliance (inverse stiffness, m/N)
    pub point_compliance: f32,
    /// Angular constraint compliance (inverse stiffness, N⋅m/rad)
    pub angle_compliance: f32,
}

/// DistanceJoint configuration - maintains distance between anchor points
#[derive(Clone, Debug, Default, Reflect)]
pub struct DistanceJointConfig {
    /// Distance constraint compliance (inverse stiffness, m/N)
    pub compliance: f32,
    /// Target rest length for the joint
    pub rest_length: f32,
    /// Minimum distance limit (optional)
    pub min_distance: Option<f32>,
    /// Maximum distance limit (optional)
    pub max_distance: Option<f32>,
}

/// PrismaticJoint configuration - allows sliding along a specific axis
#[derive(Clone, Debug, Default, Reflect)]
pub struct PrismaticJointConfig {
    /// Free movement axis (default: Vec2::X)
    pub free_axis: Vec2,
    /// Axis alignment compliance for slider orientation
    pub axis_compliance: f32,
    /// Translation limit compliance
    pub limit_compliance: f32,
    /// Angular constraint compliance (prevents rotation)
    pub angle_compliance: f32,
    /// Minimum translation distance
    pub min_distance: Option<f32>,
    /// Maximum translation distance
    pub max_distance: Option<f32>,
}

/// RevoluteJoint configuration - allows rotation around anchor point
#[derive(Clone, Debug, Default, Reflect)]
pub struct RevoluteJointConfig {
    /// Position constraint compliance (m/N)
    pub point_compliance: f32,
    /// Angle limit compliance (N⋅m/rad)
    pub limit_compliance: f32,
    /// Basis rotation angle for joint orientation (radians)
    pub basis: Option<f32>,
    /// Minimum angle limit (radians)
    pub min_angle: Option<f32>,
    /// Maximum angle limit (radians)
    pub max_angle: Option<f32>,
}

impl JointConfiguration {
    /// Create default configuration for specified joint type
    pub fn new(joint_type: JointType) -> Self {
        let mut config = Self::default();
        config.joint_type = joint_type;
        config
    }

    /// Convert to enum type configuration with advanced features
    pub fn to_enum(&self) -> JointConfigurationEnum {
        match self.joint_type {
            JointType::Fixed => JointConfigurationEnum::Fixed {
                common: self.common.clone(),
                config: self.fixed.clone(),
                advanced: self.advanced.clone(),
            },
            JointType::Distance => JointConfigurationEnum::Distance {
                common: self.common.clone(),
                config: self.distance.clone(),
                advanced: self.advanced.clone(),
            },
            JointType::Prismatic => JointConfigurationEnum::Prismatic {
                common: self.common.clone(),
                config: self.prismatic.clone(),
                advanced: self.advanced.clone(),
            },
            JointType::Revolute => JointConfigurationEnum::Revolute {
                common: self.common.clone(),
                config: self.revolute.clone(),
                advanced: self.advanced.clone(),
            },
        }
    }

    /// Reset to default configuration for current joint type
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Configure as rigid connection - completely locks bodies together
    pub fn rigid_connection(&mut self) {
        self.joint_type = JointType::Fixed;
        self.fixed.point_compliance = 0.0;
        self.fixed.angle_compliance = 0.0;
        self.common.disable_collision = true; // Usually want to disable collision for rigid connections
    }

    /// Configure as spring connection - elastic behavior with damping
    pub fn spring_connection(&mut self) {
        self.joint_type = JointType::Distance;
        self.distance.compliance = 0.001; // Soft spring
        self.distance.rest_length = 50.0;
        self.common.damping_linear = 0.5;
        self.common.damping_angular = 0.5;
        self.common.disable_collision = false; // Allow collision for springs
    }

    /// Configure as sliding door - constrained linear motion
    pub fn sliding_door(&mut self) {
        self.joint_type = JointType::Prismatic;
        self.prismatic.free_axis = Vec2::X;
        self.prismatic.min_distance = Some(0.0);
        self.prismatic.max_distance = Some(200.0);
        self.prismatic.axis_compliance = 0.0;
        self.prismatic.angle_compliance = 0.0;
        self.prismatic.limit_compliance = 0.0;
        self.common.disable_collision = true; // Doors usually don't collide with frames
    }

    /// Configure as hinge - limited rotation around pivot point
    pub fn hinge(&mut self) {
        self.joint_type = JointType::Revolute;
        self.revolute.min_angle = Some(-std::f32::consts::PI / 4.0); // -45 degrees
        self.revolute.max_angle = Some(std::f32::consts::PI / 4.0); // +45 degrees
        self.revolute.point_compliance = 0.0;
        self.revolute.limit_compliance = 0.0;
        self.common.disable_collision = true; // Hinges usually don't collide
    }

    /// Configure as breakable joint - will break under excessive force
    pub fn breakable_connection(&mut self) {
        self.joint_type = JointType::Distance;
        self.distance.compliance = 0.0001; // Stiff but breakable
        self.distance.rest_length = 100.0;
        self.advanced.breakable = true;
        self.advanced.break_force = 1000.0; // Break at 1000N
        self.advanced.break_torque = 500.0; // Break at 500Nm
        self.advanced.track_forces = true;
        self.common.disable_collision = false;
    }

    /// Configure as motorized joint - powered motion control
    pub fn motorized_hinge(&mut self) {
        self.joint_type = JointType::Revolute;
        self.revolute.min_angle = None; // Unlimited rotation
        self.revolute.max_angle = None;
        self.revolute.point_compliance = 0.0;
        self.advanced.motor_enabled = true;
        self.advanced.motor_target_velocity = 2.0; // 2 rad/s
        self.advanced.motor_max_force = 500.0;
        self.advanced.motor_stiffness = 1000.0;
        self.advanced.motor_damping = 50.0;
        self.common.disable_collision = true;
    }

    /// Configure as suspension - soft distance joint with high damping
    pub fn suspension(&mut self) {
        self.joint_type = JointType::Distance;
        self.distance.compliance = 0.01; // Very soft
        self.distance.rest_length = 80.0;
        self.distance.min_distance = Some(20.0);
        self.distance.max_distance = Some(120.0);
        self.common.damping_linear = 2.0; // High damping
        self.common.damping_angular = 1.0;
        self.common.disable_collision = false;
    }

    /// Configure as constraint rope - one-way distance limit
    pub fn rope_constraint(&mut self) {
        self.joint_type = JointType::Distance;
        self.distance.compliance = 0.0; // Rigid limit
        self.distance.max_distance = Some(150.0); // Maximum rope length
        self.distance.min_distance = None; // No minimum constraint
        self.common.damping_linear = 0.1;
        self.common.damping_angular = 0.5;
        self.common.disable_collision = false;
    }
}
