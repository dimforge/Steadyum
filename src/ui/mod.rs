use crate::control::{CharacterController, CharacterControlOptions};
use crate::physics::{ColHandle, PhysicsState, RbHandle};
use crate::selection::Selection;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{
    egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText},
    EguiContexts,
};
use strum_macros::EnumIter;
use ui_state::OpenObjectTab;

pub use self::plugin::RapierUiPlugin;
use crate::cli::CliArgs;
use crate::operation::Operations;
use crate::styling::Theme;
pub(self) use gizmo::add_missing_gizmos;
pub(self) use input_blocking::focus_ui;
pub(self) use keyboard::handle_keyboard_inputs;
pub use ui_state::{ActiveMouseAction, SelectedTool, UiState};

pub(crate) use debug_render::DebugRenderState;

mod debug_render;
mod gizmo;
mod input_blocking;
mod keyboard;
mod main_menu;
mod play_stop;
mod plugin;
mod popup_menu;
mod right_panel;
mod simulation_infos;
mod tools;
mod ui_state;

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum ButtonTexture {
    Play,
    Pause,
    Undo,
    Redo,
    // Tools
    Cut,
    Translate,
    Rotate,
    Drag,
    Projectile,
    AddBall,
    AddCuboid,
    AddPlane,
    #[cfg(feature = "dim3")]
    AddCone,
    #[cfg(feature = "dim3")]
    AddCylinder,
    AddCapsule,
    DrawShape,
    #[cfg(feature = "dim3")]
    ImportMesh,
    #[cfg(feature = "dim3")]
    ImportVoxels,
    AddHeightfield,
    // Operations on multiple shapes
    AddIntersection,
}

impl ButtonTexture {
    pub fn icon(self) -> &'static str {
        match self {
            Self::Play => "",
            Self::Pause => "",
            Self::Undo => "",
            Self::Redo => "",
            Self::Cut => "",
            Self::Translate => "",
            Self::Rotate => "",
            Self::Drag => "",
            Self::Projectile => "",
            #[cfg(feature = "dim2")]
            Self::AddBall => "",
            #[cfg(feature = "dim2")]
            Self::AddCuboid => "",
            #[cfg(feature = "dim2")]
            Self::AddCapsule => "",
            #[cfg(feature = "dim3")]
            Self::AddBall => "",
            #[cfg(feature = "dim3")]
            Self::AddCuboid => "",
            #[cfg(feature = "dim3")]
            Self::AddCapsule => "",
            Self::AddPlane => "",
            #[cfg(feature = "dim3")]
            Self::AddCone => "",
            #[cfg(feature = "dim3")]
            Self::AddCylinder => "",
            Self::DrawShape => "",
            #[cfg(feature = "dim3")]
            Self::ImportMesh => "",
            #[cfg(feature = "dim3")]
            Self::ImportVoxels => "",
            Self::AddHeightfield => "",
            Self::AddIntersection => "",
        }
    }

    pub fn rich_text(self) -> RichText {
        let txt = egui::RichText::new(self.icon());

        match self {
            Self::Play => txt
                .color(Color32::LIGHT_GREEN)
                .font(egui::FontId::monospace(40.0).clone()),
            Self::Pause => txt
                .color(Color32::LIGHT_RED)
                .font(egui::FontId::monospace(40.0).clone()),
            Self::Undo | Self::Redo => txt
                .color(Color32::LIGHT_BLUE)
                .font(egui::FontId::monospace(40.0).clone()),
            Self::Cut | Self::Translate | Self::Rotate | Self::Drag | Self::Projectile => txt
                .color(Color32::LIGHT_BLUE)
                .font(egui::FontId::monospace(20.0).clone()),
            Self::AddBall
            | Self::AddCuboid
            | Self::AddPlane
            | Self::AddCapsule
            | Self::DrawShape
            | Self::AddHeightfield => txt
                .color(Color32::LIGHT_GREEN)
                .font(egui::FontId::monospace(20.0).clone()),
            #[cfg(feature = "dim3")]
            Self::AddCone | Self::AddCylinder | Self::ImportMesh | Self::ImportVoxels => txt
                .color(Color32::LIGHT_GREEN)
                .font(egui::FontId::monospace(20.0).clone()),
            Self::AddIntersection => txt
                .color(Color32::LIGHT_YELLOW)
                .font(egui::FontId::monospace(20.0).clone()),
        }
    }
}

fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "blender-icons".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../../assets/blender-icons.ttf"))),
    );
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("blender-icons".to_owned());
    ctx.set_fonts(fonts);
}

pub fn update_ui(
    mut commands: Commands,
    (cli, mut theme): (Res<CliArgs>, ResMut<Theme>),
    mut ui_context: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut debug_render_state: ResMut<DebugRenderState>,
    mut physics: ResMut<PhysicsState>,
    mut operations: ResMut<Operations>,
    exit: MessageWriter<AppExit>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut rb_query: Query<(Entity, &RbHandle)>,
    mut col_query: Query<(Entity, &ColHandle)>,
    mut character_controllers: Query<(
        &mut CharacterController,
        &mut CharacterControlOptions,
    )>,
    mut selections: Query<(Entity, &mut Selection)>,
    mut visibility: Query<(Entity, &mut Visibility)>,
    mut transforms: Query<(Entity, &mut Transform)>,
    mut init_frames: Local<u32>,
) {
    // Skip early frames until egui context is fully initialized.
    // Frame 0-1: egui context not ready yet.
    // Frame 2: load custom fonts, skip UI this frame to let egui process them.
    // Frame 3+: normal operation.
    if *init_frames < 2 {
        *init_frames += 1;
        return;
    }
    if *init_frames == 2 {
        if let Ok(ctx) = ui_context.ctx_mut() {
            load_fonts(ctx);
        }
        *init_frames = 3;
        return; // Skip this frame to let egui process the new fonts.
    }

    if let Ok(window) = windows.single() {
        main_menu::ui(
            window,
            &mut theme,
            &mut ui_context,
            &mut ui_state,
            &mut *physics,
            &mut *debug_render_state,
            &mut *operations,
            exit,
        );
        play_stop::ui(
            window,
            &cli,
            &mut ui_context,
            &mut ui_state,
            &mut *physics,
        );
        popup_menu::ui(
            window,
            &mut ui_context,
            &mut *physics,
        );
        tools::ui(
            window,
            &mut ui_context,
            &mut ui_state,
            &mut *physics,
            &mut *operations,
        );
        simulation_infos::ui(&mut ui_context, &mut ui_state, &*physics);
        right_panel::ui(
            &mut commands,
            window,
            &cli,
            &mut ui_context,
            &mut ui_state,
            &mut *physics,
            &mut rb_query,
            &mut col_query,
            &mut character_controllers,
            &mut selections,
            &mut visibility,
            &mut transforms,
        );
    }
}
