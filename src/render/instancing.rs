//! A shader that renders a mesh multiple times in one draw call.

use crate::cli::CliArgs;
use crate::render::ColliderRender;
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ExtractedView, NoFrustumCulling},
        RenderApp, RenderStage,
    },
};
use bevy_rapier::geometry::{Collider, ColliderView};
use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone, Debug, Component)]
pub struct CuboidInstances;
pub struct CuboidInstance {
    pub scale: Vec3,
}

pub fn init_primitive_instances(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((
        meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        CuboidInstances,
        SpatialBundle::VISIBLE_IDENTITY,
        InstanceMaterialData(vec![]),
        // NOTE: Frustum culling is done based on the Aabb of the Mesh and the GlobalTransform.
        // As the cube is at the origin, if its Aabb moves outside the view frustum, all the
        // instanced cubes will be culled.
        // The InstanceMaterialData contains the 'GlobalTransform' information for this custom
        // instancing, and that is not taken into account with the built-in frustum culling.
        // We must disable the built-in frustum culling by adding the `NoFrustumCulling` marker
        // component to avoid incorrect culling.
        NoFrustumCulling,
    ));
}

pub fn update_instance_material_data(
    args: Res<CliArgs>,
    colliders: Query<(&Collider, &ColliderRender, &GlobalTransform)>,
    mut cuboid_instances: Query<&mut InstanceMaterialData, With<CuboidInstances>>,
) {
    if !args.lower_graphics {
        return;
    }

    let mut cuboid_instances = cuboid_instances.single_mut();
    cuboid_instances.0.clear();

    for (collider, collider_render, transform) in colliders.iter() {
        let (_scale, rot, tra) = transform.to_scale_rotation_translation();
        match collider.as_typed_shape() {
            ColliderView::Cuboid(cuboid) => cuboid_instances.0.push(InstanceData {
                position: tra,
                rotation: Mat3::from_quat(rot),
                #[cfg(feature = "dim2")]
                scale: Vec3::new(cuboid.half_extents().x, cuboid.half_extents().y, 0.5) * 2.0,
                #[cfg(feature = "dim3")]
                scale: cuboid.half_extents() * 2.0,
                color: collider_render.color.into(),
            }),
            _ => {}
        }
    }
}

#[derive(Component, Deref)]
pub struct InstanceMaterialData(Vec<InstanceData>);

impl ExtractComponent for InstanceMaterialData {
    type Query = &'static InstanceMaterialData;
    type Filter = ();

    fn extract_component(item: QueryItem<'_, Self::Query>) -> Self {
        InstanceMaterialData(item.0.clone())
    }
}

pub struct InstancingMaterialPlugin;

impl Plugin for InstancingMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<InstanceMaterialData>::default())
            .add_startup_system(init_primitive_instances)
            .add_system(update_instance_material_data);
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<InstancingPipeline>()
            .init_resource::<SpecializedMeshPipelines<InstancingPipeline>>()
            .add_system_to_stage(RenderStage::Queue, queue_instancing)
            .add_system_to_stage(RenderStage::Prepare, prepare_instance_buffers);
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    position: Vec3,
    rotation: Mat3,
    scale: Vec3,
    color: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
fn queue_instancing(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<InstancingPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancingPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<InstanceMaterialData>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in &mut views {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                let key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

#[derive(Component)]
pub struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

#[derive(Resource)]
pub struct InstancingPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

impl FromWorld for InstancingPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        InstancingPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for InstancingPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: VertexFormat::Float32x3.size(),
                    shader_location: 4,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: VertexFormat::Float32x3.size() * 2,
                    shader_location: 5,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: VertexFormat::Float32x3.size() * 3,
                    shader_location: 6,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: VertexFormat::Float32x3.size() * 4,
                    shader_location: 7,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: VertexFormat::Float32x3.size() * 5,
                    shader_location: 8,
                },
            ],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
        ]);

        Ok(descriptor)
    }
}

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

pub struct DrawMeshInstanced;

impl EntityRenderCommand for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<InstanceBuffer>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, instance_buffer_query): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();
        let instance_buffer = instance_buffer_query.get_inner(item).unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
