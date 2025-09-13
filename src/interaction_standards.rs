//! Unified Interaction Design Standards for Avian Physics Editor
//!
//! This module defines the standardized interaction patterns and behaviors
//! used throughout the editor to ensure consistent user experience.
//!
//! # Design Principles
//!
//! 1. **Consistency**: All interactive elements follow the same patterns
//! 2. **Predictability**: User actions have expected outcomes
//! 3. **Feedback**: Clear visual and audio feedback for all interactions
//! 4. **Accessibility**: Support for different input methods and preferences
//! 5. **Performance**: Smooth interactions without lag or stuttering

use bevy::prelude::*;
// Removed unused imports

/// Standard interaction settings used across all editor components
#[derive(Resource, Default, Reflect)]
#[reflect(Resource, Default)]
pub struct EditorInteractionStandards {
    /// Standard colors for different interaction states
    pub colors: InteractionColors,
    /// Standard timing values for animations and feedback
    pub timing: InteractionTiming,
    /// Standard sizes and distances for interactive elements
    pub dimensions: InteractionDimensions,
    /// Standard input handling configuration
    pub input: InputStandards,
}

/// Standardized color scheme for interactive elements
#[derive(Reflect, Clone)]
pub struct InteractionColors {
    /// Default color for interactive elements
    pub default: Color,
    /// Color when element is hovered
    pub hover: Color,
    /// Color when element is selected
    pub selected: Color,
    /// Color when element is being dragged
    pub dragging: Color,
    /// Color for disabled elements
    pub disabled: Color,
    /// Color for error states
    pub error: Color,
    /// Color for success states
    pub success: Color,
    /// Color for warning states
    pub warning: Color,
}

impl Default for InteractionColors {
    fn default() -> Self {
        Self {
            default: Color::srgb(0.7, 0.7, 0.7),
            hover: Color::srgb(0.9, 0.9, 0.9),
            selected: Color::srgb(0.2, 0.6, 1.0),
            dragging: Color::srgb(1.0, 0.8, 0.2),
            disabled: Color::srgb(0.4, 0.4, 0.4),
            error: Color::srgb(1.0, 0.3, 0.3),
            success: Color::srgb(0.3, 1.0, 0.3),
            warning: Color::srgb(1.0, 0.8, 0.2),
        }
    }
}

/// Standardized timing values for interactions
#[derive(Reflect, Clone)]
pub struct InteractionTiming {
    /// Duration for hover animations (in seconds)
    pub hover_animation: f32,
    /// Duration for selection animations (in seconds)
    pub selection_animation: f32,
    /// Duration for drag feedback (in seconds)
    pub drag_feedback: f32,
    /// Delay before showing tooltips (in seconds)
    pub tooltip_delay: f32,
    /// Duration for error message display (in seconds)
    pub error_display: f32,
}

impl Default for InteractionTiming {
    fn default() -> Self {
        Self {
            hover_animation: 0.15,
            selection_animation: 0.2,
            drag_feedback: 0.1,
            tooltip_delay: 0.8,
            error_display: 3.0,
        }
    }
}

/// Standardized dimensions for interactive elements
#[derive(Reflect, Clone)]
pub struct InteractionDimensions {
    /// Minimum touch target size (in pixels)
    pub min_touch_target: f32,
    /// Standard control point radius
    pub control_point_radius: f32,
    /// Hover detection tolerance
    pub hover_tolerance: f32,
    /// Drag threshold before starting drag operation
    pub drag_threshold: f32,
    /// Selection outline thickness
    pub selection_outline: f32,
}

impl Default for InteractionDimensions {
    fn default() -> Self {
        Self {
            min_touch_target: 44.0, // iOS/Android standard
            control_point_radius: 6.0,
            hover_tolerance: 3.0,
            drag_threshold: 5.0,
            selection_outline: 2.0,
        }
    }
}

/// Standardized input handling configuration
#[derive(Reflect, Clone)]
pub struct InputStandards {
    /// Enable multi-selection with modifier keys
    pub enable_multi_selection: bool,
    /// Primary modifier key for multi-selection (Shift)
    pub multi_select_modifier: KeyCode,
    /// Secondary modifier key for toggle selection (Ctrl)
    pub toggle_select_modifier: KeyCode,
    /// Key for undo operations
    pub undo_key: KeyCode,
    /// Key for redo operations
    pub redo_key: KeyCode,
    /// Key for delete operations
    pub delete_key: KeyCode,
    /// Key for escape/cancel operations
    pub escape_key: KeyCode,
}

impl Default for InputStandards {
    fn default() -> Self {
        Self {
            enable_multi_selection: true,
            multi_select_modifier: KeyCode::ShiftLeft,
            toggle_select_modifier: KeyCode::ControlLeft,
            undo_key: KeyCode::KeyZ,
            redo_key: KeyCode::KeyY,
            delete_key: KeyCode::Delete,
            escape_key: KeyCode::Escape,
        }
    }
}

/// Standard interaction states for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum InteractionState {
    /// Element is in default state
    Default,
    /// Element is being hovered
    Hovered,
    /// Element is selected
    Selected,
    /// Element is being dragged
    Dragging,
    /// Element is disabled
    Disabled,
    /// Element is in error state
    Error,
}

