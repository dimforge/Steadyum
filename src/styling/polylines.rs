use bevy::prelude::*;
use bevy_polyline::polyline::Polyline;

#[cfg(feature = "dim2")]
pub fn cuboid_polyline() -> Polyline {
    let vtx = [
        Vec3::new(0.5, 0.5, 0.0),
        Vec3::new(-0.5, 0.5, 0.0),
        Vec3::new(-0.5, -0.5, 0.0),
        Vec3::new(0.5, -0.5, 0.0),
    ];

    Polyline {
        vertices: vec![vtx[0], vtx[1], vtx[2], vtx[3], vtx[0]],
    }
}

#[cfg(feature = "dim3")]
pub fn cuboid_polyline() -> Polyline {
    let vtx = [
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.5, 0.5, -0.5),
        Vec3::new(0.5, -0.5, -0.5),
        Vec3::new(0.5, -0.5, 0.5),
        Vec3::new(-0.5, 0.5, 0.5),
        Vec3::new(-0.5, 0.5, -0.5),
        Vec3::new(-0.5, -0.5, -0.5),
        Vec3::new(-0.5, -0.5, 0.5),
    ];

    Polyline {
        vertices: vec![
            vtx[0], vtx[1], vtx[2], vtx[3], vtx[0], vtx[4], vtx[5], vtx[1], vtx[5], vtx[6], vtx[2],
            vtx[6], vtx[7], vtx[3], vtx[7], vtx[4],
        ],
    }
}
