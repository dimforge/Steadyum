use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy_rapier::parry::shape::TriMesh;
use bevy_rapier::rapier::math::{Isometry, Point};

pub fn bevy_pbr_bundle_from_trimesh(
    meshes: &mut Assets<Mesh>,
    trimesh: &TriMesh,
    position: Isometry<f32>,
) -> PbrBundle {
    let mesh = bevy_mesh_from_trimesh(&trimesh);
    let tra = bevy_rapier::utils::iso_to_transform(&position);
    let scaled_tra = Transform {
        translation: tra.translation,
        rotation: tra.rotation,
        scale: Vec3::splat(1.00001),
    };
    PbrBundle {
        mesh: meshes.add(mesh),
        transform: scaled_tra,
        global_transform: GlobalTransform::from(scaled_tra),
        ..Default::default()
    }
}

#[cfg(feature = "dim3")]
pub fn bevy_mesh_from_trimesh(trimesh: &TriMesh) -> Mesh {
    bevy_mesh_from_trimesh_elements(trimesh.vertices(), trimesh.indices())
}

#[cfg(feature = "dim3")]
pub fn bevy_mesh_from_trimesh_elements(vertices: &[Point<f32>], indices: &[[u32; 3]]) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        Default::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(
            vertices
                .iter()
                .map(|vertex| [vertex.x, vertex.y, vertex.z])
                .collect::<Vec<_>>(),
        ),
    );

    // Compute vertex normals by averaging the normals of every triangle they appear in.
    // NOTE: This is a bit shonky, but good enough for visualisation.
    let mut normals: Vec<Vec3> = vec![Vec3::ZERO; vertices.len()];
    for triangle in indices.iter() {
        let ab = vertices[triangle[1] as usize] - vertices[triangle[0] as usize];
        let ac = vertices[triangle[2] as usize] - vertices[triangle[0] as usize];
        let normal = ab.cross(&ac);
        // Contribute this normal to each vertex in the triangle.
        for i in 0..3 {
            normals[triangle[i] as usize] += Vec3::new(normal.x, normal.y, normal.z);
        }
    }
    let normals: Vec<[f32; 3]> = normals
        .iter()
        .map(|normal| {
            let normal = normal.normalize();
            [normal.x, normal.y, normal.z]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
    // There's nothing particularly meaningful we can do
    // for this one without knowing anything about the overall topology.
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::from(
            vertices
                .iter()
                .map(|_vertex| [0.0, 0.0])
                .collect::<Vec<_>>(),
        ),
    );
    mesh.insert_indices(Indices::U32(
        indices
            .iter()
            .flat_map(|triangle| triangle.iter())
            .chain(indices.iter().flat_map(|triangle| triangle.iter().rev()))
            .cloned()
            .collect(),
    ));
    mesh
}

#[cfg(feature = "dim2")]
pub fn bevy_mesh_from_trimesh(trimesh: &TriMesh) -> Mesh {
    bevy_mesh_from_trimesh_elements(trimesh.vertices(), Some(trimesh.indices().to_vec()))
}

#[cfg(feature = "dim2")]
pub fn bevy_mesh_from_trimesh_elements(
    vertices: &[Point<f32>],
    mut indices: Option<Vec<[u32; 3]>>,
) -> Mesh {
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        Default::default(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(
            vertices
                .iter()
                .map(|vertex| [vertex.x, vertex.y, 0.0])
                .collect::<Vec<_>>(),
        ),
    );

    if indices.is_none() {
        indices = Some(
            (1..vertices.len() as u32 - 1)
                .map(|i| [0, i, i + 1])
                .collect(),
        );
    }

    mesh.insert_indices(Indices::U32(
        indices
            .unwrap()
            .iter()
            .flat_map(|triangle| triangle.iter())
            .cloned()
            .collect(),
    ));

    let normals: Vec<_> = (0..vertices.len()).map(|_| [0.0, 0.0, 1.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, VertexAttributeValues::from(normals));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::from(
            (0..vertices.len())
                .map(|_vertex| [0.0, 0.0])
                .collect::<Vec<_>>(),
        ),
    );

    mesh
}
