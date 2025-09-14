use bevy::input::mouse::{MouseButton, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::input::egui_wants_any_input;

/// Camera controller resource for managing camera movement and zoom
#[derive(Resource, Reflect)]
#[reflect(Resource, Default)]
pub struct CameraController {
    /// Movement speed in world units per second
    pub move_speed: f32,
    /// Zoom sensitivity factor
    pub zoom_sensitivity: f32,
    /// Minimum zoom level
    pub min_zoom: f32,
    /// Maximum zoom level
    pub max_zoom: f32,
    /// Whether camera movement is enabled
    pub movement_enabled: bool,
    /// Whether camera zoom is enabled
    pub zoom_enabled: bool,
    /// Whether to zoom towards cursor position
    pub zoom_to_cursor: bool,
    /// Speed multiplier when shift is held
    pub speed_multiplier: f32,
    /// Last mouse position for drag calculation
    pub last_mouse_pos: Option<Vec2>,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 500.0,
            zoom_sensitivity: 0.1,
            min_zoom: 0.1,
            max_zoom: 10.0,
            movement_enabled: true,
            zoom_enabled: true,
            zoom_to_cursor: true,
            speed_multiplier: 2.0,
            last_mouse_pos: None,
        }
    }
}

/// Plugin for 2D camera control functionality
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraController>()
            .register_type::<CameraController>()
            .add_systems(
                Update,
                (camera_movement, camera_zoom, camera_drag_movement)
                    .run_if(not(egui_wants_any_input)),
            );
    }
}

/// System for handling camera movement with arrow keys
fn camera_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_controller: Res<CameraController>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if !camera_controller.movement_enabled {
        return;
    }

    let mut direction = Vec2::ZERO;

    // Only support arrow keys, no WASD
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    if direction != Vec2::ZERO {
        let direction = direction.normalize();
        let mut speed = camera_controller.move_speed;

        // Apply speed multiplier when shift is held
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
        {
            speed *= camera_controller.speed_multiplier;
        }

        let delta = direction * speed * time.delta_secs();

        for mut transform in &mut camera_query {
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        }
    }
}

/// System for handling camera movement with right mouse button drag
fn camera_drag_movement(
    mouse_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_controller: ResMut<CameraController>,
    mut camera_query: Query<(&Camera, &mut Transform, &Projection), With<Camera2d>>,
) {
    if !camera_controller.movement_enabled {
        return;
    }

    let Ok(window) = primary_window.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        camera_controller.last_mouse_pos = None;
        return;
    };

    if mouse_input.just_pressed(MouseButton::Right) {
        camera_controller.last_mouse_pos = Some(cursor_pos);
    } else if mouse_input.pressed(MouseButton::Right) {
        if let Some(last_pos) = camera_controller.last_mouse_pos {
            let delta = cursor_pos - last_pos;

            for (camera, mut transform, projection) in &mut camera_query {
                let Projection::Orthographic(projection) = projection else {
                    continue;
                };

                // Convert screen delta to world delta
                let mut world_delta = Vec2::new(-delta.x, delta.y); // Invert X and Y for natural movement

                // Scale by camera zoom level
                world_delta *= projection.scale;

                // Apply speed multiplier when shift is held
                if keyboard_input.pressed(KeyCode::ShiftLeft)
                    || keyboard_input.pressed(KeyCode::ShiftRight)
                {
                    world_delta *= camera_controller.speed_multiplier;
                }

                // Scale by viewport size to normalize movement
                if let Some(viewport_rect) = camera.logical_viewport_rect() {
                    let viewport_size = viewport_rect.size();
                    world_delta *= projection.area.size() / viewport_size;
                }

                transform.translation.x += world_delta.x;
                transform.translation.y += world_delta.y;
            }
        }

        camera_controller.last_mouse_pos = Some(cursor_pos);
    } else {
        camera_controller.last_mouse_pos = None;
    }
}

/// System for handling camera zoom with mouse wheel
fn camera_zoom(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    camera_controller: Res<CameraController>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&Camera, &mut Transform, &mut Projection), With<Camera2d>>,
) {
    if !camera_controller.zoom_enabled {
        return;
    }

    let mut total_zoom_delta = 0.0;
    for event in mouse_wheel_events.read() {
        total_zoom_delta += event.y;
    }

    if total_zoom_delta == 0.0 {
        return;
    }

    let Ok(window) = primary_window.single() else {
        return;
    };

    for (camera, mut transform, mut projection) in &mut camera_query {
        let Projection::Orthographic(projection) = projection.as_mut() else {
            continue; // Skip non-orthographic projections
        };

        let old_scale = projection.scale;
        projection.scale *= 1.0 - total_zoom_delta * camera_controller.zoom_sensitivity;
        projection.scale = projection
            .scale
            .clamp(camera_controller.min_zoom, camera_controller.max_zoom);

        if camera_controller.zoom_to_cursor {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(viewport_rect) = camera.logical_viewport_rect() {
                    let viewport_size = viewport_rect.size();
                    let cursor_normalized =
                        ((cursor_pos - viewport_rect.min) / viewport_size) * 2.0 - Vec2::ONE;
                    let cursor_normalized = Vec2::new(cursor_normalized.x, -cursor_normalized.y);

                    let cursor_world_pos =
                        transform.translation.truncate() + cursor_normalized * projection.area.max;
                    let proposed_cam_pos = cursor_world_pos
                        - cursor_normalized * projection.area.max * projection.scale / old_scale;

                    transform.translation = proposed_cam_pos.extend(transform.translation.z);
                }
            }
        }
    }
}
