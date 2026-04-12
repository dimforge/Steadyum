use crate::physics::{Counters, PhysicsState};
use crate::ui::UiState;
use bevy_egui::{egui, EguiContexts};

pub(super) fn ui(ui_context: &mut EguiContexts, ui_state: &mut UiState, physics: &PhysicsState) {
    egui::Window::new("ℹ Simulation infos")
        .open(&mut ui_state.simulation_infos_open)
        .resizable(false)
        .show(ui_context.ctx_mut().expect("EguiContext"), |ui| {
            ui.label(stats_string(physics));
            ui.collapsing("Profile infos", |ui| {
                ui.horizontal_wrapped(|ui| ui.label(profiling_string(&physics.counters)));
            });
        });
}

fn stats_string(physics: &PhysicsState) -> String {
    format!(
        r#"Rigid-bodies: {}
Colliders: {}
Impulse joints: {}"#,
        physics.bodies.len(),
        physics.colliders.len(),
        physics.impulse_joints.len(),
    )
}

fn profiling_string(counters: &Counters) -> String {
    format!(
        r#"Total: {:.2}ms
Collision detection: {:.2}ms
        Broad-phase: {:.2}ms
        Narrow-phase: {:.2}ms
Island computation: {:.2}ms
Solver: {:.2}ms
        Velocity assembly: {:.2}ms
        Velocity resolution: {:.2}ms
        Velocity integration: {:.2}ms
CCD: {:.2}ms
        # of substeps: {}
        TOI computation: {:.2}ms
        Broad-phase: {:.2}ms
        Narrow-phase: {:.2}ms
        Solver: {:.2}ms"#,
        counters.step_time_ms(),
        counters.collision_detection_time_ms(),
        counters.broad_phase_time_ms(),
        counters.narrow_phase_time_ms(),
        counters.island_construction_time_ms(),
        counters.solver_time_ms(),
        counters.solver.velocity_assembly_time.time_ms(),
        counters.velocity_resolution_time_ms(),
        counters.solver.velocity_update_time.time_ms(),
        counters.ccd_time_ms(),
        counters.ccd.num_substeps,
        counters.ccd.toi_computation_time.time_ms(),
        counters.ccd.broad_phase_time.time_ms(),
        counters.ccd.narrow_phase_time.time_ms(),
        counters.ccd.solver_time.time_ms(),
    )
}
