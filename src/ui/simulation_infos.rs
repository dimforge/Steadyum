use crate::ui::UiState;
use bevy_egui::{egui, EguiContexts};
use bevy_rapier::plugin::RapierContext;
use bevy_rapier::rapier::counters::Counters;

pub(super) fn ui(ui_context: &mut EguiContexts, ui_state: &mut UiState, physics: &RapierContext) {
    egui::Window::new("ℹ Simulation infos")
        .open(&mut ui_state.simulation_infos_open)
        .resizable(false)
        .show(ui_context.ctx_mut(), |ui| {
            ui.label(stats_string(physics));
            ui.collapsing("Profile infos", |ui| {
                ui.horizontal_wrapped(|ui| ui.label(profiling_string(&physics.pipeline.counters)));
            });
            // ui.collapsing("Serialization infos", |ui| {
            //     ui.horizontal_wrapped(|ui| ui.label(serialization_string(0, physics)));
            // });
        });
}

fn stats_string(physics: &RapierContext) -> String {
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
        Position assembly: {:.2}ms
        Position resolution: {:.2}ms
CCD: {:.2}ms
        # of substeps: {}
        TOI computation: {:.2}ms
        Broad-phase: {:.2}ms
        Narrow-phase: {:.2}ms
        Solver: {:.2}ms"#,
        counters.step_time(),
        counters.collision_detection_time(),
        counters.broad_phase_time(),
        counters.narrow_phase_time(),
        counters.island_construction_time(),
        counters.solver_time(),
        counters.solver.velocity_assembly_time.time(),
        counters.velocity_resolution_time(),
        counters.solver.velocity_update_time.time(),
        counters.solver.position_assembly_time.time(),
        counters.position_resolution_time(),
        counters.ccd_time(),
        counters.ccd.num_substeps,
        counters.ccd.toi_computation_time.time(),
        counters.ccd.broad_phase_time.time(),
        counters.ccd.narrow_phase_time.time(),
        counters.ccd.solver_time.time(),
    )
}

// fn serialization_string(timestep_id: usize, physics: &PhysicsContext) -> String {
//     let t = instant::now();
//     // let t = instant::now();
//     let bf = bincode::serialize(&physics.broad_phase).unwrap();
//     // println!("bf: {}", instant::now() - t);
//     // let t = instant::now();
//     let nf = bincode::serialize(&physics.narrow_phase).unwrap();
//     // println!("nf: {}", instant::now() - t);
//     // let t = instant::now();
//     let bs = bincode::serialize(&physics.bodies).unwrap();
//     // println!("bs: {}", instant::now() - t);
//     // let t = instant::now();
//     let cs = bincode::serialize(&physics.colliders).unwrap();
//     // println!("cs: {}", instant::now() - t);
//     // let t = instant::now();
//     let js = bincode::serialize(&physics.impulse_joints).unwrap();
//     // println!("js: {}", instant::now() - t);
//     let serialization_time = instant::now() - t;
//     let hash_bf = md5::compute(&bf);
//     let hash_nf = md5::compute(&nf);
//     let hash_bodies = md5::compute(&bs);
//     let hash_colliders = md5::compute(&cs);
//     let hash_joints = md5::compute(&js);
//     format!(
//         r#"Serialization time: {:.2}ms
// Hashes at frame: {}
// |_ Broad phase [{:.1}KB]: {}
// |_ Narrow phase [{:.1}KB]: {}
// |_ Bodies [{:.1}KB]: {}
// |_ Colliders [{:.1}KB]: {}
// |_ Joints [{:.1}KB]: {}"#,
//         serialization_time,
//         timestep_id,
//         bf.len() as f32 / 1000.0,
//         format!("{:?}", hash_bf).split_at(10).0,
//         nf.len() as f32 / 1000.0,
//         format!("{:?}", hash_nf).split_at(10).0,
//         bs.len() as f32 / 1000.0,
//         format!("{:?}", hash_bodies).split_at(10).0,
//         cs.len() as f32 / 1000.0,
//         format!("{:?}", hash_colliders).split_at(10).0,
//         js.len() as f32 / 1000.0,
//         format!("{:?}", hash_joints).split_at(10).0,
//     )
// }
