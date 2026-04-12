use crate::cli::CliArgs;
use crate::physics::PhysicsState;
use bevy::window::Window;
use bevy_egui::egui::PointerButton;
use bevy_egui::{egui, EguiContexts};

use super::{ButtonTexture, UiState};

pub(super) fn ui(
    window: &Window,
    cli: &CliArgs,
    ui_context: &mut EguiContexts,
    ui_state: &mut UiState,
    physics: &mut PhysicsState,
) {
    if ui_state.single_step {
        ui_state.single_step = false;
        ui_state.running = false;
        physics.running = false;
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
        .show(ui_context.ctx_mut().expect("EguiContext"), |ui| {
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
                        physics.running = !physics.running;
                    }

                    ui_state.running = !ui_state.running;
                } else if play_pause_button.clicked_by(PointerButton::Secondary) {
                    if !cli.distributed_physics {
                        physics.running = true;
                    }

                    ui_state.running = true;
                    ui_state.single_step = true;
                }

                let _ = ui.button(ButtonTexture::Redo.rich_text());
            })
        });
}
