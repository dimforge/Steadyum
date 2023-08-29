#[cfg(feature = "dim2")]
pub use self::camera2d::{OrbitCamera, OrbitCameraPlugin};
#[cfg(feature = "dim3")]
pub use self::camera3d::{OrbitCamera, OrbitCameraPlugin};

#[cfg(feature = "dim2")]
mod camera2d;
#[cfg(feature = "dim3")]
mod camera3d;
