use bevy::{
    asset::uuid_handle,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    mesh::MeshVertexBufferLayoutRef,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

pub const GIZMO_SHADER_HANDLE: Handle<Shader> = uuid_handle!("a3c0e3b0-4e7b-4f1a-9c3d-0d2e4f5a6b7c");

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GizmoMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl From<Color> for GizmoMaterial {
    fn from(color: Color) -> Self {
        GizmoMaterial {
            color: color.into(),
        }
    }
}

impl Material for GizmoMaterial {
    fn vertex_shader() -> ShaderRef {
        GIZMO_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        GIZMO_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        Ok(())
    }
}

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
