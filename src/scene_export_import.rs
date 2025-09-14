use avian2d::prelude::*;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::scene::{DynamicScene, DynamicSceneBuilder, serde::SceneDeserializer};

use rfd::FileDialog;
use serde::de::DeserializeSeed;
use std::path::PathBuf;
use thiserror::Error;

use crate::panel_state::{EntityInspectorState, PanelState};
use crate::{CollisionLayerPresets, CreationProperties};

/// Scene export/import plugin
pub struct SceneExportImportPlugin;

impl Plugin for SceneExportImportPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SceneExportEvent>()
            .add_event::<SceneImportEvent>()
            .add_systems(Update, handle_scene_export)
            .add_systems(Update, handle_scene_import);
    }
}

/// Scene export event
#[derive(Event)]
pub enum SceneExportEvent {
    /// Export all entities and resources
    All,
    /// Export specific entities and all resources
    Entities(Vec<Entity>),
}

/// Scene import event
#[derive(Event)]
pub enum SceneImportEvent {
    /// Show file dialog and import
    FromDialog,
    /// Import from specific path
    FromPath(PathBuf),
}

/// Scene export/import errors
#[derive(Error, Debug)]
pub enum SceneError {
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("File error: {0}")]
    FileError(String),
    #[error("No entities found")]
    NoEntitiesFound,
}

/// Command for spawning a dynamic scene
pub struct SpawnDynamicSceneCommand {
    pub scene: DynamicScene,
    pub entity_map: Option<EntityHashMap<Entity>>,
}

impl Command for SpawnDynamicSceneCommand {
    fn apply(self, world: &mut World) {
        let mut entity_map = self.entity_map.unwrap_or_default();

        match self.scene.write_to_world(world, &mut entity_map) {
            Ok(()) => {
                info!(
                    "Scene spawned successfully with {} entities",
                    entity_map.len()
                );

                // You can access the entity_map to work with spawned entities
                for (scene_entity, world_entity) in entity_map.iter() {
                    debug!(
                        "Mapped scene entity {:?} to world entity {:?}",
                        scene_entity, world_entity
                    );
                }
            }
            Err(e) => {
                error!("Failed to spawn scene: {}", e);
            }
        }
    }
}

/// Handle scene export
fn handle_scene_export(
    mut events: EventReader<SceneExportEvent>,
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
        // Get entities to export
        let entities = match event {
            SceneExportEvent::All => {
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
            SceneExportEvent::Entities(entities) => entities.clone(),
        };

        if entities.is_empty() {
            warn!("No entities to export");
            continue;
        }

        // Show file dialog
        let dialog = FileDialog::new()
            .add_filter("Scene files", &["ron", "scn"])
            .add_filter("All files", &["*"])
            .set_title("Save Scene As");

        if let Some(file_path) = dialog.save_file() {
            match export_scene(&entities, world, &type_registry, &file_path) {
                Ok(()) => {
                    info!(
                        "Scene exported successfully to {:?} with {} entities and all resources",
                        file_path,
                        entities.len()
                    );
                }
                Err(e) => {
                    error!("Scene export failed: {}", e);
                }
            }
        }
    }
}

/// Handle scene import
fn handle_scene_import(
    mut events: EventReader<SceneImportEvent>,
    mut commands: Commands,
    type_registry: Res<AppTypeRegistry>,
) {
    for event in events.read() {
        match event {
            SceneImportEvent::FromDialog => {
                let dialog = FileDialog::new()
                    .add_filter("Scene files", &["ron", "scn"])
                    .add_filter("All files", &["*"])
                    .set_title("Load Scene");

                if let Some(file_path) = dialog.pick_file() {
                    match import_scene_with_commands(&file_path, &type_registry, &mut commands) {
                        Ok(()) => {
                            info!("Scene import queued successfully from {:?}", file_path);
                        }
                        Err(e) => {
                            error!("Scene import failed: {}", e);
                        }
                    }
                }
            }
            SceneImportEvent::FromPath(path) => {
                match import_scene_with_commands(path, &type_registry, &mut commands) {
                    Ok(()) => {
                        info!("Scene import queued successfully from {:?}", path);
                    }
                    Err(e) => {
                        error!("Scene import failed: {}", e);
                    }
                }
            }
        }
    }
}

/// Export scene with entities and resources
fn export_scene(
    entities: &[Entity],
    world: &World,
    type_registry: &AppTypeRegistry,
    file_path: &PathBuf,
) -> Result<(), SceneError> {
    // Create scene builder and extract both entities and resources
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(entities.iter().copied())
        .allow_resource::<CollisionLayerPresets>()
        .allow_resource::<PanelState>()
        .allow_resource::<EntityInspectorState>()
        .allow_resource::<CreationProperties>()
        .allow_all_components()
        .extract_resources() // This will extract all resources with ReflectResource
        .build();

    // Serialize to RON format
    let registry = type_registry.read();
    let scene_data = scene
        .serialize(&registry)
        .map_err(|e| SceneError::SerializationError(e.to_string()))?;

    // Write to file
    std::fs::write(file_path, scene_data).map_err(|e| SceneError::FileError(e.to_string()))?;

    Ok(())
}

/// Import scene with entities and resources using commands.queue
fn import_scene_with_commands(
    file_path: &PathBuf,
    type_registry: &AppTypeRegistry,
    commands: &mut Commands,
) -> Result<(), SceneError> {
    // Read file content
    let file_content =
        std::fs::read_to_string(file_path).map_err(|e| SceneError::FileError(e.to_string()))?;

    // Deserialize scene
    let registry = type_registry.read();
    let mut deserializer = ron::de::Deserializer::from_str(&file_content)
        .map_err(|e| SceneError::SerializationError(e.to_string()))?;
    let scene_deserializer = SceneDeserializer {
        type_registry: &registry,
    };
    let dynamic_scene = scene_deserializer
        .deserialize(&mut deserializer)
        .map_err(|e| SceneError::SerializationError(e.to_string()))?;

    info!(
        "Scene deserialized successfully with {} entities and resources",
        dynamic_scene.entities.len()
    );

    // Queue the scene for spawning using commands
    commands.queue(SpawnDynamicSceneCommand {
        scene: dynamic_scene,
        entity_map: None,
    });

    Ok(())
}