/// Component for tracking interaction state
#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
pub struct InteractionStateComponent {
    pub current_state: InteractionState,
    pub previous_state: InteractionState,
    pub state_change_time: f32,
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::Default
    }
}

/// Plugin for unified interaction standards
#[derive(Default)]
pub struct InteractionStandardsPlugin;

impl Plugin for InteractionStandardsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorInteractionStandards>()
            .register_type::<EditorInteractionStandards>()
            .register_type::<InteractionColors>()
            .register_type::<InteractionTiming>()
            .register_type::<InteractionDimensions>()
            .register_type::<InputStandards>()
            .register_type::<InteractionState>()
            .register_type::<InteractionStateComponent>()
            .add_systems(
                Update,
                (update_interaction_states, apply_interaction_visual_feedback),
            );
    }
}

/// System to update interaction states based on input
pub fn update_interaction_states(
    mut interaction_query: Query<&mut InteractionStateComponent>,
    time: Res<Time>,
) {
    for mut interaction in interaction_query.iter_mut() {
        if interaction.current_state != interaction.previous_state {
            interaction.previous_state = interaction.current_state;
            interaction.state_change_time = time.elapsed_secs();
        }
    }
}

/// System to apply visual feedback based on interaction states
pub fn apply_interaction_visual_feedback(
    mut gizmos: Gizmos,
    interaction_query: Query<(&InteractionStateComponent, &Transform)>,
    standards: Res<EditorInteractionStandards>,
) {
    for (interaction, transform) in interaction_query.iter() {
        let color = match interaction.current_state {
            InteractionState::Default => standards.colors.default,
            InteractionState::Hovered => standards.colors.hover,
            InteractionState::Selected => standards.colors.selected,
            InteractionState::Dragging => standards.colors.dragging,
            InteractionState::Disabled => standards.colors.disabled,
            InteractionState::Error => standards.colors.error,
        };

        // Apply visual feedback (example: outline)
        let position = transform.translation.truncate();
        gizmos.circle_2d(position, standards.dimensions.control_point_radius, color);
    }
}

/// Utility functions for consistent interaction behavior
pub mod utils {
    use crate::ControlPointType;

    use super::*;

    /// Check if multi-selection modifier is pressed
    pub fn is_multi_select_pressed(
        keyboard: &ButtonInput<KeyCode>,
        standards: &InputStandards,
    ) -> bool {
        keyboard.pressed(standards.multi_select_modifier)
    }

    /// Check if toggle selection modifier is pressed
    pub fn is_toggle_select_pressed(
        keyboard: &ButtonInput<KeyCode>,
        standards: &InputStandards,
    ) -> bool {
        keyboard.pressed(standards.toggle_select_modifier)
    }

    /// Get appropriate color for interaction state with animation
    pub fn get_animated_color(
        state: InteractionState,
        previous_state: InteractionState,
        state_change_time: f32,
        current_time: f32,
        colors: &InteractionColors,
        timing: &InteractionTiming,
    ) -> Color {
        let elapsed = current_time - state_change_time;
        let animation_duration = match state {
            InteractionState::Hovered => timing.hover_animation,
            InteractionState::Selected => timing.selection_animation,
            InteractionState::Dragging => timing.drag_feedback,
            _ => 0.0,
        };

        if elapsed >= animation_duration {
            // Animation complete, return target color
            match state {
                InteractionState::Default => colors.default,
                InteractionState::Hovered => colors.hover,
                InteractionState::Selected => colors.selected,
                InteractionState::Dragging => colors.dragging,
                InteractionState::Disabled => colors.disabled,
                InteractionState::Error => colors.error,
            }
        } else {
            // Interpolate between previous and current color
            let t = elapsed / animation_duration;
            let from_color = match previous_state {
                InteractionState::Default => colors.default,
                InteractionState::Hovered => colors.hover,
                InteractionState::Selected => colors.selected,
                InteractionState::Dragging => colors.dragging,
                InteractionState::Disabled => colors.disabled,
                InteractionState::Error => colors.error,
            };
            let to_color = match state {
                InteractionState::Default => colors.default,
                InteractionState::Hovered => colors.hover,
                InteractionState::Selected => colors.selected,
                InteractionState::Dragging => colors.dragging,
                InteractionState::Disabled => colors.disabled,
                InteractionState::Error => colors.error,
            };

            // Simple linear interpolation
            Color::srgb(
                from_color.to_srgba().red
                    + (to_color.to_srgba().red - from_color.to_srgba().red) * t,
                from_color.to_srgba().green
                    + (to_color.to_srgba().green - from_color.to_srgba().green) * t,
                from_color.to_srgba().blue
                    + (to_color.to_srgba().blue - from_color.to_srgba().blue) * t,
            )
        }
    }

    /// Calculate appropriate control point radius based on type and standards
    pub fn get_control_point_radius(
        point_type: ControlPointType,
        dimensions: &InteractionDimensions,
    ) -> f32 {
        match point_type {
            ControlPointType::RadiusControl | ControlPointType::LengthControl => {
                dimensions.control_point_radius * 1.2
            }
            _ => dimensions.control_point_radius,
        }
    }
}
