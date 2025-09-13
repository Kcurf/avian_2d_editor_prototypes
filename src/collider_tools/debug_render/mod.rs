//! Debug rendering system for the Avian Physics Editor
//!
//! This module provides consistent visualization for anchors, joints, and other debug elements.

use bevy::prelude::*;

pub mod anchor;
pub mod joint;

pub use anchor::*;
pub use joint::*;

/// Custom gizmo configuration group for editor debug rendering
/// This ensures editor gizmos appear on top of other debug rendering including physics debug
#[derive(Default, Reflect, GizmoConfigGroup)]
#[reflect(Default)]
pub struct EditorGizmoConfigGroup;

/// Plugin for debug rendering functionality
#[derive(Default)]
pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_gizmo_group::<EditorGizmoConfigGroup>()
            .add_systems(Startup, setup)
            .add_plugins((AnchorDebugRenderPlugin, JointDebugRenderPlugin));
    }
}

// Configure smaller depth_bias to ensure joint gizmos render on top
fn setup(mut config_store: ResMut<GizmoConfigStore>) {
    let (gizmo_config, _) = config_store.config_mut::<EditorGizmoConfigGroup>();
    gizmo_config.depth_bias = -0.5; // Smaller than default 0, renders in front
    gizmo_config.line.width = 3.0; // Slightly thicker lines for better visibility
}
