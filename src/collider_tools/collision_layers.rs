use avian2d::prelude::{CollisionLayers, LayerMask};
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 运行时碰撞层定义
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct RuntimeCollisionLayer {
    pub name: String,        // 层名称
    pub bit: u8,             // 位位置 (0-31)
    pub description: String, // 层描述
}

impl RuntimeCollisionLayer {
    pub fn new(name: &str, bit: u8, description: &str) -> Self {
        Self {
            name: name.to_string(),
            bit,
            description: description.to_string(),
        }
    }

    pub fn to_layer_mask(&self) -> LayerMask {
        LayerMask(1 << self.bit)
    }
}

/// 基本碰撞层预设
#[derive(Debug, Clone, Reflect)]
pub struct CollisionLayerPreset {
    pub name: String,
    pub layers: CollisionLayers,
    pub description: String,
}

/// 碰撞层包装器，用于序列化
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct CollisionLayersWrapper {
    pub memberships: u32,
    pub filters: u32,
}

impl From<CollisionLayers> for CollisionLayersWrapper {
    fn from(layers: CollisionLayers) -> Self {
        Self {
            memberships: layers.memberships.0,
            filters: layers.filters.0,
        }
    }
}

impl From<CollisionLayersWrapper> for CollisionLayers {
    fn from(wrapper: CollisionLayersWrapper) -> Self {
        Self {
            memberships: LayerMask(wrapper.memberships),
            filters: LayerMask(wrapper.filters),
        }
    }
}

/// 自定义碰撞层预设
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct CustomCollisionLayerPreset {
    pub name: String,
    #[serde(
        serialize_with = "serialize_layers",
        deserialize_with = "deserialize_layers"
    )]
    pub layers: CollisionLayers,
    pub description: String,
    pub created_at: String,
}

fn serialize_layers<S>(layers: &CollisionLayers, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let wrapper = CollisionLayersWrapper::from(layers.clone());
    wrapper.serialize(serializer)
}

fn deserialize_layers<'de, D>(deserializer: D) -> Result<CollisionLayers, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wrapper: CollisionLayersWrapper = CollisionLayersWrapper::deserialize(deserializer)?;
    Ok(CollisionLayers::from(wrapper))
}

/// 碰撞层管理资源
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct CollisionLayerPresets {
    pub layers: Vec<RuntimeCollisionLayer>,       // 用户定义的层
    pub basic_presets: Vec<CollisionLayerPreset>, // 基本预设
    pub custom_presets: Vec<CustomCollisionLayerPreset>, // 自定义预设
    pub name_to_bit: HashMap<String, u8>,         // 名称映射
}

impl Default for CollisionLayerPresets {
    fn default() -> Self {
        let basic_presets = vec![
            CollisionLayerPreset {
                name: "DEFAULT".to_string(),
                layers: CollisionLayers::default(),
                description: "默认层和所有过滤器".to_string(),
            },
            CollisionLayerPreset {
                name: "ALL".to_string(),
                layers: CollisionLayers {
                    memberships: LayerMask::ALL,
                    filters: LayerMask::ALL,
                },
                description: "所有成员和过滤器".to_string(),
            },
            CollisionLayerPreset {
                name: "NONE".to_string(),
                layers: CollisionLayers {
                    memberships: LayerMask::NONE,
                    filters: LayerMask::NONE,
                },
                description: "无成员和无过滤器".to_string(),
            },
            CollisionLayerPreset {
                name: "ALL_MEMBERSHIPS".to_string(),
                layers: CollisionLayers {
                    memberships: LayerMask::ALL,
                    filters: LayerMask::NONE,
                },
                description: "所有成员但无过滤器".to_string(),
            },
            CollisionLayerPreset {
                name: "ALL_FILTERS".to_string(),
                layers: CollisionLayers {
                    memberships: LayerMask::NONE,
                    filters: LayerMask::ALL,
                },
                description: "所有过滤器但无成员".to_string(),
            },
        ];

        Self {
            layers: Vec::new(),
            basic_presets,
            custom_presets: Vec::new(),
            name_to_bit: HashMap::new(),
        }
    }
}

