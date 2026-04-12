use bevy::prelude::Transform;
use super::rapier::math::Pose;

/// Convert a rapier Pose to a Bevy Transform.
#[cfg(feature = "dim3")]
pub fn pose_to_transform(pose: &Pose) -> Transform {
    Transform {
        translation: pose.translation,
        rotation: pose.rotation.into(),
        scale: bevy::math::Vec3::ONE,
    }
}

/// Convert a rapier Pose to a Bevy Transform.
#[cfg(feature = "dim2")]
pub fn pose_to_transform(pose: &Pose) -> Transform {
    Transform {
        translation: pose.translation.extend(0.0),
        rotation: bevy::math::Quat::from_rotation_z(pose.rotation.angle()),
        scale: bevy::math::Vec3::ONE,
    }
}

/// Convert a Bevy Transform to a rapier Pose.
#[cfg(feature = "dim3")]
pub fn transform_to_pose(transform: &Transform) -> Pose {
    Pose::from_parts(transform.translation, transform.rotation.into())
}

/// Convert a Bevy Transform to a rapier Pose.
#[cfg(feature = "dim2")]
pub fn transform_to_pose(transform: &Transform) -> Pose {
    let (axis, angle) = transform.rotation.to_axis_angle();
    let angle = if axis.z < 0.0 { -angle } else { angle };
    Pose::new(transform.translation.truncate(), angle)
}
