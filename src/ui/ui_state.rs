use bevy::prelude::*;
use bevy_egui::egui::{Color32, FontId, RichText, TextureId};

// TODO: not sure where to put this?
#[derive(Copy, Clone, Debug, PartialEq, Eq, Resource)]
pub enum ActiveMouseAction {
    Insertion,
    Selection,
    Drag,
    Projectile,
    None,
}

#[derive(Resource)]
pub struct UiState {
    pub button_texture_handles: Vec<Handle<Image>>,
    pub button_textures: Vec<TextureId>,
    pub debug_render_open: bool,
    pub simulation_infos_open: bool,
    pub selected_tool: SelectedTool,
    pub open_object_tab: OpenObjectTab,
    pub single_step: bool,
    pub running: bool,
    pub interpolation: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            button_texture_handles: vec![],
            button_textures: vec![],
            debug_render_open: false,
            simulation_infos_open: false,
            selected_tool: SelectedTool::Drag,
            open_object_tab: OpenObjectTab::SelectionInspector,
            single_step: false,
            running: false,
            interpolation: true,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SelectedTool {
    Cut,
    Translate,
    Rotate,
    Drag,
    Projectile,
    AddBall,
    AddCuboid,
    AddCapsule,
    #[cfg(feature = "dim3")]
    AddCylinder,
    #[cfg(feature = "dim3")]
    AddCone,
    AddHeightfield,
    AddPlane,
    DrawShape,
}

impl Default for SelectedTool {
    fn default() -> Self {
        SelectedTool::Translate
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpenObjectTab {
    SelectionInspector,
    NewBodyInspector,
}

impl OpenObjectTab {
    pub fn icon(self) -> &'static str {
        match self {
            Self::SelectionInspector => "",
            Self::NewBodyInspector => "",
        }
    }

    pub fn rich_text(self) -> RichText {
        let txt = RichText::new(self.icon());

        match self {
            Self::SelectionInspector => txt
                .color(Color32::GOLD)
                .font(FontId::monospace(20.0).clone()),
            Self::NewBodyInspector => txt
                .color(Color32::GOLD)
                .font(FontId::monospace(20.0).clone()),
        }
    }
}
