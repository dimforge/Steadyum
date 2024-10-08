use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
/// A torus (donut) shape.
#[derive(Debug, Clone, Copy)]
pub struct TruncatedTorus {
    pub radius: f32,
    pub ring_radius: f32,
    pub subdivisions_segments: usize,
    pub subdivisions_sides: usize,
    pub angle: f32,
}

impl Default for TruncatedTorus {
    fn default() -> Self {
        TruncatedTorus {
            radius: 1.0,
            ring_radius: 0.5,
            subdivisions_segments: 32,
            subdivisions_sides: 24,
            angle: std::f32::consts::PI / 2.0,
        }
    }
}

#[cfg(feature = "dim2")]
impl From<TruncatedTorus> for Mesh {
    fn from(torus: TruncatedTorus) -> Self {
        use na::point;

        let r1 = torus.radius - torus.ring_radius;
        let r2 = torus.radius + torus.ring_radius;
        let mut vertices = vec![];
        let mut indices = vec![];
        let dtheta = 2.0 * std::f32::consts::PI / (torus.subdivisions_segments - 1) as f32;

        for i in 0..torus.subdivisions_segments {
            let (s, c) = ((i as f32) * dtheta).sin_cos();
            vertices.push(point![c * r1, s * r1]);
        }

        for i in 0..torus.subdivisions_segments {
            let (s, c) = ((i as f32) * dtheta).sin_cos();
            vertices.push(point![c * r2, s * r2]);
        }

        for i in 0..torus.subdivisions_segments - 1 {
            let a = i;
            let b = i + torus.subdivisions_segments;
            let c = i + 1 + torus.subdivisions_segments;
            let d = i + 1;
            indices.push([a as u32, b as u32, c as u32]);
            indices.push([a as u32, c as u32, d as u32]);
        }

        crate::utils::bevy_mesh_from_trimesh_elements(&vertices, Some(indices))
    }
}

#[cfg(feature = "dim3")]
impl From<TruncatedTorus> for Mesh {
    fn from(torus: TruncatedTorus) -> Self {
        // code adapted from http://apparat-engine.blogspot.com/2013/04/procedural-meshes-torus.html
        // (source code at https://github.com/SEilers/Apparat)

        let n_vertices = (torus.subdivisions_segments + 1) * (torus.subdivisions_sides + 1);
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_vertices);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n_vertices);

        let segment_stride = torus.angle / torus.subdivisions_segments as f32;
        let side_stride = 2.0 * std::f32::consts::PI / torus.subdivisions_sides as f32;

        for segment in 0..=torus.subdivisions_segments {
            let theta = segment_stride * segment as f32;
            let segment_pos = Vec3::new(theta.cos(), 0.0, theta.sin() * torus.radius);

            for side in 0..=torus.subdivisions_sides {
                let phi = side_stride * side as f32;

                let x = theta.cos() * (torus.radius + torus.ring_radius * phi.cos());
                let z = theta.sin() * (torus.radius + torus.ring_radius * phi.cos());
                let y = torus.ring_radius * phi.sin();

                let normal = segment_pos.cross(Vec3::Y).normalize();

                positions.push([x, y, z]);
                normals.push(normal.into());
                uvs.push([
                    segment as f32 / torus.subdivisions_segments as f32,
                    side as f32 / torus.subdivisions_sides as f32,
                ]);
            }
        }

        let n_faces = (torus.subdivisions_segments) * (torus.subdivisions_sides);
        let n_triangles = n_faces * 2;
        let n_indices = n_triangles * 3;

        let mut indices: Vec<u32> = Vec::with_capacity(n_indices);

        let n_vertices_per_row = torus.subdivisions_sides + 1;
        for segment in 0..torus.subdivisions_segments {
            for side in 0..torus.subdivisions_sides {
                let lt = side + segment * n_vertices_per_row;
                let rt = (side + 1) + segment * n_vertices_per_row;

                let lb = side + (segment + 1) * n_vertices_per_row;
                let rb = (side + 1) + (segment + 1) * n_vertices_per_row;

                indices.push(lt as u32);
                indices.push(rt as u32);
                indices.push(lb as u32);

                indices.push(rt as u32);
                indices.push(rb as u32);
                indices.push(lb as u32);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
        mesh.insert_indices(Indices::U32(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
