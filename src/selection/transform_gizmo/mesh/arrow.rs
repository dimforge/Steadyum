use bevy::prelude::*;
use bevy_rapier::parry::shape::Capsule;

#[derive(Debug, Clone, Copy)]
pub struct Arrow {
    pub radius: f32,
    pub length: f32,
    pub head_radius: f32,
    pub head_length: f32,
}

impl Default for Arrow {
    fn default() -> Self {
        Arrow {
            radius: 0.02,
            length: 0.6,
            head_radius: 0.08,
            head_length: 0.2,
        }
    }
}

impl From<Arrow> for Mesh {
    #[cfg(feature = "dim2")]
    fn from(arrow: Arrow) -> Self {
        use na::point;
        let hl = arrow.length / 2.0;

        let vertices = vec![
            point![arrow.radius, -hl],
            point![arrow.radius, hl],
            point![arrow.head_radius, hl],
            point![0.0, hl + arrow.head_length],
            point![-arrow.head_radius, hl],
            point![-arrow.radius, hl],
            point![-arrow.radius, -hl],
        ];
        let indices = vec![[0u32, 1, 6], [6, 1, 5], [2, 3, 4]];
        crate::utils::bevy_mesh_from_trimesh_elements(&vertices, Some(indices))
    }

    #[cfg(feature = "dim3")]
    fn from(arrow: Arrow) -> Self {
        use bevy_rapier::parry::shape::Cone;

        let mut arrow_capsule = Capsule::new_y(arrow.length / 2.0, arrow.radius).to_trimesh(10, 10);
        let mut arrow_cone = Cone::new(arrow.head_length / 2.0, arrow.head_radius).to_trimesh(10);

        let id_shift = arrow_capsule.0.len() as u32;
        arrow_cone
            .0
            .iter_mut()
            .for_each(|v| v.y += arrow.length / 2.0);
        arrow_cone.1.iter_mut().for_each(|idx| {
            idx[0] += id_shift;
            idx[1] += id_shift;
            idx[2] += id_shift;
        });
        arrow_capsule.0.append(&mut arrow_cone.0);
        arrow_capsule.1.append(&mut arrow_cone.1);
        crate::utils::bevy_mesh_from_trimesh_elements(&arrow_capsule.0, &arrow_capsule.1)
    }
}
