use crate::cli::CliArgs;
use bevy::window::Window;
use bevy_egui::egui::PointerButton;
use bevy_egui::{egui, EguiContext};
use bevy_rapier::plugin::{RapierConfiguration, RapierContext};

use super::{ButtonTexture, UiState};

pub(super) fn ui(
    window: &Window,
    cli: &CliArgs,
    ui_context: &mut EguiContext,
    ui_state: &mut UiState,
    _physics_context: &mut RapierContext,
    physics_config: &mut RapierConfiguration,
) {
    if ui_state.single_step {
        ui_state.single_step = false;
        ui_state.running = false;
        physics_config.physics_pipeline_active = false;
    }

    let button_sz = [40.0, 40.0];
    let num_buttons = 3;
    let pos = [
        (window.width() - num_buttons as f32 * button_sz[0]) / 2.0,
        window.height() - button_sz[1] - 30.0,
    ];

    egui::Window::new("play_stop")
        .resizable(false)
        .title_bar(false)
        .fixed_pos(pos)
        .show(ui_context.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                let _ = ui.button(ButtonTexture::Undo.rich_text());

                let play_pause = if ui_state.running {
                    ButtonTexture::Pause
                } else {
                    ButtonTexture::Play
                };

                let play_pause_button = ui.button(play_pause.rich_text());
                if play_pause_button.clicked_by(PointerButton::Primary) {
                    if !cli.distributed_physics {
                        physics_config.physics_pipeline_active =
                            !physics_config.physics_pipeline_active;
                    }

                    ui_state.running = !ui_state.running;
                } else if play_pause_button.clicked_by(PointerButton::Secondary) {
                    if !cli.distributed_physics {
                        physics_config.physics_pipeline_active = true;
                    }

                    ui_state.running = true;
                    ui_state.single_step = true;
                }

                let _ = ui.button(ButtonTexture::Redo.rich_text());
            })
        });
}
