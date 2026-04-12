use crate::physics::{
    DebugRenderBackend, DebugRenderMode, DebugRenderObject, DebugRenderPipeline, DebugRenderStyle,
    PhysicsState, Vector,
};
use crate::ui::UiState;
use bevy::prelude::*;
use bevy_egui::egui::Ui;

/// Debug render state resource. Holds the toggles AND the rapier DebugRenderPipeline.
#[derive(Resource)]
pub struct DebugRenderState {
    pub enabled: bool,
    pub render_collider_aabbs: bool,
    pub render_collider_shapes: bool,
    pub render_rigid_body_axes: bool,
    pub render_impulse_joints: bool,
    pub render_solver_contacts: bool,
    pub render_contacts: bool,
    pub pipeline: DebugRenderPipeline,
}

impl Default for DebugRenderState {
    fn default() -> Self {
        let style = DebugRenderStyle {
            rigid_body_axes_length: 0.5,
            ..DebugRenderStyle::default()
        };
        Self {
            enabled: false,
            render_collider_aabbs: false,
            render_collider_shapes: true,
            render_rigid_body_axes: true,
            render_impulse_joints: true,
            render_solver_contacts: false,
            render_contacts: false,
            pipeline: DebugRenderPipeline::new(style, DebugRenderMode::default()),
        }
    }
}

impl DebugRenderState {
    pub fn mode(&self) -> DebugRenderMode {
        let mut mode = DebugRenderMode::empty();
        if self.render_collider_shapes {
            mode |= DebugRenderMode::COLLIDER_SHAPES;
        }
        if self.render_collider_aabbs {
            mode |= DebugRenderMode::COLLIDER_AABBS;
        }
        if self.render_rigid_body_axes {
            mode |= DebugRenderMode::RIGID_BODY_AXES;
        }
        if self.render_impulse_joints {
            mode |= DebugRenderMode::IMPULSE_JOINTS | DebugRenderMode::MULTIBODY_JOINTS;
        }
        if self.render_solver_contacts {
            mode |= DebugRenderMode::SOLVER_CONTACTS;
        }
        if self.render_contacts {
            mode |= DebugRenderMode::CONTACTS;
        }
        mode
    }
}

/// Bevy gizmo backend for rapier's DebugRenderPipeline.
struct GizmoBackend<'a, 'w, 's> {
    gizmos: &'a mut Gizmos<'w, 's>,
}

impl<'a, 'w, 's> DebugRenderBackend for GizmoBackend<'a, 'w, 's> {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject,
        a: Vector,
        b: Vector,
        color: [f32; 4],
    ) {
        // Rapier's color is HSLA; convert to Bevy's Color::hsla
        let bevy_color = Color::hsla(color[0], color[1], color[2], color[3]);
        #[cfg(feature = "dim3")]
        {
            self.gizmos.line(a, b, bevy_color);
        }
        #[cfg(feature = "dim2")]
        {
            self.gizmos.line_2d(a, b, bevy_color);
        }
    }
}

/// System that renders the physics debug visualization via Bevy gizmos.
pub fn render_debug(
    physics: Res<PhysicsState>,
    mut debug_render: ResMut<DebugRenderState>,
    mut gizmos: Gizmos,
) {
    if !debug_render.enabled {
        return;
    }

    debug_render.pipeline.mode = debug_render.mode();

    let DebugRenderState { pipeline, .. } = &mut *debug_render;
    let mut backend = GizmoBackend { gizmos: &mut gizmos };
    pipeline.render(
        &mut backend,
        &physics.bodies,
        &physics.colliders,
        &physics.impulse_joints,
        &physics.multibody_joints,
        &physics.narrow_phase,
    );
}

pub(super) fn ui(ui: &mut Ui, ui_state: &mut UiState, debug_render: &mut DebugRenderState) {
    ui.checkbox(&mut debug_render.enabled, "Enabled");
    ui.separator();

    let items = [
        (&mut debug_render.render_collider_aabbs, "AABBs"),
        (&mut debug_render.render_collider_shapes, "Colliders"),
        (&mut debug_render.render_rigid_body_axes, "Bodies"),
        (&mut debug_render.render_impulse_joints, "Joints"),
        (&mut debug_render.render_solver_contacts, "Solver Contacts"),
        (&mut debug_render.render_contacts, "Contacts"),
    ];

    for (enabled, text) in items {
        ui.checkbox(enabled, text);
    }

    ui.checkbox(&mut ui_state.interpolation, "Interpolation");
}
