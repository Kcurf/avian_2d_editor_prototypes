use bevy::prelude::*;

/// 面板状态资源
#[derive(Resource, Default, Clone)]
pub struct PanelState {
    pub left_panel_visible: bool,
    pub right_panel_visible: bool,
    pub bottom_panel_visible: bool,
}

/// 实体检查器页面模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityInspectorMode {
    #[default]
    ComponentManagement,
    ComponentInspector,
    ShapeEdit,
}

/// 实体检查器状态资源
#[derive(Resource, Default)]
pub struct EntityInspectorState {
    pub current_mode: EntityInspectorMode,
}

/// 面板控制事件
#[derive(Event, Debug)]
pub enum PanelControlEvent {
    ToggleLeftPanel,
    ToggleRightPanel,
    ToggleBottomPanel,
    MaximizeViewport,
}

pub struct PanelControlPlugin;

impl Plugin for PanelControlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PanelState {
            left_panel_visible: true,
            right_panel_visible: true,
            bottom_panel_visible: false,
        })
        .insert_resource(EntityInspectorState::default())
        .add_event::<PanelControlEvent>()
        .add_systems(Update, handle_panel_controls);
    }
}

/// 面板控制系统
fn handle_panel_controls(
    mut events: EventReader<PanelControlEvent>,
    mut panel_state: ResMut<PanelState>,
) {
    for event in events.read() {
        match event {
            PanelControlEvent::ToggleLeftPanel => {
                panel_state.left_panel_visible = !panel_state.left_panel_visible;
            }
            PanelControlEvent::ToggleRightPanel => {
                panel_state.right_panel_visible = !panel_state.right_panel_visible;
            }
            PanelControlEvent::ToggleBottomPanel => {
                panel_state.bottom_panel_visible = !panel_state.bottom_panel_visible;
            }
            PanelControlEvent::MaximizeViewport => {
                panel_state.left_panel_visible = false;
                panel_state.right_panel_visible = false;
                panel_state.bottom_panel_visible = false;
            }
        }
    }
}