impl CollisionLayerPresets {
    pub fn add_custom_layer(&mut self, name: &str, description: &str) -> Result<u8, String> {
        // 验证名称
        if name.trim().is_empty() {
            return Err("层名称不能为空".to_string());
        }

        if self.name_to_bit.contains_key(name) {
            return Err(format!("层 '{}' 已存在", name));
        }

        // 防止与基本预设冲突
        if ["DEFAULT", "ALL", "NONE", "ALL_MEMBERSHIPS", "ALL_FILTERS"].contains(&name) {
            return Err("不能使用基本预设名称".to_string());
        }

        // 分配位位置
        let used_bits: Vec<u8> = self.layers.iter().map(|l| l.bit).collect();
        for bit in 0..32 {
            if !used_bits.contains(&bit) {
                let layer = RuntimeCollisionLayer::new(name, bit, description);
                self.layers.push(layer);
                self.name_to_bit.insert(name.to_string(), bit);
                return Ok(bit);
            }
        }

        Err("没有可用的位位置 (0-31 都已使用)".to_string())
    }

    pub fn remove_custom_layer(&mut self, name: &str) -> Result<(), String> {
        if let Some(pos) = self.layers.iter().position(|l| l.name == name) {
            self.layers.remove(pos);
            self.name_to_bit.remove(name);
            Ok(())
        } else {
            Err(format!("层 '{}' 不存在", name))
        }
    }

    pub fn save_custom_preset(
        &mut self,
        name: &str,
        description: &str,
        layers: CollisionLayers,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("预设名称不能为空".to_string());
        }

        if self.custom_presets.iter().any(|p| p.name == name) {
            return Err(format!("预设 '{}' 已存在", name));
        }

        if ["DEFAULT", "ALL", "NONE", "ALL_MEMBERSHIPS", "ALL_FILTERS"].contains(&name) {
            return Err("不能使用基本预设名称".to_string());
        }

        let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let preset = CustomCollisionLayerPreset {
            name: name.to_string(),
            layers,
            description: description.to_string(),
            created_at,
        };

        self.custom_presets.push(preset);
        Ok(())
    }

    pub fn remove_custom_preset(&mut self, name: &str) -> Result<(), String> {
        if let Some(pos) = self.custom_presets.iter().position(|p| p.name == name) {
            self.custom_presets.remove(pos);
            Ok(())
        } else {
            Err(format!("预设 '{}' 不存在", name))
        }
    }

    pub fn apply_any_preset(&self, preset_name: &str) -> Option<CollisionLayers> {
        // 先尝试基本预设
        if let Some(layers) = self.apply_basic_preset(preset_name) {
            return Some(layers);
        }
        // 再尝试自定义预设
        self.apply_custom_preset(preset_name)
    }

    pub fn get_preset_name_for_layers(&self, layers: &CollisionLayers) -> Option<String> {
        // 先检查基本预设
        if let Some(preset) = self.basic_presets.iter().find(|p| p.layers == *layers) {
            return Some(preset.name.clone());
        }
        // 再检查自定义预设
        if let Some(preset) = self.custom_presets.iter().find(|p| p.layers == *layers) {
            return Some(preset.name.clone());
        }
        None
    }

    pub fn apply_basic_preset(&self, preset_name: &str) -> Option<CollisionLayers> {
        self.basic_presets
            .iter()
            .find(|p| p.name == preset_name)
            .map(|p| p.layers.clone())
    }

    pub fn apply_custom_preset(&self, preset_name: &str) -> Option<CollisionLayers> {
        self.custom_presets
            .iter()
            .find(|p| p.name == preset_name)
            .map(|p| p.layers.clone())
    }

    pub fn get_basic_preset_names(&self) -> Vec<String> {
        self.basic_presets.iter().map(|p| p.name.clone()).collect()
    }

    pub fn get_custom_preset_names(&self) -> Vec<String> {
        self.custom_presets.iter().map(|p| p.name.clone()).collect()
    }

    pub fn get_all_preset_names(&self) -> Vec<String> {
        let mut all_names = self.get_basic_preset_names();
        all_names.extend(self.get_custom_preset_names());
        all_names
    }

    pub fn has_layers(&self) -> bool {
        !self.layers.is_empty()
    }

    pub fn get_next_available_bit(&self) -> Option<u8> {
        let used_bits: Vec<u8> = self.layers.iter().map(|l| l.bit).collect();
        for bit in 0..32 {
            if !used_bits.contains(&bit) {
                return Some(bit);
            }
        }
        None
    }

    pub fn layer_mask_to_names(&self, mask: LayerMask) -> Vec<String> {
        let mut names = Vec::new();
        for layer in &self.layers {
            if (mask.0 & (1 << layer.bit)) != 0 {
                names.push(layer.name.clone());
            }
        }
        names
    }

    pub fn names_to_layer_mask(&self, names: &[String]) -> LayerMask {
        let mut mask = LayerMask(0);
        for name in names {
            if let Some(&bit) = self.name_to_bit.get(name) {
                mask.0 |= 1 << bit;
            }
        }
        mask
    }
}

