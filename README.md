# Avian Physics Editor

**Unofficial 2D Physics Editor Prototype for Avian Physics Engine**

A 2D physics editor built with Bevy 0.16 and the Avian physics engine, providing transform tools, selection systems, collider creation tools, joint creation, and physics editing capabilities.

## Project Overview

This is a hobbyist project implementing a 2D physics editor prototype for the Avian physics engine. **This is NOT an official implementation**, but rather an experimental tool built based on community projects and reference implementations.

## Projects Referenced and Adapted

This project heavily references and adapts from the following projects during its implementation:

### Core Reference Projects

#### 1. **bevy_editor_prototypes** (Primary Reference)
- **Grid System**: Directly adapted the infinite grid rendering implementation from `bevy_editor_prototypes`
- **Transform Gizmos**: Core transform tool code directly copied and modified from `bevy_editor_prototypes`
- **Contributions**: Provided complete grid rendering and transform tool architecture foundation

#### 2. **bevy_egui**
- **UI Framework**: Used as the foundation for the editor user interface
- **Contributions**: Provided integration layer between Bevy and egui

#### 3. **bevy-inspector-egui**
- **Component Inspector**: Used for entity component property editing and inspection
- **Contributions**: Provided powerful runtime component editing functionality

#### 4. **bevy_enoki**
- **Asset Loading**: Referenced asset loading and management implementations
- **Contributions**: Provided asset loading implementation references

#### 5. **bevy_granite**
- **Editor UI Integration**: Referenced actual usage patterns of bevy_egui in Bevy editor implementations
- **Contributions**: Provided practical examples for editor UI integration and workflow implementation

### Core Technology Stack

- **Bevy 0.16**: Game engine framework
- **Avian Physics**: XPBD-based physics engine
- **bevy_egui 0.36**: UI framework
- **bevy-inspector-egui 0.33**: Component inspector
- **serde/ron**: Serialization format
- **rfd**: File dialogs

## Main Features

### Editor Tools
- **Transform Tools**: Move (W), Rotate (E), Scale (R) tools with grid snapping support
- **Selection System**: Single/multi-entity selection with Shift/Ctrl modifier support
- **Physics Controls**: Pause/resume, step, time control
- **Scene Export**: Save and load physics scene configurations

### Collider Tools
- **Creation Tools**: Interactive rectangle and circle collider creation
- **Editing Tools**: Collider property editing and adjustment
- **Collision Layer Management**: Runtime collision layer creation and management
- **Joint Tools**: Interactive joint creation and configuration

### User Interface
- **Internationalization Support**: English/Chinese interface switching
- **Theme System**: Light/dark theme switching
- **Modular Architecture**: Top toolbar, left tool panel, right entity inspector
- **Font Management**: System font loading and fallback font support

## Architecture Design

### Modular Structure
```
src/
├── collider_tools/     # Collider tools group
├── transform_gizmos/   # Transform tools (adapted from bevy_editor_prototypes)
├── grid/              # Grid rendering (adapted from bevy_editor_prototypes)
├── selection/         # Selection system
├── camera/            # Camera controls
├── ui/                # User interface
├── scene_export.rs    # Scene export
└── interaction_standards.rs  # Interaction standards
```

### Plugin Architecture
```rust
pub struct AvianEditorPlugin;

impl Plugin for AvianEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsDebugPlugin::default(),
            PhysicsPlugins::default(),
            // ... other plugins
            EditorUIPlugin,
            SceneExportPlugin,
        ));
    }
}
```

## Building and Running

### Prerequisites
- Rust 2024 Edition
- Cargo package manager

### Building the Project
```bash
# Build the entire workspace
cargo build

# Build only the avian_editor crate
cargo build -p avian_editor
```

## Usage Instructions

### Basic Operations
1. **Selection Mode**: Click entities to select, hold Shift for multi-selection
2. **Transform Tools**: Press W/E/R keys to switch between move/rotate/scale modes
3. **Create Colliders**: Switch to creation mode and drag to create rectangle or circle colliders
4. **Create Joints**: Select two entities, create joints by dragging in anchor mode
5. **Scene Export**: Use top menu to export current scene

### Editor Modes
- **Select Mode**: Entity selection and manipulation
- **Create Mode**: Collider creation
- **Edit Mode**: Collider property editing
- **Anchor Mode**: Joint anchor point setup
- **Joint Mode**: Joint creation and editing

## Development Notes

### Code Adaptation Notes
- **Grid System**: Code structure and rendering logic mainly adapted from `bevy_editor_prototypes`
- **Transform Tools**: Core implementation directly based on modifications from `bevy_editor_prototypes`
- **UI System**: Integrated functionality from `bevy_egui` and `bevy-inspector-egui`
- **Physics Editing**: Designed based on Avian physics engine APIs

### Development Standards
- Follow Bevy 0.16 API changes
- Use ECS architecture patterns
- Modular design with separation of concerns
- Code formatting using `rustfmt`

## Contributing

This is a personal project primarily used for learning and experimentation. Feedback and suggestions are welcome, but please understand this is a prototype implementation.

## License

This project primarily references multiple open-source projects. Please refer to the respective project licenses for specific licensing information. This project code is licensed under the MIT License.

## Disclaimer

**This is NOT an official Avian physics engine editor implementation.** This is a hobbyist project for learning and experimental purposes. The project may contain unstable features and incomplete implementations.

## Contact

For questions or suggestions, please submit through GitHub Issues.
