use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::scene::DynamicSceneBuilder;
use rfd::FileDialog;
use std::path::PathBuf;
use thiserror::Error;

/// Scene export errors
#[derive(Error, Debug)]
pub enum SceneExportError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("No physics entities found")]
    NoEntitiesFound,
    #[error("Type registry not available")]
    TypeRegistryNotAvailable,
}

/// Scene export plugin
pub struct SceneExportPlugin;

impl Plugin for SceneExportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExportSceneEvent>()
            .add_systems(Update, handle_scene_export);
    }
}

/// Export scene event
#[derive(Event)]
pub enum ExportSceneEvent {
    /// Export specified entities
    Entities(Vec<Entity>),
    /// Export only collider entities
    CollidersOnly,
    /// Export only joint entities
    JointsOnly,
    /// Export all physics entities (colliders and joints)
    All,
}

/// Handle scene export events
fn handle_scene_export(
    mut events: EventReader<ExportSceneEvent>,
    world: &World,
    type_registry: Res<AppTypeRegistry>,
    collider_query: Query<Entity, With<Collider>>,
    joint_query: Query<
        Entity,
        Or<(
            With<FixedJoint>,
            With<DistanceJoint>,
            With<RevoluteJoint>,
            With<PrismaticJoint>,
        )>,
    >,
) {
    for event in events.read() {
        let entities = match event {
            ExportSceneEvent::Entities(entities) => entities.clone(),
            ExportSceneEvent::CollidersOnly => collider_query.iter().collect(),
            ExportSceneEvent::JointsOnly => joint_query.iter().collect(),
            ExportSceneEvent::All => {
                let mut entities: Vec<Entity> = Vec::new();

                // Add collider entities
                for entity in collider_query.iter() {
                    entities.push(entity);
                }

                // Add joint entities
                for entity in joint_query.iter() {
                    if !entities.contains(&entity) {
                        entities.push(entity);
                    }
                }

                entities
            }
        };

        // Show file dialog and save to file
        let dialog = FileDialog::new()
            .add_filter("Scene files", &["ron", "scn"])
            .add_filter("All files", &["*"])
            .set_title("Save Scene As");

        if let Some(file_path) = dialog.save_file() {
            match export_entities_to_file(&entities, world, &type_registry, &file_path) {
                Ok(()) => {
                    info!(
                        "Scene exported successfully to {:?} with {} entities",
                        file_path,
                        entities.len()
                    );
                }
                Err(e) => {
                    error!("Scene export failed: {}", e);
                }
            }
        } else {
            // User cancelled the dialog
            info!("Scene export cancelled by user");
        }
    }
}

/// Export specific entities to file
pub fn export_entities_to_file(
    entities: &[Entity],
    world: &World,
    type_registry: &AppTypeRegistry,
    file_path: &PathBuf,
) -> Result<(), SceneExportError> {
    if entities.is_empty() {
        return Err(SceneExportError::NoEntitiesFound);
    }

    // Create scene builder and extract entities
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(entities.iter().copied())
        .deny_component::<DebugRender>()
        .build();

    // Serialize to RON format
    let registry = type_registry.read();
    let scene_data = scene
        .serialize(&registry)
        .map_err(|e| SceneExportError::SerializationError(e.to_string()))?;

    // Write to file
    std::fs::write(file_path, scene_data)
        .map_err(|e| SceneExportError::SerializationError(e.to_string()))?;

    Ok(())
}
