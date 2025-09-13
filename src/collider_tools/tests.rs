//! Tests for collider tools functionality

#[cfg(test)]
mod tests {
    use crate::collider_tools::anchor::AnchorCreationState;
    use crate::collider_tools::utils::find_closest_vertex;
    use avian2d::prelude::*;
    use bevy::prelude::*;

    #[test]
    fn test_find_closest_vertex_rectangle() {
        // Create a rectangle collider
        let collider = Collider::rectangle(2.0, 1.0);
        let transform = Transform::from_xyz(0.0, 0.0, 0.0);
        let global_transform = GlobalTransform::from(transform);

        // Test point near a corner
        let test_point = Vec2::new(1.8, 0.8);
        let closest = find_closest_vertex(test_point, &collider, &global_transform);

        assert!(closest.is_some());
        let closest_point = closest.unwrap();

        // Should be close to the corner at (1.0, 0.5)
        assert!((closest_point.x - 1.0).abs() < 0.1);
        assert!((closest_point.y - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_find_closest_vertex_circle() {
        // Create a circle collider
        let collider = Collider::circle(1.0);
        let transform = Transform::from_xyz(0.0, 0.0, 0.0);
        let global_transform = GlobalTransform::from(transform);

        // Test point
        let test_point = Vec2::new(1.5, 0.0);
        let closest = find_closest_vertex(test_point, &collider, &global_transform);

        assert!(closest.is_some());
        let closest_point = closest.unwrap();

        // Should be one of the 8 circumference points
        assert!((closest_point.length() - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_anchor_creation_state() {
        let state = AnchorCreationState::default();

        // Test initial state
        assert!(!state.ctrl_pressed);
        assert!(!state.shift_pressed);
        assert!(!state.preview_mode);
        assert!(state.preview_position.is_none());
        assert!(state.preview_collider.is_none());
        assert!(state.selected_anchor.is_none());
        assert!(state.mouse_position.is_none());
    }

    #[test]
    fn test_collision_layer_presets_initialization() {
        let mut app = App::new();

        // Add the management plugin which should initialize the resource
        app.add_plugins(crate::collider_tools::collision_layers::CollisionLayerManagementPlugin);

        // Update once to run plugin initialization
        app.update();

        // Check that the resource exists and has default values
        let presets = app
            .world()
            .get_resource::<crate::collider_tools::collision_layers::CollisionLayerPresets>();
        assert!(
            presets.is_some(),
            "CollisionLayerPresets resource should be initialized"
        );

        let presets = presets.unwrap();
        assert!(
            !presets.basic_presets.is_empty(),
            "Basic presets should be initialized"
        );
        assert_eq!(
            presets.layers.len(),
            0,
            "No custom layers should exist by default"
        );
        assert_eq!(
            presets.custom_presets.len(),
            0,
            "No custom presets should exist by default"
        );

        // Check that basic presets are properly initialized
        let preset_names: Vec<String> = presets.get_all_preset_names();
        assert!(
            preset_names.contains(&"DEFAULT".to_string()),
            "DEFAULT preset should exist"
        );
        assert!(
            preset_names.contains(&"ALL".to_string()),
            "ALL preset should exist"
        );
        assert!(
            preset_names.contains(&"NONE".to_string()),
            "NONE preset should exist"
        );
    }
}
