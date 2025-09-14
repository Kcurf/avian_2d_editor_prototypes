use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

// 翻译资源存储
static TRANSLATIONS: Lazy<RwLock<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// 当前语言和回退语言
static CURRENT_LANGUAGE: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("en".to_string()));
static FALLBACK_LANGUAGE: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("en".to_string()));

/// 设置当前语言
pub fn set_language(locale: &str) {
    let mut current_locale = CURRENT_LANGUAGE.write().unwrap();
    *current_locale = locale.to_string();
}

/// 获取当前语言
pub fn get_language() -> String {
    let current_locale = CURRENT_LANGUAGE.read().unwrap();
    current_locale.clone()
}

/// 设置回退语言
pub fn set_fallback(locale: &str) {
    let mut fallback_locale = FALLBACK_LANGUAGE.write().unwrap();
    *fallback_locale = locale.to_string();
}

/// 获取回退语言
pub fn get_fallback() -> String {
    let fallback_locale = FALLBACK_LANGUAGE.read().unwrap();
    fallback_locale.clone()
}

/// 从文本加载翻译
pub fn load_translations_from_text(
    language: impl AsRef<str>,
    content: impl AsRef<str>,
) -> Result<(), String> {
    let translations = parse_translations(content.as_ref().to_string(), true);
    load_translations_from_map(language, translations);
    Ok(())
}

/// 从HashMap加载翻译
pub fn load_translations_from_map(
    language: impl AsRef<str>,
    translations: HashMap<String, String>,
) {
    let mut translations_map = TRANSLATIONS.write().unwrap();
    translations_map.insert(language.as_ref().to_string(), translations);
}

/// 解析翻译文本
fn parse_translations(content: String, clean_empty: bool) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut name = String::default();
    let mut values = vec![];

    for line in content.split("\n") {
        let trimmed_line = line.trim();

        // 跳过注释行 (以 # 开头)
        if trimmed_line.starts_with('#') {
            continue;
        }

        // 跳过空行
        if trimmed_line.is_empty() {
            continue;
        }

        if !line.contains("=") {
            values.push(line.to_string());
            continue;
        }

        if !name.is_empty() {
            let value = values.join("\n").trim().to_string();
            let allow = if value.is_empty() { !clean_empty } else { true };
            if allow {
                map.insert(name, value);
            }
            values.clear();
        }

        if line.contains("\\=") {
            let items_of_escaping: Vec<&str> = line.split("\\=").collect();
            let mut e_names = vec![];
            let mut e_values = vec![];
            let len = items_of_escaping.len();

            for i in 0..len {
                let item = items_of_escaping.get(i).unwrap();
                if item.contains('=') {
                    let (first, second) = item.split_once('=').unwrap();
                    e_names.push(first.trim().to_string());
                    e_values.push(second.trim().to_string());
                    if i + 1 == len {
                        break;
                    }
                    let remain: Vec<String> = items_of_escaping[i + 1..]
                        .iter()
                        .map(|&item| item.to_string())
                        .collect();
                    e_values.extend(remain);
                    break;
                } else {
                    e_names.push(item.trim().to_string());
                }
            }
            name = e_names.join("=");
            values.push(e_values.join("="));
        } else {
            let (first, second) = line.split_once('=').unwrap();
            name = first.trim().to_string();
            values.push(second.trim().to_string());
        }
    }

    if !name.is_empty() {
        let value = values.join("\n").trim().to_string();
        let allow = if value.is_empty() { !clean_empty } else { true };
        if allow {
            map.insert(name, value);
        }
    }

    map
}

