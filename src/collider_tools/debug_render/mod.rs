//! Debug rendering system for the Avian Physics Editor
//!
//! This module provides consistent visualization for anchors, joints, and other debug elements.

use bevy::prelude::*;

pub mod anchor;
pub mod joint;

pub use anchor::*;
pub use joint::*;

/// Plugin for debug rendering functionality
#[derive(Default)]
pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((AnchorDebugRenderPlugin, JointDebugRenderPlugin));
    }
}
