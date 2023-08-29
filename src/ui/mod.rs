use crate::selection::Selection;
use crate::utils::{ColliderComponentsMut, RigidBodyComponentsMut};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText},
    EguiContext,
};
use bevy_rapier::control::KinematicCharacterController;
use bevy_rapier::plugin::{RapierConfiguration, RapierContext};
use bevy_rapier::render::DebugRenderContext;
use strum_macros::EnumIter;
use ui_state::OpenObjectTab;

pub use self::plugin::RapierUiPlugin;
use crate::cli::CliArgs;
use crate::control::CharacterControlOptions;
use crate::operation::Operations;
use crate::styling::Theme;
pub(self) use gizmo::add_missing_gizmos;
pub(self) use input_blocking::focus_ui;
pub(self) use keyboard::handle_keyboard_inputs;
pub use ui_state::{ActiveMouseAction, SelectedTool, UiState};

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

pub fn load_assets(
    mut ui_context: ResMut<EguiContext>,
    _ui_state: ResMut<UiState>,
    _assets: Res<AssetServer>,
) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "blender-icons".to_owned(),
        FontData::from_static(include_bytes!("../../assets/blender-icons.ttf")),
    );
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("blender-icons".to_owned());
    ui_context.ctx_mut().set_fonts(fonts);
}

pub fn update_ui(
    mut commands: Commands,
    (windows, cli): (Res<Windows>, Res<CliArgs>),
    mut theme: ResMut<Theme>,
    mut ui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut debug_render_context: ResMut<DebugRenderContext>,
    mut physics_context: ResMut<RapierContext>,
    mut physics_config: ResMut<RapierConfiguration>,
    mut operations: ResMut<Operations>,
    exit: EventWriter<AppExit>,
    mut bodies: Query<RigidBodyComponentsMut>,
    mut colliders: Query<ColliderComponentsMut>,
    mut character_controllers: Query<(
        &mut KinematicCharacterController,
        &mut CharacterControlOptions,
    )>,
    mut selections: Query<(Entity, &mut Selection)>,
    mut visibility: Query<(Entity, &mut Visibility)>,
    mut transforms: Query<(Entity, &mut Transform)>,
) {
    if let Some(window) = windows.get_primary() {
        main_menu::ui(
            window,
            &mut theme,
            &mut ui_context,
            &mut ui_state,
            &mut *physics_context,
            &mut *physics_config,
            &mut *debug_render_context,
            &mut *operations,
            exit,
        );
        play_stop::ui(
            window,
            &cli,
            &mut ui_context,
            &mut ui_state,
            &mut *physics_context,
            &mut *physics_config,
        );
        popup_menu::ui(
            window,
            &mut ui_context,
            &mut *physics_context,
            &mut *physics_config,
        );
        tools::ui(
            window,
            &mut ui_context,
            &mut ui_state,
            &mut *physics_context,
            &mut *physics_config,
            &mut *operations,
        );
        simulation_infos::ui(&mut ui_context, &mut ui_state, &*physics_context);
        right_panel::ui(
            &mut commands,
            window,
            &cli,
            &mut ui_context,
            &mut ui_state,
            &mut *physics_context,
            &mut *physics_config,
            &mut bodies,
            &mut colliders,
            &mut character_controllers,
            &mut selections,
            &mut visibility,
            &mut transforms,
        );
    }
}
