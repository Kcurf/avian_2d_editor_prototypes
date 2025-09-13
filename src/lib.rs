use avian2d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::prelude::*;

pub mod camera;
pub mod collider_tools;
pub mod grid;
pub mod interaction_standards;
pub mod scene_export;
pub mod selection;
pub mod transform_gizmos;
pub mod ui;
pub mod utils;

pub use camera::*;
pub use collider_tools::*;
pub use grid::*;
pub use interaction_standards::*;
pub use scene_export::*;
pub use selection::*;
pub use transform_gizmos::*;
pub use ui::*;
pub use utils::*;

// Re-export collision layer UI for easy access
pub use crate::collider_tools::collision_layers::CollisionLayerManagementPlugin;
pub use crate::collider_tools::collision_layers::CollisionLayerPresets;

/// Main plugin for the Avian Editor
///
/// This plugin provides a complete editor interface for Avian physics.
/// It includes collider creation, selection, transformation tools, and more.
///
/// # Required Dependencies
///
/// This plugin requires the following plugins to be registered before it:
/// - `PhysicsPlugins` (from avian2d or avian3d)
/// - `DefaultPlugins` (from bevy)
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use avian2d::prelude::*;
/// use avian_editor::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(PhysicsPlugins::default())
///     .add_plugins(AvianEditorPlugin)
///     .run();
/// ```
pub struct AvianEditorPlugin;

impl Plugin for AvianEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            CoreUtilsPlugin,
            SelectionPlugin,
            InteractionStandardsPlugin,
            ColliderToolsPluginGroup,
            SceneExportPlugin,
            InfiniteGridPlugin,
            CameraControllerPlugin,
            TransformGizmoPlugin,
            EditorUIPlugin,
        ));
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    // pub use crate::collider_tools::selection::ColliderPickable;
    pub use crate::collider_tools::{ColliderCreationState, ColliderType, ToolMode};

    pub use crate::{
        AvianEditorPlugin, ColliderToolsPluginGroup, EditorSelection, InteractionStandardsPlugin,
        PhysicsManagementPlugin, PhysicsManager, Selectable, SelectionPlugin, TransformGizmoPlugin,
    };
}
