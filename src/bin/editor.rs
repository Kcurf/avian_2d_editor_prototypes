use avian_editor::*;
use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AvianEditorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_mode_switching)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn Camera2d - automatically requires Camera component
    commands.spawn((
        Camera2d,
        SpritePickingCamera,
        GizmoCamera,
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));

    commands.spawn(InfiniteGrid);

    // Spawn a simple ground collider
    commands.spawn((
        Transform::from_xyz(0.0, -200.0, 0.0),
        Collider::rectangle(1000.0, 50.0),
        RigidBody::Static,
        Friction::new(0.7),
        Restitution::new(0.3),
        DebugRender::default(),
        GizmoTransformable,
        Selectable::default(),
    ));

    // Add some interactive objects like in the working example
    for i in 0..3 {
        let x = (i as f32 - 1.0) * 100.0;
        commands.spawn((
            Transform::from_xyz(x, 0.0, 0.0),
            Collider::rectangle(50.0, 50.0),
            RigidBody::Dynamic,
            Friction::new(0.5),
            Restitution::new(0.3),
            DebugRender::default(),
            GizmoTransformable,
            Selectable::default(),
            ColliderType::Rectangle,
        ));
    }
}

fn handle_mode_switching(
    mut creation_properties: ResMut<CreationProperties>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut export_events: EventWriter<ExportSceneEvent>,
    selection: Res<EditorSelection>,
) {
    // Collider type switching with letter keys
    if keyboard.just_pressed(KeyCode::KeyQ) {
        creation_properties.collider_type = ColliderType::Rectangle;
        info!("Collider Type: Rectangle (Q)");
    } else if keyboard.just_pressed(KeyCode::KeyW) {
        creation_properties.collider_type = ColliderType::Circle;
        info!("Collider Type: Circle (W)");
    } else if keyboard.just_pressed(KeyCode::KeyE) {
        creation_properties.collider_type = ColliderType::Capsule;
        info!("Collider Type: Capsule (E)");
    } else if keyboard.just_pressed(KeyCode::KeyR) {
        creation_properties.collider_type = ColliderType::Triangle;
        info!("Collider Type: Triangle (R)");
    } else if keyboard.just_pressed(KeyCode::KeyT) {
        creation_properties.collider_type = ColliderType::Polygon;
        info!("Collider Type: Polygon (T)");
    }

    // Export scene with different combinations
    // Ctrl+E: Export all physics entities
    // Ctrl+Shift+E: Export only colliders
    // Ctrl+Alt+E: Export only joints
    // Ctrl+Shift+Alt+E: Export selected entities only
    if keyboard.just_pressed(KeyCode::KeyE) && keyboard.pressed(KeyCode::ControlLeft) {
        if keyboard.pressed(KeyCode::ShiftLeft) && keyboard.pressed(KeyCode::AltLeft) {
            // Ctrl+Shift+Alt+E: Export selected entities only
            let selected_entities: Vec<Entity> = selection.iter().collect();
            info!("Exporting {} selected entities...", selected_entities.len());
            export_events.write(ExportSceneEvent::Entities(selected_entities));
        } else if keyboard.pressed(KeyCode::ShiftLeft) {
            // Ctrl+Shift+E: Export only colliders
            info!("Exporting colliders only...");
            export_events.write(ExportSceneEvent::CollidersOnly);
        } else if keyboard.pressed(KeyCode::AltLeft) {
            // Ctrl+Alt+E: Export only joints
            info!("Exporting joints only...");
            export_events.write(ExportSceneEvent::JointsOnly);
        } else {
            // Ctrl+E: Export all physics entities
            info!("Exporting all physics entities...");
            export_events.write(ExportSceneEvent::All);
        }
    }
}
