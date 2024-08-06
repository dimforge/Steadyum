use crate::builtin_scenes;
use crate::operation::{Operation, Operations};
use crate::styling::Theme;
use crate::ui::{debug_render, UiState};
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::Window;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier::plugin::{RapierConfiguration, RapierContext};
use bevy_rapier::render::DebugRenderContext;
use std::path::PathBuf;

use crate::builtin_scenes::BuiltinScene;
#[cfg(not(target_arch = "wasm32"))]
use native_dialog::FileDialog;

pub(super) fn ui(
    _window: &Window,
    theme: &mut Theme,
    ui_context: &mut EguiContexts,
    ui_state: &mut UiState,
    _physics_context: &mut RapierContext,
    _physics_config: &mut RapierConfiguration,
    debug_render_context: &mut DebugRenderContext,
    operations: &mut Operations,
    mut exit: EventWriter<AppExit>,
) {
    egui::Window::new("main menu")
        .resizable(false)
        .title_bar(false)
        .fixed_pos([5.0, 5.0])
        .show(ui_context.ctx_mut(), |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("📁 Open…").clicked() {
                        match import_data::<RapierContext>() {
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
                                operations.push(Operation::ImportScene(scene.context));
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
                        debug_render::ui(ui, ui_state, &mut *debug_render_context);
                    });

                    ui.checkbox(&mut theme.dark_mode, "Dark mode");

                    if ui.button("ℹ Simulation infos…").clicked() {
                        ui_state.simulation_infos_open = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("❌ Clear scene").clicked() {
                        operations.push(Operation::ClearScene)
                    }
                    if ui.button("🚪 Exit").clicked() {
                        exit.send(AppExit::Success);
                    }
                });
            })
        });
}

#[cfg(not(target_arch = "wasm32"))]
fn import_data<T: for<'a> serde::Deserialize<'a>>() -> anyhow::Result<Option<T>> {
    if let Some(path) = FileDialog::new()
        .add_filter("Json", &["json"])
        .show_open_single_file()?
    {
        let data = std::fs::read(path)?;
        Ok(serde_json::from_slice(&data)?)
    } else {
        Ok(None)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn export_path() -> anyhow::Result<Option<PathBuf>> {
    Ok(FileDialog::new()
        .add_filter("Json", &["json"])
        .show_save_single_file()?)
}
