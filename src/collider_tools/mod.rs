use crate::GizmoTransformable;
use crate::selection::Selectable;
use avian2d::prelude::*;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_egui::input::egui_wants_any_keyboard_input;

pub mod anchor;
pub use anchor::*;
pub mod joint;
pub use joint::*;
pub mod creation;
pub mod edit;
pub mod joint_config;
pub mod joint_selection;
// Selection module for collider interaction
pub mod collision_layers;
pub mod debug_render;
pub mod physics_management;
pub mod selection;
pub mod utils;
pub mod visualization;

pub use creation::*;
pub use debug_render::*;
pub use edit::*;
pub use physics_management::*;
pub use selection::*;
pub use visualization::*;

// Export individual plugins for modular usage
pub use anchor::AnchorCreationPlugin;
pub use collision_layers::CollisionLayerManagementPlugin;
pub use creation::CreationPlugin;
pub use edit::EditPlugin;
pub use joint::JointCreationPlugin;
pub use joint_selection::JointSelectionPlugin;
pub use physics_management::PhysicsManagementPlugin;
pub use selection::ColliderSelectionPlugin;

#[cfg(test)]
mod tests;

/// Core plugin for collider tools functionality
///
/// This plugin handles shared resources, state management, and cross-mode systems
/// that are needed by all collider tools plugins.
#[derive(Default)]
pub struct ColliderCorePlugin;

impl Plugin for ColliderCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColliderCreationState>()
            .init_resource::<ColliderEditState>()
            .init_state::<ToolMode>()
            .insert_resource(NextState::Pending(ToolMode::Select))
            // Add systems that need to run across all modes
            .add_systems(
                Update,
                (
                    draw_selected_collider_outlines::<EditorGizmoConfigGroup>,
                    handle_creation_mode_switching.run_if(not(egui_wants_any_keyboard_input)),
                ),
            );
    }
}

/// Plugin group for collider tools functionality
///
/// This plugin group provides comprehensive collider creation, selection, and editing capabilities
/// for the Avian Physics Editor by combining all individual collider tools plugins.
#[derive(Default)]
pub struct ColliderToolsPluginGroup;

impl PluginGroup for ColliderToolsPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ColliderCorePlugin)
            .add(ColliderSelectionPlugin)
            .add(CollisionLayerManagementPlugin)
            .add(CreationPlugin)
            .add(EditPlugin)
            .add(AnchorCreationPlugin)
            .add(JointCreationPlugin)
            .add(JointSelectionPlugin)
            .add(DebugRenderPlugin)
            .add(PhysicsManagementPlugin)
    }
}

/// Current creation mode for the editor
///
/// Determines how mouse input is interpreted and what operations are available.
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ToolMode {
    #[default]
    /// Selection mode - click to select existing colliders
    Select,
    /// Creation mode - click and drag to create new colliders
    Create,
    /// Edit mode - modify existing colliders
    Edit,
    /// Anchor mode - create and manipulate anchor points
    Anchor,
    /// Joint mode - create joints between anchors and center of mass
    Joint,
}

/// Supported collider types for creation
///
/// Each type has different creation behavior and physics properties.
#[derive(Default, Clone, Copy, PartialEq, Debug, Component, Reflect)]
#[require(Selectable, GizmoTransformable)]
#[reflect(Component)]
pub enum ColliderType {
    #[default]
    /// Rectangle collider - defined by width and height
    Rectangle,
    /// Circle collider - defined by radius
    Circle,
    /// Capsule collider - defined by length and radius (pill shape)
    Capsule,
    /// Triangle collider - defined by three vertices
    Triangle,
    /// Polygon collider - convex polygon defined by multiple vertices
    Polygon,
}

/// Stored collider data for undo operations
#[derive(Clone, Debug, Reflect)]
pub struct ColliderData {
    /// Transform of the collider entity
    pub transform: Transform,
    /// Collider component data
    #[reflect(ignore)]
    pub collider: Collider,
    /// Collider type for reconstruction
    pub collider_type: ColliderType,
}

/// Preview collider data for real-time visualization during creation
///
/// Stores temporary data while user is dragging to create a collider.
#[derive(Clone, Reflect)]
pub struct PreviewCollider {
    /// Starting position of the mouse drag (world coordinates)
    pub start_pos: Vec2,
    /// Current mouse position during drag (world coordinates)
    pub current_pos: Vec2,
    /// Type of collider being created
    pub collider_type: ColliderType,
    /// Calculated vertices for visualization
    pub vertices: Vec<Vec2>,
}

pub fn handle_creation_mode_switching(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_mode: Res<State<ToolMode>>,
    mut next_mode: ResMut<NextState<ToolMode>>,
) {
    // Cycle through modes with Tab key
    // Alternative mode switching with Tab key (cycle through modes)
    if keyboard_input.just_pressed(KeyCode::Tab) {
        let current = current_mode.get();
        next_mode.set(match current {
            ToolMode::Select => ToolMode::Create,
            ToolMode::Create => ToolMode::Edit,
            ToolMode::Edit => ToolMode::Anchor,
            ToolMode::Anchor => ToolMode::Joint,
            ToolMode::Joint => ToolMode::Select,
        });
        info!("Mode: {:?} (Tab)", next_mode);
    }
}
