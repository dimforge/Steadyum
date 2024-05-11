use super::{ButtonTexture, SelectedTool, UiState};
use crate::operation::{Operation, Operations};
use bevy::window::Window;
use bevy_egui::{egui, EguiContexts};

use bevy_rapier::plugin::{RapierConfiguration, RapierContext};

#[cfg(feature = "voxels")]
use {
    crate::utils::{ColliderBundle, RigidBodyBundle},
    bevy::prelude::Transform,
    bevy_rapier::geometry::Collider,
    bevy_rapier::math::Vect,
};

#[cfg(feature = "dim3")]
use bevy_rapier::geometry::ComputedColliderShape;

pub(super) fn ui(
    window: &Window,
    ui_context: &mut EguiContexts,
    ui_state: &mut UiState,
    _physics_context: &mut RapierContext,
    _physics_config: &mut RapierConfiguration,
    operations: &mut Operations,
) {
    let set_style = |ui: &mut egui::Ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.spacing_mut().button_padding.x = 4.0;
    };

    let button_sz = [20.0, 20.0];
    let num_rows = 6;
    let mut pos = [
        10.0,
        window.height() - button_sz[1] * 1.0 * (num_rows as f32) - 20.0,
    ];

    egui::Window::new("selection tools")
        .resizable(false)
        .title_bar(false)
        .fixed_pos(pos)
        .show(ui_context.ctx_mut(), |ui| {
            set_style(ui);

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::Cut,
                    ButtonTexture::Cut.rich_text(),
                );

                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::Translate,
                    ButtonTexture::Translate.rich_text(),
                );
            });
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::Rotate,
                    ButtonTexture::Rotate.rich_text(),
                );
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::Drag,
                    ButtonTexture::Drag.rich_text(),
                );
            });
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::Projectile,
                    ButtonTexture::Projectile.rich_text(),
                );
            });
        });

    pos[0] += button_sz[0] * 4.0;

    egui::Window::new("insertion tools")
        .resizable(false)
        .title_bar(false)
        .fixed_pos(pos)
        .show(ui_context.ctx_mut(), |ui| {
            set_style(ui);

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::AddBall,
                    ButtonTexture::AddBall.rich_text(),
                );
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::AddCuboid,
                    ButtonTexture::AddCuboid.rich_text(),
                );
            });
            #[cfg(feature = "dim3")]
            {
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut ui_state.selected_tool,
                        SelectedTool::AddCylinder,
                        ButtonTexture::AddCylinder.rich_text(),
                    );
                    ui.selectable_value(
                        &mut ui_state.selected_tool,
                        SelectedTool::AddCone,
                        ButtonTexture::AddCone.rich_text(),
                    );
                });
            }
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::AddCapsule,
                    ButtonTexture::AddCapsule.rich_text(),
                );
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::DrawShape,
                    ButtonTexture::DrawShape.rich_text(),
                );
            });

            #[cfg(feature = "dim3")]
            #[cfg(not(target_arch = "wasm32"))]
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(ButtonTexture::ImportMesh.rich_text()))
                    .clicked()
                {
                    if let Ok(Some(path)) = native_dialog::FileDialog::new()
                        .add_filter("STL Mesh", &["stl"])
                        .add_filter("OBJ Mesh", &["obj"])
                        .show_open_single_file()
                    {
                        operations.push(Operation::ImportMesh(path, ComputedColliderShape::TriMesh))
                    }
                }

                #[cfg(feature = "voxels")]
                if ui
                    .add(egui::Button::new(ButtonTexture::ImportVoxels.rich_text()))
                    .clicked()
                {
                    if let Ok(Some(path)) = native_dialog::FileDialog::new()
                        .add_filter("MagicaVoxel", &["vox"])
                        .add_filter("Mesh", &["stl", "obj"])
                        .show_open_single_file()
                    {
                        if path.extension().map(|ext| ext == "vox") == Some(true) {
                            match dot_vox::load(path.as_path().to_str().unwrap()) {
                                Ok(data) => {
                                    for model in &data.models {
                                        let voxel_size = 0.1;
                                        let centers: Vec<_> = model
                                            .voxels
                                            .iter()
                                            .map(|v| {
                                                Vect::new(v.x as f32, v.y as f32, v.z as f32)
                                                    * voxel_size
                                            })
                                            .collect();
                                        let voxels = Collider::voxels(&centers, voxel_size);
                                        operations.push(Operation::AddCollider(
                                            ColliderBundle::new(voxels),
                                            RigidBodyBundle::fixed(),
                                            Transform::default(),
                                        ))
                                    }
                                }
                                Err(err) => log::error!("Failed to load voxel file {}", err),
                            }
                        } else {
                            operations.push(Operation::ImportMesh(
                                path,
                                ComputedColliderShape::Voxels {
                                    voxel_size: 0.1,
                                    fill_mode: crate::FillMode::SurfaceOnly, // FillMode::default(),
                                },
                            ))
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut ui_state.selected_tool,
                    SelectedTool::AddHeightfield,
                    ButtonTexture::AddHeightfield.rich_text(),
                );
            });
        });

    pos[0] += button_sz[0] * 4.0;
    egui::Window::new("extra_tools")
        .resizable(false)
        .title_bar(false)
        .fixed_pos(pos)
        .show(ui_context.ctx_mut(), |ui| {
            set_style(ui);

            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(
                        ButtonTexture::AddIntersection.rich_text(),
                    ))
                    .clicked()
                {
                    operations.push(Operation::AddIntersection)
                }
            });
        });
}