/// 碰撞层管理事件
#[derive(Event, Debug, Reflect)]
pub struct AddCustomLayerEvent {
    pub name: String,
    pub description: String,
}

#[derive(Event, Debug, Reflect)]
pub struct RemoveCustomLayerEvent {
    pub name: String,
}

#[derive(Event, Debug, Reflect)]
pub struct SaveCustomPresetEvent {
    pub name: String,
    pub description: String,
    pub layers: CollisionLayers,
}

#[derive(Event, Debug, Reflect)]
pub struct RemoveCustomPresetEvent {
    pub name: String,
}

/// CollisionLayers扩展trait
pub trait CollisionLayersExt {
    fn from_any_preset(preset_name: &str, presets: &CollisionLayerPresets) -> Option<Self>
    where
        Self: Sized;

    fn get_description(&self, presets: &CollisionLayerPresets) -> String;
    fn from_custom_layers(
        membership_names: &[String],
        filter_names: &[String],
        presets: &CollisionLayerPresets,
    ) -> Self
    where
        Self: Sized;
}

impl CollisionLayersExt for CollisionLayers {
    fn from_any_preset(preset_name: &str, presets: &CollisionLayerPresets) -> Option<Self> {
        presets.apply_any_preset(preset_name)
    }

    fn get_description(&self, presets: &CollisionLayerPresets) -> String {
        let membership_names = presets.layer_mask_to_names(self.memberships);
        let filter_names = presets.layer_mask_to_names(self.filters);

        if membership_names.is_empty() && filter_names.is_empty() {
            "无配置".to_string()
        } else {
            format!(
                "成员: [{}] | 过滤器: [{}]",
                if membership_names.is_empty() {
                    "无"
                } else {
                    &membership_names.join(", ")
                },
                if filter_names.is_empty() {
                    "无"
                } else {
                    &filter_names.join(", ")
                }
            )
        }
    }

    fn from_custom_layers(
        membership_names: &[String],
        filter_names: &[String],
        presets: &CollisionLayerPresets,
    ) -> Self {
        Self {
            memberships: presets.names_to_layer_mask(membership_names),
            filters: presets.names_to_layer_mask(filter_names),
        }
    }
}

/// 碰撞层管理插件
pub struct CollisionLayerManagementPlugin;

impl Plugin for CollisionLayerManagementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionLayerPresets>()
            .add_event::<AddCustomLayerEvent>()
            .add_event::<RemoveCustomLayerEvent>()
            .add_event::<SaveCustomPresetEvent>()
            .add_event::<RemoveCustomPresetEvent>()
            .add_systems(EguiPrimaryContextPass, handle_collision_layer_events);
    }
}

/// 碰撞层事件处理系统
fn handle_collision_layer_events(
    mut add_events: EventReader<AddCustomLayerEvent>,
    mut remove_events: EventReader<RemoveCustomLayerEvent>,
    mut save_preset_events: EventReader<SaveCustomPresetEvent>,
    mut remove_preset_events: EventReader<RemoveCustomPresetEvent>,
    mut presets: ResMut<CollisionLayerPresets>,
) {
    // 处理添加层事件
    for event in add_events.read() {
        match presets.add_custom_layer(&event.name, &event.description) {
            Ok(bit) => info!("成功添加层 '{}' (位: {})", event.name, bit),
            Err(e) => warn!("添加层失败: {}", e),
        }
    }

    // 处理删除层事件
    for event in remove_events.read() {
        match presets.remove_custom_layer(&event.name) {
            Ok(()) => info!("成功删除层 '{}'", event.name),
            Err(e) => warn!("删除层失败: {}", e),
        }
    }

    // 处理保存预设事件
    for event in save_preset_events.read() {
        match presets.save_custom_preset(&event.name, &event.description, event.layers.clone()) {
            Ok(()) => info!("成功保存预设 '{}'", event.name),
            Err(e) => warn!("保存预设失败: {}", e),
        }
    }

    // 处理删除预设事件
    for event in remove_preset_events.read() {
        match presets.remove_custom_preset(&event.name) {
            Ok(()) => info!("成功删除预设 '{}'", event.name),
            Err(e) => warn!("删除预设失败: {}", e),
        }
    }
}
