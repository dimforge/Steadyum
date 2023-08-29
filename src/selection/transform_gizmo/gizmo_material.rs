use bevy::pbr::MaterialPipelineKey;
use bevy::{
    pbr::MaterialPipeline,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

pub const GIZMO_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13953800272683943019);

#[derive(Component, Clone)]
#[cfg(feature = "dim2")]
pub struct GizmoStateMaterials {
    pub idle: Handle<ColorMaterial>,
    pub hovered: Handle<ColorMaterial>,
}

#[derive(Component, Clone)]
#[cfg(feature = "dim3")]
pub struct GizmoStateMaterials {
    pub idle: Handle<GizmoMaterial>,
    pub hovered: Handle<GizmoMaterial>,
}

#[derive(Debug, Clone, Default, TypeUuid, AsBindGroup)]
#[uuid = "674139bc-f374-4c3e-a935-9eee5f064ad2"]
pub struct GizmoMaterial {
    #[uniform(0)]
    pub color: Color,
}
impl From<Color> for GizmoMaterial {
    fn from(color: Color) -> Self {
        GizmoMaterial { color }
    }
}

impl Material for GizmoMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GIZMO_SHADER_HANDLE.typed())
    }

    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(GIZMO_SHADER_HANDLE.typed())
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}
