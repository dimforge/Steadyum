use crate::builtin_scenes;
use crate::operation::{Operation, Operations};
use crate::physics::PhysicsState;
use crate::styling::Theme;
use crate::ui::{debug_render, UiState};
use crate::ui::debug_render::DebugRenderState;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::Window;
use bevy_egui::{egui, EguiContexts};
use std::path::PathBuf;

use crate::builtin_scenes::BuiltinScene;
#[cfg(not(target_arch = "wasm32"))]
use native_dialog::FileDialogBuilder;

pub(super) fn ui(
    _window: &Window,
    theme: &mut Theme,
    ui_context: &mut EguiContexts,
    ui_state: &mut UiState,
    _physics: &mut PhysicsState,
    debug_render_state: &mut DebugRenderState,
    operations: &mut Operations,
    mut exit: MessageWriter<AppExit>,
) {
    egui::Window::new("main menu")
        .resizable(false)
        .title_bar(false)
        .fixed_pos([5.0, 5.0])
        .show(ui_context.ctx_mut().expect("EguiContext"), |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(all(not(target_arch = "wasm32"), feature = "serde-serialize"))]
                    if ui.button("📁 Open…").clicked() {
                        match import_data::<PhysicsState>() {
                            Ok(Some(scene)) => {
                                operations.push(Operation::ClearScene);
                                operations.push(Operation::ImportScene(scene))
                            }
                            Ok(None) => {}
                            Err(e) => error!("Failed to import scene: {:?}", e),
                        }
                    }

                    ui.menu_button("📂 Built-in scenes", |ui| {
                        for (name, builder) in builtin_scenes::builders() {
                            if ui.button(name).clicked() {
                                let scene = builder();
                                operations.push(Operation::ClearScene);
                                operations.push(Operation::ImportScene(scene.state));
                            }
                        }
                    });

                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("💾 Export…").clicked() {
                        if let Ok(Some(path)) = export_path() {
                            operations.push(Operation::ExportScene(path));
                        }
                    }

                    ui.menu_button("🐞 Debug render", |ui| {
                        debug_render::ui(ui, ui_state, debug_render_state);
                    });

                    ui.checkbox(&mut theme.dark_mode, "Dark mode");

                    if ui.button("ℹ Simulation infos…").clicked() {
                        ui_state.simulation_infos_open = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("❌ Clear scene").clicked() {
                        operations.push(Operation::ClearScene)
                    }
                    if ui.button("🚪 Exit").clicked() {
                        exit.write(AppExit::Success);
                    }
                });
            })
        });
}

#[cfg(all(not(target_arch = "wasm32"), feature = "serde-serialize"))]
fn import_data<T: for<'a> serde::Deserialize<'a>>() -> anyhow::Result<Option<T>> {
    if let Some(path) = FileDialogBuilder::default()
        .add_filter("Json", &["json"])
        .open_single_file()
        .show()?
    {
        let data = std::fs::read(path)?;
        Ok(serde_json::from_slice(&data)?)
    } else {
        Ok(None)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn export_path() -> anyhow::Result<Option<PathBuf>> {
    Ok(FileDialogBuilder::default()
        .add_filter("Json", &["json"])
        .save_single_file()
        .show()?)
}