/// 格式化翻译文本
fn format(template: &str, args: &HashMap<&str, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in args {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}
/// 翻译函数
pub fn translate(
    language: impl AsRef<str>,
    fallback_language: impl AsRef<str>,
    key: &str,
    args: &HashMap<&str, String>,
) -> String {
    let language = language.as_ref();
    let fallback_language = fallback_language.as_ref();

    if language.is_empty() && fallback_language.is_empty() {
        return key.to_string();
    }

    let language = if language.is_empty() {
        fallback_language
    } else {
        language
    };
    let mut translated = extract_translate(language, key, args);

    if translated.is_none() {
        translated = extract_translate(fallback_language, key, args);
    }

    translated.unwrap_or_else(|| key.to_string())
}

/// 提取翻译
fn extract_translate(
    language: impl AsRef<str>,
    key: &str,
    args: &HashMap<&str, String>,
) -> Option<String> {
    let translations = TRANSLATIONS.read().unwrap();
    if let Some(language_map) = translations.get(language.as_ref()) {
        if let Some(template) = language_map.get(key) {
            if !template.is_empty() {
                return Some(format(template, args));
            }
        }
    }
    None
}

/// 翻译宏
#[macro_export]
macro_rules! tr {
    ($key:expr, {$($name:ident: $val:expr),*}) => {{
        let mut args = std::collections::HashMap::new();
        $(
            args.insert(stringify!($name), $val.to_string());
        )*
        $crate::ui::i18n::translate(
            &$crate::ui::i18n::get_language(),
            &$crate::ui::i18n::get_fallback(),
            $key,
            &args
        )
    }};
    ($key:expr) => {{
        $crate::ui::i18n::translate(
            &$crate::ui::i18n::get_language(),
            &$crate::ui::i18n::get_fallback(),
            $key,
            &std::collections::HashMap::new()
        )
    }};
}

/// 测试注释处理功能
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_translations_with_comments() {
        let test_content = r#"
# 这是一个注释
key1 = value1
key2 = value2

# 这是另一个注释
key3 = value3
    "#;

        let result = parse_translations(test_content.to_string(), true);

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
        assert_eq!(result.get("key3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_parse_translations_with_inline_comments() {
        let test_content = r#"
key1 = value1
key2 = value2 # 这行不是注释，因为#不在开头
key3 = value3
    "#;

        let result = parse_translations(test_content.to_string(), true);

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(
            result.get("key2"),
            Some(&"value2 # 这行不是注释，因为#不在开头".to_string())
        );
        assert_eq!(result.get("key3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_parse_translations_with_empty_lines() {
        let test_content = r#"
key1 = value1

key2 = value2

key3 = value3
    "#;

        let result = parse_translations(test_content.to_string(), true);

        assert_eq!(result.len(), 3);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
        assert_eq!(result.get("key3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_parse_translations_with_hash_in_value() {
        let test_content = r#"
key1 = value1 # 这个#是值的一部分
key2 = value2
    "#;

        let result = parse_translations(test_content.to_string(), true);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get("key1"),
            Some(&"value1 # 这个#是值的一部分".to_string())
        );
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }
}

/// 初始化翻译资源
pub fn init_translations() {
    // 英文翻译
    let en_us = r#"
app_title = Avian Physics Editor
export_scene = 📁 Export Scene
export_all = Export All Physics Entities
export_selected = Export Selected Entities
export_colliders = Export Colliders Only
export_joints = Export Joints Only
resume_physics = ▶ Resume Physics
pause_physics = ⏸ Pause Physics
tool_mode = Tool Mode
mode_select = Select
mode_create = Create
mode_edit = Edit
mode_anchor = Anchor
mode_joint = Joint
transform_gizmo = Transform Gizmo
collider_editor = Collider Editor
anchor_tools = Anchor Tools
joint_settings = Joint Settings
no_entity_selected = No entity selected
selected_entity = Selected Entity
delete_entity = Delete Entity
no_selection = No entity selected
control_points = Control Points
no_point_selected = No point selected
dragging_point = Dragging Point
editing_entity = Editing Entity
created_anchors = Created Anchors
preview_position = Preview Position
selected_anchor = Selected Anchor
preview_mode = Preview Mode (Right Click)
shift_pressed = Shift Pressed (Snap)
clear_all = Clear All
entity_inspector = Entity Inspector
inspector_mode = Inspector Mode
component_management = Component Management
component_inspector = Component Inspector
shape_edit = Shape Edit

# Shape editing translations
shape_properties = Shape Properties
no_collider_for_shape_edit = No collider component found for shape editing
missing_required_components = Missing required components for shape editing
rectangle_properties = Rectangle Properties
width = Width
height = Height
preset_sizes = Preset Sizes
square_small = Small Square
square_medium = Medium Square
square_large = Large Square
rectangle_wide = Wide Rectangle
rectangle_tall = Tall Rectangle
rectangle_wide_large = Large Wide Rectangle
circle_properties = Circle Properties
radius = Radius
diameter = Diameter
preset_radii = Preset Radii
radius_small = Small Radius
radius_medium = Medium Radius
radius_large = Large Radius
radius_extra_large = Extra Large Radius
radius_huge = Huge Radius
capsule_properties = Capsule Properties
height = Height
rotation = Rotation
preset_capsules = Preset Capsules
capsule_pill = Pill
capsule_tall = Tall Capsule
capsule_wide = Wide Capsule
triangle_properties = Triangle Properties
triangle_side_lengths = Triangle Side Lengths
triangle_radius = Triangle Circumradius
triangle_angles = Triangle Angles
side_ab = Side A-B (opposite C)
side_bc = Side B-C (opposite A)
side_ca = Side C-A (opposite B)
angle_at_a = Angle at A (°)
angle_at_b = Angle at B (°)
angle_at_c = Angle at C (°)
triangle_vertices = Triangle Vertices
vertex = Vertex
apply_triangle_changes = Apply Changes
preset_triangles = Preset Triangles
equilateral_triangle = Equilateral
right_triangle = Right Triangle
polygon_properties = Polygon Properties
vertex_count = Vertex Count
edit_vertices = Edit Vertices
polygon_too_complex = Polygon too complex for manual editing
polygon_edit_warning = Complex polygon editing not yet implemented
preset_polygons = Preset Polygons
pentagon = Pentagon
hexagon = Hexagon
octagon = Octagon
transform_properties = Transform Properties
position = Position
scale = Scale
reset_transform = Reset Transform
invalid_triangle_shape = Invalid triangle shape
invalid_polygon_shape = Invalid polygon shape
remove_components = Remove Components
add_components = Add Components
selection_controls = Selection Controls
clear_selection = Clear Selection
select_all = Select All
joint_type = Select joint type
distance = Distance
revolute = Revolute
prismatic = Prismatic
fixed = Fixed
presets = Quick Presets
rigid = Rigid
spring = Spring
sliding = Sliding Door
hinge = Hinge
reset_defaults = Reset to Defaults
linear_damping = Linear Damping
angular_damping = Angular Damping
disable_collision = Disable Collision
point_compliance = Point Compliance
angle_compliance = Angle Compliance
limit_compliance = Limit Compliance
rest_length = Rest Length
distance_limits = Distance Limits
min = Min
max = Max
free_axis = Free Axis
axis_compliance = Axis Compliance
angle_limits = Angle Limits
joint_instructions = Joint Creation Instructions
click_drag = Click and drag between anchors/entities
configure_properties = Configure properties above
joint_created = Joint will be created on release
currently_dragging = 🖱️ Currently dragging...
distance_label = Distance
creation_controls = Creation Controls
creation_instruction_1 = Click and drag to draw rectangle collider
creation_instruction_2 = Right click to draw circle collider
creation_instruction_3 = Select collider type from dropdown
creation_instruction_4 = Configure properties in panel
creation_instruction_5 = Press Enter to create collider
edit_instruction_1 = Drag control points to modify shape
edit_instruction_2 = Click control point to select
edit_instruction_3 = Delete key to remove selected point
edit_instruction_4 = Ctrl+Z to undo changes
edit_instruction_5 = Ctrl+Y to redo changes
anchor_instruction_1 = Click on collider to create anchor
anchor_instruction_2 = Shift+click for multiple anchors
anchor_instruction_3 = Delete key to remove anchor
anchor_instruction_4 = Clear button to remove all
anchor_instruction_5 = Tab key to switch to joint mode
anchor_instruction_6 = Drag between anchors to connect
anchor_instruction_7 = Preview position before placement
anchor_instruction_8 = Enter key to confirm placement
joint_instruction_1 = Drag between anchors to create joint
joint_instruction_2 = Select joint type from dropdown
joint_instruction_3 = Configure joint properties
selection_instruction_1 = Click on entity to select
selection_instruction_2 = Shift+click for multi-select
selection_instruction_3 = W key for move mode
selection_instruction_4 = E key for rotate mode
selection_instruction_5 = R key for scale mode
collider_type = Collider Type
body_type = Body Type
color = Color
basic_config = Basic Configuration
presets_config = Presets Configuration
mass_properties = Mass Properties
material_properties = Material Properties
motion_properties = Motion Properties
collision_properties = Collision Properties
performance_properties = Performance Properties
advanced_physics = Advanced Physics
rectangle = Rectangle
circle = Circle
capsule = Capsule
triangle = Triangle
polygon = Polygon
static = Static
dynamic = Dynamic
kinematic = Kinematic
character_controller = Character Controller
high_speed_object = High Speed Object
bouncy_ball = Bouncy Ball
static_platform = Static Platform
sensor = Sensor
physics_prop = Physics Prop
vehicle = Vehicle
anti_gravity = Anti Gravity
destructible = Destructible
explicit_mass = Explicit Mass
use_explicit_mass = Use Explicit Mass
auto_calculated = Auto Calculated
density = Density
explicit_angular_inertia = Explicit Angular Inertia
use_explicit_angular_inertia = Use Explicit Angular Inertia
explicit_center_of_mass = Explicit Center of Mass
use_explicit_center_of_mass = Use Explicit Center of Mass
mass_properties_control = Mass Properties Control
disable_auto_mass = Disable Auto Mass
disable_auto_angular_inertia = Disable Auto Angular Inertia
disable_auto_center_of_mass = Disable Auto Center of Mass
friction = Friction
use_explicit_friction = Use Explicit Friction
global_default = Global Default
static_friction = Static Friction
use_explicit_static_friction = Use Explicit Static Friction
use_dynamic_friction = Use Dynamic Friction
restitution = Restitution
use_explicit_restitution = Use Explicit Restitution
linear_velocity = Linear Velocity
angular_velocity = Angular Velocity
collision_layers = Collision Layers
explicit_linear_damping = Explicit Linear Damping
use_explicit_linear_damping = Use Explicit Linear Damping
explicit_angular_damping = Explicit Angular Damping
use_explicit_angular_damping = Use Explicit Angular Damping
gravity_scale = Gravity Scale
use_explicit_gravity_scale = Use Explicit Gravity Scale
max_linear_speed = Max Linear Speed
limit_linear_speed = Limit Linear Speed
no_limit = No Limit
max_angular_speed = Max Angular Speed
limit_angular_speed = Limit Angular Speed
locked_axes = Locked Axes
lock_x = Lock X
lock_y = Lock Y
lock_rotation = Lock Rotation
dominance = Dominance
use_dominance = Use Dominance
default = Default
sensor_no_collision = Sensor (No Collision Response)
collision_margin = Collision Margin
speculative_margin = Speculative Margin (CCD)
use_speculative_margin = Use Speculative Margin
swept_ccd = Enable Swept CCD
collision_events = Enable Collision Events
collider_disabled = Disable Collider
disable_sleeping = Disable Sleeping
disable_physics = Disable Physics Simulation
transform_interpolation = Enable Transform Interpolation
constant_force_world = Constant Force (World Space)
use_constant_force = Use Constant Force
none = None
constant_force_local = Constant Force (Local Space)
use_constant_local_force = Use Constant Local Force
constant_torque = Constant Torque
use_constant_torque = Use Constant Torque
constant_linear_acceleration_world = Constant Linear Acceleration (World Space)
use_constant_linear_acceleration = Use Constant Linear Acceleration
constant_linear_acceleration_local = Constant Linear Acceleration (Local Space)
use_constant_local_linear_acceleration = Use Constant Local Linear Acceleration
constant_angular_acceleration = Constant Angular Acceleration
use_constant_angular_acceleration = Use Constant Angular Acceleration
config_updated = ✓ Configuration updated
anchor_controls = Anchor Controls
create_select_anchor = Left Click: Create/Select anchor
move_anchor = Drag: Move selected anchor
toggle_preview = Right Click: Toggle preview mode
snap_vertices = Shift: Snap to vertices
precise_placement = Ctrl: Precise placement
remove_anchor = Delete: Remove selected anchor
switch_tools = Tab: Switch to other tools
edit_controls = Edit Controls
drag_control_points = Click and drag control points to modify shape
undo = Ctrl+Z: Undo changes
redo = Ctrl+Y: Redo changes
cancel_editing = Escape: Cancel editing
selection_instructions = Selection Controls
left_click_select = Left click: Select entity
shift_click_multi = Shift+click: Multi-select
ctrl_click_toggle = Ctrl+click: Toggle selection
drag_select = Drag select: Box selection
delete_entities = Delete: Remove selected entities
gizmo_mode = Gizmo Mode
translate_w = Translate (W)
rotate_e = Rotate (E)
scale_r = Scale (R)
snap_settings = Snap Settings
enable_snapping = Enable Snapping
angle_snap = Angle Snap
scale_snap = Scale Snap
center_origin = Center to Origin
# Missing translations for hardcoded strings and tr! keys
language_switcher = 🌐 Language
english = English
chinese = 中文
# Transform gizmo modes
translate_mode = Translate Mode
rotate_mode = Rotate Mode
scale_mode = Scale Mode
# Transform gizmo controls
center_to_origin = Center to Origin
# Anchor controls
quick_actions = Quick Actions
multiple = Multiple
connect = Connect
confirm = Confirm
# Joint creation
select_type = Select Type
# Material properties
static_friction_coefficient = Static Friction Coefficient
use_dynamic_friction = Use Dynamic Friction
# Collision detection
speculative_margin_ccd = Speculative Margin (CCD)
# Additional missing keys
# Shape operations
add_vertex = Add Vertex
modify = Modify
# Workflow controls
switch_mode = Switch Mode
# Joint creation
joint_creation_instructions = Joint Creation Instructions
create_joint = Create Joint
currently_dragging = Currently Dragging
distance = Distance
# Presets
quick_presets = Quick Presets
# Material properties
friction_coefficient = Friction Coefficient
# Motion properties
dominance_value = Dominance Value
# Collision detection
sensor_no_collision_response = Sensor (No Collision Response)
preset_selector = Preset Selector
no_layers_defined = No layers defined
available_layers = Available Layers
layer_name = Layer Name
description = Description
add_new_layer = Add New Layer
save_as_preset = Save as Preset
detailed_config = Detailed Configuration
member_layers = Member Layers
filter_layers = Filter Layers
basic_preset = Basic
collision_layer_management = Collision Layer Management
enable_swept_ccd = Enable Swept CCD
enable_collision_events = Enable Collision Events
disable_collider = Disable Collider
# Physics settings
disable_physics_simulation = Disable Physics Simulation
enable_transform_interpolation = Enable Transform Interpolation
# Physics forces
constant_force_world_space = Constant Force (World Space)
constant_force_local_space = Constant Force (Local Space)
constant_linear_acceleration_world_space = Constant Linear Acceleration (World Space)
constant_linear_acceleration_local_space = Constant Linear Acceleration (Local Space)
# Gizmo operations
gizmo_operations = Gizmo Operations
move_axis = Move Axis
rotate = Rotate
scale = Scale
snapping = Snapping
# Configuration properties
linear = Linear
angular = Angular
point = Point
angle = Angle
align = Align
limit = Limit
axis = Axis
compliance = Compliance
min = Min
max = Max
# Component management
add_name = Add Name
add_transform = Add Transform
add_visibility = Add Visibility
add_rigid_body = Add Rigid Body
add_circle_collider = Add Circle Collider
add_linear_velocity = Add Linear Velocity
add_angular_velocity = Add Angular Velocity
add_collision_layers = Add Collision Layers
delete = Delete
despawn_entity_tooltip = Despawn entity
remove_component_tooltip = Remove component
new_entity = New Entity
# Joint types
distance_joint = Distance Joint
revolute_joint = Revolute Joint
prismatic_joint = Prismatic Joint
fixed_joint = Fixed Joint
# Joint properties
distance_joint_properties = Distance Joint Properties
revolute_joint_properties = Revolute Joint Properties
prismatic_joint_properties = Prismatic Joint Properties
fixed_joint_properties = Fixed Joint Properties
# Presets
rigid_preset = Rigid
spring_preset = Spring
sliding_preset = Sliding
hinge_preset = Hinge
breakable_preset = Breakable
motorized_preset = Motorized
suspension_preset = Suspension
rope_preset = Rope
# Advanced joint properties
advanced_properties = Advanced Properties
breakable_joint = Breakable Joint
breakable_settings = Breakable Settings
break_force = Break Force
break_torque = Break Torque
breakable_description = Joint will break when forces exceed thresholds
joint_motor = Joint Motor
motor_settings = Motor Settings
target_velocity = Target Velocity
max_force = Max Force
motor_stiffness = Motor Stiffness
motor_damping = Motor Damping
motor_description = Automatic motor with velocity and force control
force_tracking = Force Tracking
force_tracking_settings = Force Tracking Settings
track_forces = Track Forces
force_tracking_description = Monitor forces acting on the joint
advanced_physics = Advanced Physics
joint_disable_settings = Joint Disable Settings
disable_collision = Disable Collision
disable_on_break = Disable on Break
advanced_physics_description = Advanced joint behavior and physics settings
disable_joint = Disable Joint
# Help text updates
presets = Presets
advanced_features = Advanced Features
# Values and defaults
global_default_0_0 = Global Default (0.0)
global_default_0_5 = Global Default (0.5)
global_default_1_0 = Global Default (1.0)
# State and status
edit_state_unavailable = Edit state unavailable
anchor_state_unavailable = Anchor state unavailable
joint_state_unavailable = Joint state unavailable
joint_configuration_unavailable = Joint configuration unavailable
creation_properties_unavailable = Creation properties unavailable
no_entity_selected_instruction = No entity selected. Click on an entity to inspect its components.
selected = Selected
# Help sections
drawing_controls = Drawing Controls
shape_configuration = Shape Configuration
advanced_shapes = Advanced Shapes
point_editing = Point Editing
history_management = History Management
shape_operations = Shape Operations
anchor_creation = Anchor Creation
positioning = Positioning
workflow = Workflow
joint_creation = Joint Creation
configuration = Configuration
joint_types = Joint Types
properties = Properties
transform_gizmo_controls = Transform Gizmo Controls
gizmo_operations = Gizmo Operations
# Control descriptions
left_click_drag = Left Click+Drag
draw_rectangle = Draw Rectangle
right_click_drag = Right Click+Drag
draw_circle = Draw Circle
dropdown = Dropdown
select_collider_type = Select Collider Type
panel = Panel
configure_properties = Configure Properties
capsule_shape = Capsule
two_points = Two Points
define_capsule = Define Capsule
polygon_shape = Polygon
multiple_clicks = Multiple Clicks
define_vertices = Define Vertices
triangle_shape = Triangle
three_points = Three Points
define_triangle = Define Triangle
add_point = Add Point
click_edge = Click Edge
insert_vertex = Insert Vertex
move_shape = Move Shape
drag_body = Drag Body
translate = Translate
cancel = Cancel
escape_key = Escape
exit_editing = Exit Editing
click_collider = Click Collider
create_anchor = Create Anchor
shift_click = Shift+Click
multiple_anchors = Multiple Anchors
clear_button = Clear Button
remove_all = Remove All
right_click = Right Click
preview_mode_toggle = Preview Mode
snap = Snap
snap_to_vertices = Snap to Vertices
precise = Precise
ctrl_key = Ctrl
precise_placement = Precise Placement
tab_key = Tab
switch_to_joint_mode = Switch to Joint Mode
drag_between = Drag Between
connect_anchors = Connect Anchors
enter_key = Enter
confirm_placement = Confirm Placement
fixed_length = Fixed Length
maintain_distance = Maintain Distance
hinge_type = Hinge
rotate_around_axis = Rotate Around Axis
slider = Slider
linear_motion = Linear Motion
rigid_type = Rigid
no_relative_motion = No Relative Motion
lower = Lower
more_flexible = More Flexible
enable = Enable
restrict_motion = Restrict Motion
toggle = Toggle
between_bodies = Between Bodies
select_entity = Select Entity
multi_select = Multi-Select
move_axis = Move Axis
drag_axis_arrow = Drag Axis Arrow
constrained_move = Constrained Move
rotate_action = Rotate
drag_rotation_ring = Drag Rotation Ring
constrained_rotate = Constrained Rotate
scale_action = Scale
drag_scale_handle = Drag Scale Handle
uniform_scale = Uniform Scale
snapping_action = Snapping
toggle_grid_snap = Toggle Grid Snap
# Friction types
static_friction_coefficient = Static Friction Coefficient
use_dynamic_friction = Use Dynamic Friction
restitution_coefficient = Restitution Coefficient
# Advanced physics options
use_constant_angular_acceleration = Use Constant Angular Acceleration
use_local_constant_force = Use Local Constant Force
use_local_constant_linear_acceleration = Use Local Constant Linear Acceleration
use_dominance_value = Use Dominance Value
# Motion control
motion_control = Motion Control
# Collision detection
collision_detection = Collision Detection
# Performance optimization
performance_optimization = Performance Optimization
# Tool mode
gizmo_mode_translate = Translate Mode
gizmo_mode_rotate = Rotate Mode
gizmo_mode_scale = Scale Mode
# Angle and compliance
angle = Angle
axis = Axis
limit = Limit
compliance = Compliance
# Preset types
trigger_zone = Trigger Zone
# UI elements
no_selection = No selection
# Additional control descriptions
modify_shape = Modify Shape
click_control_point = Click Point
select = Select
remove_point = Remove Point
undo_changes = Undo Changes
redo_changes = Redo Changes
insert_vertex = Insert Vertex
remove_anchor = Remove Anchor
create_joint = Create Joint
select_joint_type = Select Joint Type

# Component management
component_management = Component Management
add_components = Add Components
current_components = Current Components
component_already_added = Already added
no_components = No components
remove = Remove
add = Add
add_component_tooltip = Add this component to the entity
remove_component_tooltip = Remove this component from the entity
total_components = Total Components

# Component categories and names
rigid_body = Rigid Body
rigid_body_desc = Makes the entity participate in physics simulation
collider = Collider
collider_desc = Defines the collision shape for physics interactions
mass = Mass
mass_desc = Explicit mass value for the entity
angular_inertia = Angular Inertia
angular_inertia_desc = Resistance to rotational motion
center_of_mass = Center of Mass
center_of_mass_desc = Custom center of mass position
friction = Friction
friction_desc = Surface friction coefficient
restitution = Restitution
restitution_desc = Bounciness of collisions
linear_damping = Linear Damping
linear_damping_desc = Resistance to linear motion
angular_damping = Angular Damping
angular_damping_desc = Resistance to rotational motion
gravity_scale = Gravity Scale
gravity_scale_desc = Multiplier for gravity effects
locked_axes = Locked Axes
locked_axes_desc = Constraints on movement and rotation
max_linear_speed = Max Linear Speed
max_linear_speed_desc = Maximum linear velocity limit
max_angular_speed = Max Angular Speed
max_angular_speed_desc = Maximum angular velocity limit
sensor = Sensor
sensor_desc = Detects collisions without physical response
collision_layers = Collision Layers
collision_layers_desc = Defines which layers can collide
collision_margin = Collision Margin
collision_margin_desc = Extra margin around collision shapes
sleeping_disabled = Sleeping Disabled
sleeping_disabled_desc = Prevents entity from sleeping to save performance
transform_interpolation = Transform Interpolation
transform_interpolation_desc = Smooths movement for better visual quality
sprite = Sprite
sprite_desc = 2D sprite rendering component
color_material = Color Material
color_material_desc = Simple colored material for sprites
constant_force = Constant Force
constant_force_desc = Continuous force applied to entity
constant_torque = Constant Torque
constant_torque_desc = Continuous rotational force applied to entity
dominance = Dominance
dominance_desc = Controls how entities push each other
# Panel controls
left_panel = L
right_panel = R
asset_panel = B
reset_layout = Reset
max_viewport = Max Viewport
# Asset management
category = Category
search = Search
search_assets = Search assets
view = View
grid = Grid
list = List
details = Details
import = Import
refresh = Refresh
status = Status
path = Path
total_size = Total Size
# Sprite component translations
sprite = Sprite
sprite_desc = 2D sprite component for rendering images
sprite_settings = Sprite Settings
image_asset = Image Asset
select_image_asset = Select Image Asset
current_asset_loaded = Asset Loaded
no_asset_selected = No Asset Selected
no_assets_available = No Assets Available
import_image = Import Image
import_more_images = Import More Images
use_image_size = Use Image Size
select = Select
apply = Apply
add_sprite_tooltip = Add Sprite with selected image
available_images = Available Images
no_assets_available = No assets available
import_images_first = Import images first
# Asset management
asset_management = Asset Management
asset_channel_not_available = Asset channel not available
loaded_images = Loaded Images
import_image = Import Image
no_images_loaded = No images loaded
select = Select
loading = Loading...
unavailable = Unavailable
image_loading = Image is loading
image_unavailable = Image unavailable
click_to_select = Click to select
unknown_time = Unknown time
"#;

    // 中文翻译
    let zh_cn = r#"
app_title = Avian 物理编辑器
export_scene = 📁 导出场景
export_all = 导出所有物理实体
export_selected = 导出选中实体
export_colliders = 仅导出碰撞体
export_joints = 仅导出关节
resume_physics = ▶ 恢复物理
pause_physics = ⏸ 暂停物理
tool_mode = 工具模式
mode_select = 选择
mode_create = 创建
mode_edit = 编辑
mode_anchor = 锚点
mode_joint = 关节
transform_gizmo = 变换手柄
collider_editor = 碰撞体编辑器
anchor_tools = 锚点工具
joint_settings = 关节设置
no_entity_selected = 未选中实体
selected_entity = 选中实体
delete_entity = 删除实体
no_selection = 无选择
control_points = 控制点
no_point_selected = 未选中控制点
dragging_point = 正在拖拽控制点
editing_entity = 正在编辑实体
created_anchors = 已创建锚点
preview_position = 预览位置
selected_anchor = 选中锚点
preview_mode = 预览模式 (右键)
shift_pressed = Shift 按下 (吸附)
clear_all = 清除全部
entity_inspector = 实体检查器
inspector_mode = 检查器模式
component_management = 组件管理
component_inspector = 组件检查器
shape_edit = 形状编辑
remove_components = 移除组件
add_components = 添加组件
selection_controls = 选择控制
clear_selection = 清除选择
select_all = 选择全部
joint_type = 选择关节类型
distance = 距离
revolute = 旋转
prismatic = 平移
fixed = 固定
presets = 快速预设
rigid = 刚性
spring = 弹簧
sliding = 滑动门
hinge = 铰链
reset_defaults = 重置为默认
linear_damping = 线性阻尼
angular_damping = 角度阻尼
disable_collision = 禁用碰撞
point_compliance = 点柔度
angle_compliance = 角度柔度
limit_compliance = 限制柔度
rest_length = 静止长度
distance_limits = 距离限制
min = 最小
max = 最大
free_axis = 自由轴
axis_compliance = 轴柔度
angle_limits = 角度限制
joint_instructions = 关节创建说明
click_drag = 在锚点/实体之间点击并拖拽
configure_properties = 在上方配置属性
joint_created = 关节将在释放时创建
currently_dragging = 🖱️ 正在拖拽...
distance_label = 距离
creation_controls = 创建控制
creation_instruction_1 = 点击并拖拽绘制矩形碰撞体
creation_instruction_2 = 右键点击绘制圆形碰撞体
creation_instruction_3 = 从下拉菜单选择碰撞体类型
creation_instruction_4 = 在面板中配置属性
creation_instruction_5 = 按Enter键创建碰撞体
edit_instruction_1 = 拖拽控制点修改形状
edit_instruction_2 = 点击控制点选择
edit_instruction_3 = Delete键删除选中点
edit_instruction_4 = Ctrl+Z撤销更改
edit_instruction_5 = Ctrl+Y重做更改
anchor_instruction_1 = 点击碰撞体创建锚点
anchor_instruction_2 = Shift+点击创建多个锚点
anchor_instruction_3 = Delete键删除锚点
anchor_instruction_4 = 清除按钮删除全部
anchor_instruction_5 = Tab键切换到关节模式
anchor_instruction_6 = 在锚点间拖拽连接
anchor_instruction_7 = 放置前预览位置
anchor_instruction_8 = Enter键确认放置
joint_instruction_1 = 在锚点间拖拽创建关节
joint_instruction_2 = 从下拉菜单选择关节类型
joint_instruction_3 = 配置关节属性
selection_instruction_1 = 点击实体选择
selection_instruction_2 = Shift+点击多选
selection_instruction_3 = W键移动模式
selection_instruction_4 = E键旋转模式
selection_instruction_5 = R键缩放模式
collider_type = 碰撞体类型
body_type = 物理体类型
color = 颜色
basic_config = 基础配置
presets_config = 预设配置
mass_properties = 质量属性
material_properties = 材料属性
motion_properties = 运动属性
collision_properties = 碰撞属性
performance_properties = 性能属性
advanced_physics = 高级物理
rectangle = 矩形
circle = 圆形
capsule = 胶囊
triangle = 三角形
polygon = 多边形
static = 静态
dynamic = 动态
kinematic = 运动学
character_controller = 角色控制器
high_speed_object = 高速物体
bouncy_ball = 弹性球
static_platform = 静态平台
sensor = 传感器
physics_prop = 物理道具
vehicle = 载具
anti_gravity = 反重力
destructible = 可破坏物
explicit_mass = 显式质量
use_explicit_mass = 使用显式质量
auto_calculated = 自动计算
density = 密度
explicit_angular_inertia = 显式转动惯量
use_explicit_angular_inertia = 使用显式转动惯量
explicit_center_of_mass = 显式质心
use_explicit_center_of_mass = 使用显式质心
mass_properties_control = 质量属性控制
disable_auto_mass = 禁用自动质量
disable_auto_angular_inertia = 禁用自动转动惯量
disable_auto_center_of_mass = 禁用自动质心
friction = 摩擦系数
use_explicit_friction = 使用显式摩擦
global_default = 全局默认
static_friction = 静摩擦系数
use_explicit_static_friction = 使用显式静摩擦
use_dynamic_friction = 使用动摩擦
restitution = 弹性系数
use_explicit_restitution = 使用显式弹性
linear_velocity = 线速度
angular_velocity = 角速度
collision_layers = 碰撞层
explicit_linear_damping = 显式线性阻尼
use_explicit_linear_damping = 使用显式线性阻尼
explicit_angular_damping = 显式角度阻尼
use_explicit_angular_damping = 使用显式角度阻尼
gravity_scale = 重力缩放
use_explicit_gravity_scale = 使用显式重力缩放
max_linear_speed = 最大线速度
limit_linear_speed = 限制线速度
no_limit = 无限制
max_angular_speed = 最大角速度
limit_angular_speed = 限制角速度
locked_axes = 锁定轴
lock_x = 锁定X
lock_y = 锁定Y
lock_rotation = 锁定旋转
dominance = 优势值
use_dominance = 使用优势值
default = 默认
sensor_no_collision = 传感器 (无碰撞响应)
collision_margin = 碰撞边距
speculative_margin = 推测接触边距 (CCD)
use_speculative_margin = 使用推测接触边距
swept_ccd = 启用扫描CCD
collision_events = 启用碰撞事件
collider_disabled = 禁用碰撞体
disable_sleeping = 禁用睡眠
disable_physics = 禁用物理模拟
transform_interpolation = 启用变换插值
constant_force_world = 常力 (世界空间)
use_constant_force = 使用常力
none = 无
constant_force_local = 常力 (本地空间)
use_constant_local_force = 使用本地常力
constant_torque = 常扭矩
use_constant_torque = 使用常扭矩
constant_linear_acceleration_world = 常线性加速度 (世界空间)
use_constant_linear_acceleration = 使用常线性加速度
constant_linear_acceleration_local = 常线性加速度 (本地空间)
use_constant_local_linear_acceleration = 使用本地常线性加速度
constant_angular_acceleration = 常角加速度
use_constant_angular_acceleration = 使用常角加速度
config_updated = ✓ 配置已更新
anchor_controls = 锚点控制
create_select_anchor = 左键点击: 创建/选择锚点
move_anchor = 拖拽: 移动选中锚点
toggle_preview = 右键点击: 切换预览模式
snap_vertices = Shift: 吸附到顶点
precise_placement = Ctrl: 精确放置
remove_anchor = Delete: 移除选中锚点
switch_tools = Tab: 切换到其他工具
edit_controls = 编辑控制
drag_control_points = 点击并拖拽控制点以修改形状
undo = Ctrl+Z: 撤销更改
redo = Ctrl+Y: 重做更改
cancel_editing = Escape: 取消编辑
selection_instructions = 选择控制
left_click_select = 左键点击: 选择实体
shift_click_multi = Shift+点击: 多选
ctrl_click_toggle = Ctrl+点击: 切换选择
drag_select = 拖拽选择: 框选
delete_entities = Delete: 删除选中实体
gizmo_mode = 手柄模式
translate_w = 移动 (W)
rotate_e = 旋转 (E)
scale_r = 缩放 (R)
snap_settings = 吸附设置
enable_snapping = 启用吸附
angle_snap = 角度吸附
scale_snap = 缩放吸附
center_origin = 居中到原点
# 缺失的翻译补充
language_switcher = 🌐 语言
english = English
chinese = 中文
# 变换手柄模式
translate_mode = 移动模式
rotate_mode = 旋转模式
scale_mode = 缩放模式
# 变换手柄控制
center_to_origin = 居中到原点
# 锚点控制
quick_actions = 快速操作
multiple = 多个
connect = 连接
confirm = 确认
# 关节创建
select_type = 选择类型
# 材料属性
static_friction_coefficient = 静摩擦系数
use_dynamic_friction = 使用动摩擦
# 碰撞检测
speculative_margin_ccd = 推测接触边距 (CCD)
# 额外缺失的键
# 形状操作
add_vertex = 添加顶点
modify = 修改
# 工作流控制
switch_mode = 切换模式
# 关节创建
joint_creation_instructions = 关节创建说明
create_joint = 创建关节
currently_dragging = 正在拖拽
distance = 距离
# 预设
quick_presets = 快速预设
# 材料属性
friction_coefficient = 摩擦系数
# 运动属性
dominance_value = 优势值
# 碰撞检测
sensor_no_collision_response = 传感器 (无碰撞响应)
preset_selector = 预设选择器
no_layers_defined = 未定义任何层
available_layers = 可用层
layer_name = 层名称
description = 描述
add_new_layer = 添加新层
save_as_preset = 保存为预设
detailed_config = 详细配置
member_layers = 成员层
filter_layers = 过滤器层
basic_preset = 基本
collision_layer_management = 碰撞层管理
enable_swept_ccd = 启用扫描CCD
enable_collision_events = 启用碰撞事件
disable_collider = 禁用碰撞体
# 物理设置
disable_physics_simulation = 禁用物理模拟
enable_transform_interpolation = 启用变换插值
# 物理力
constant_force_world_space = 常力 (世界空间)
constant_force_local_space = 常力 (本地空间)
constant_linear_acceleration_world_space = 常线性加速度 (世界空间)
constant_linear_acceleration_local_space = 常线性加速度 (本地空间)
# 手柄操作
gizmo_operations = 手柄操作
move_axis = 移动轴
rotate = 旋转
scale = 缩放
snapping = 吸附
# 配置属性
linear = 线性
angular = 角度
point = 点
angle = 角度
align = 对齐
limit = 限制
axis = 轴
compliance = 柔度
min = 最小
max = 最大
# 组件管理
add_name = 添加名称
add_transform = 添加变换
add_visibility = 添加可见性
add_rigid_body = 添加刚体
add_circle_collider = 添加圆形碰撞体
add_linear_velocity = 添加线速度
add_angular_velocity = 添加角速度
add_collision_layers = 添加碰撞层
delete = 删除
despawn_entity_tooltip = 销毁实体
remove_component_tooltip = 移除组件
new_entity = 新实体
# 关节类型
distance_joint = 距离关节
revolute_joint = 旋转关节
prismatic_joint = 平移关节
fixed_joint = 固定关节
# 关节属性
distance_joint_properties = 距离关节属性
revolute_joint_properties = 旋转关节属性
prismatic_joint_properties = 平移关节属性
fixed_joint_properties = 固定关节属性
# 预设
rigid_preset = 刚性
spring_preset = 弹簧
sliding_preset = 滑动
hinge_preset = 铰链
breakable_preset = 可断裂
motorized_preset = 机动化
suspension_preset = 悬挂
rope_preset = 绳索
# 高级关节属性
advanced_properties = 高级属性
breakable_joint = 可断裂关节
breakable_settings = 可断裂设置
break_force = 断裂力
break_torque = 断裂扭矩
breakable_description = 当力超过阈值时关节会断裂
joint_motor = 关节马达
motor_settings = 马达设置
target_velocity = 目标速度
max_force = 最大力
motor_stiffness = 马达刚度
motor_damping = 马达阻尼
motor_description = 带速度和力控制的自动马达
force_tracking = 力追踪
force_tracking_settings = 力追踪设置
track_forces = 追踪力
force_tracking_description = 监控作用在关节上的力
advanced_physics = 高级物理
joint_disable_settings = 关节禁用设置
disable_collision = 禁用碰撞
disable_on_break = 断裂时禁用
advanced_physics_description = 高级行为和物理设置
disable_joint = 禁用关节
# 帮助文本更新
presets = 预设
advanced_features = 高级特性
# 数值和默认值
global_default_0_0 = 全局默认 (0.0)
global_default_0_5 = 全局默认 (0.5)
global_default_1_0 = 全局默认 (1.0)
# 状态和状态信息
edit_state_unavailable = 编辑状态不可用
anchor_state_unavailable = 锚点状态不可用
joint_state_unavailable = 关节状态不可用
joint_configuration_unavailable = 关节配置不可用
creation_properties_unavailable = 创建属性不可用
no_entity_selected_instruction = 未选中实体。点击实体以检查其组件。
selected = 已选中
# 帮助章节
drawing_controls = 绘图控制
shape_configuration = 形状配置
advanced_shapes = 高级形状
point_editing = 点编辑
history_management = 历史管理
shape_operations = 形状操作
anchor_creation = 锚点创建
positioning = 定位
workflow = 工作流
joint_creation = 关节创建
configuration = 配置
joint_types = 关节类型
properties = 属性
transform_gizmo_controls = 变换手柄控制
gizmo_operations = 手柄操作
# 控制描述
left_click_drag = 左键点击+拖拽
draw_rectangle = 绘制矩形
right_click_drag = 右键点击+拖拽
draw_circle = 绘制圆形
dropdown = 下拉菜单
select_collider_type = 选择碰撞体类型
panel = 面板
configure_properties = 配置属性
capsule_shape = 胶囊
two_points = 两点
define_capsule = 定义胶囊
polygon_shape = 多边形
multiple_clicks = 多次点击
define_vertices = 定义顶点
triangle_shape = 三角形
three_points = 三点
define_triangle = 定义三角形
add_point = 添加点
click_edge = 点击边缘
insert_vertex = 插入顶点
move_shape = 移动形状
drag_body = 拖拽物体
translate = 平移
cancel = 取消
escape_key = Escape
exit_editing = 退出编辑
click_collider = 点击碰撞体
create_anchor = 创建锚点
shift_click = Shift+点击
multiple_anchors = 多个锚点
clear_button = 清除按钮
remove_all = 移除全部
right_click = 右键点击
preview_mode_toggle = 预览模式
snap = 吸附
snap_to_vertices = 吸附到顶点
precise = 精确
ctrl_key = Ctrl
precise_placement = 精确放置
tab_key = Tab
switch_to_joint_mode = 切换到关节模式
drag_between = 在...之间拖拽
connect_anchors = 连接锚点
enter_key = Enter
confirm_placement = 确认放置
fixed_length = 固定长度
maintain_distance = 保持距离
hinge_type = 铰链
rotate_around_axis = 绕轴旋转
slider = 滑块
linear_motion = 线性运动
rigid_type = 刚性
no_relative_motion = 无相对运动
lower = 更低
more_flexible = 更灵活
enable = 启用
restrict_motion = 限制运动
toggle = 切换
between_bodies = 在物体之间
select_entity = 选择实体
multi_select = 多选
move_axis = 移动轴
drag_axis_arrow = 拖拽轴箭头
constrained_move = 约束移动
rotate_action = 旋转
drag_rotation_ring = 拖拽旋转环
constrained_rotate = 约束旋转
scale_action = 缩放
drag_scale_handle = 拖拽缩放手柄
uniform_scale = 均匀缩放
snapping_action = 吸附
toggle_grid_snap = 切换网格吸附
# 摩擦类型
static_friction_coefficient = 静摩擦系数
use_dynamic_friction = 使用动摩擦
restitution_coefficient = 弹性系数
# 高级物理选项
use_constant_angular_acceleration = 使用常角加速度
use_local_constant_force = 使用本地常力
use_local_constant_linear_acceleration = 使用本地常线性加速度
use_dominance_value = 使用优势值
# 运动控制
motion_control = 运动控制
# 碰撞检测
collision_detection = 碰撞检测
# 性能优化
performance_optimization = 性能优化
# 工具模式
gizmo_mode_translate = 移动模式
gizmo_mode_rotate = 旋转模式
gizmo_mode_scale = 缩放模式
# 角度和柔度
angle = 角度
axis = 轴
limit = 限制
compliance = 柔度
# 预设类型
trigger_zone = 触发区域
# UI 元素
no_selection = 无选择
# 面板控制
left_panel = 左
right_panel = 右
asset_panel = 资产
reset_layout = 重置
max_viewport = 最大化
# 资产管理
category = 类别
search = 搜索
search_assets = 搜索资产
view = 视图
grid = 网格
list = 列表
details = 详情
import = 导入
refresh = 刷新
status = 状态
path = 路径
total_size = 总大小
# 形状编辑翻译
shape_properties = 形状属性
no_collider_for_shape_edit = 未找到碰撞体组件用于形状编辑
missing_required_components = 缺少形状编辑所需的组件
rectangle_properties = 矩形属性
width = 宽度
height = 高度
preset_sizes = 预设尺寸
square_small = 小方形
square_medium = 中方形
square_large = 大方形
rectangle_wide = 宽矩形
rectangle_tall = 高矩形
rectangle_wide_large = 大宽矩形
circle_properties = 圆形属性
radius = 半径
diameter = 直径
preset_radii = 预设半径
radius_small = 小半径
radius_medium = 中半径
radius_large = 大半径
radius_extra_large = 超大半径
radius_huge = 巨大半径
capsule_properties = 胶囊属性
height = 高度
rotation = 旋转
preset_capsules = 预设胶囊
capsule_pill = 药丸形
capsule_tall = 高胶囊
capsule_wide = 宽胶囊
triangle_properties = 三角形属性
triangle_side_lengths = 三角形边长
triangle_radius = 三角形外接圆半径
triangle_angles = 三角形角度
side_ab = 边A-B (对角C)
side_bc = 边B-C (对角A)
side_ca = 边C-A (对角B)
angle_at_a = 角A (度)
angle_at_b = 角B (度)
angle_at_c = 角C (度)
triangle_vertices = 三角形顶点
vertex = 顶点
apply_triangle_changes = 应用三角形更改
preset_triangles = 预设三角形
equilateral_triangle = 等边三角形
right_triangle = 直角三角形
polygon_properties = 多边形属性
vertex_count = 顶点数量
edit_vertices = 编辑顶点
polygon_too_complex = 多边形过于复杂，无法手动编辑
polygon_edit_warning = 复杂多边形编辑尚未实现
preset_polygons = 预设多边形
pentagon = 五边形
hexagon = 六边形
octagon = 八边形
transform_properties = 变换属性
position = 位置
scale = 缩放
reset_transform = 重置变换
invalid_triangle_shape = 无效的三角形形状
invalid_polygon_shape = 无效的多边形形状
# Sprite组件翻译
sprite = 精灵
sprite_desc = 用于渲染图像的2D精灵组件
sprite_settings = 精灵设置
image_asset = 图像资产
select_image_asset = 选择图像资产
current_asset_loaded = 资产已加载
no_asset_selected = 未选择资产
no_assets_available = 无可用资产
import_image = 导入图像
import_more_images = 导入更多图像
use_image_size = 使用图像尺寸
select = 选择
apply = 应用
add_sprite_tooltip = 添加精灵并选择图像
available_images = 可用图像
no_assets_available = 无可用资产
import_images_first = 请先导入图像
# 额外的控制描述
modify_shape = 修改形状
click_control_point = 点击控制点
select = 选择
remove_point = 移除点
undo_changes = 撤销更改
redo_changes = 重做更改
insert_vertex = 插入顶点
remove_anchor = 移除锚点
create_joint = 创建关节
select_joint_type = 选择关节类型
# 资产管理
asset_management = 资产管理
asset_channel_not_available = 资产通道不可用
loaded_images = 已加载图片
import_image = 导入图片
no_images_loaded = 没有加载图片
select = 选择
loading = 加载中...
unavailable = 不可用
image_loading = 图片正在加载
image_unavailable = 图片不可用
click_to_select = 点击选择
unknown_time = 未知时间
"#;

    // 加载翻译
    if let Err(e) = load_translations_from_text("en", en_us) {
        eprintln!("Failed to load English translations: {}", e);
    }
    if let Err(e) = load_translations_from_text("zh", zh_cn) {
        eprintln!("Failed to load Chinese translations: {}", e);
    }

    // 设置默认语言
    set_language("en");
    set_fallback("en");
}
