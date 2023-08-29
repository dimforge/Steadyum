use rapier::math::{AngVector, Isometry, Real, Rotation, Vector};
use std::ops::{Add, Mul};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct KinematicCurve<T> {
    pub control_points: Vec<T>,
    pub t0: Real,
    pub total_time: Real,
    pub loop_back: bool,
}

impl<T> KinematicCurve<T> {
    pub fn eval(&self, t: Real) -> T
    where
        T: Copy + Mul<Real, Output = T> + Add<T, Output = T>,
    {
        if t < self.t0 {
            self.control_points[0]
        } else if t > self.total_time && !self.loop_back {
            *self.control_points.last().unwrap()
        } else {
            let t = t - self.t0;
            let loop_id = (t / self.total_time).floor() as i32;
            let rel_t = if loop_id % 2 == 1 {
                (1.0 - (t / self.total_time).fract()) * self.total_time
            } else {
                (t / self.total_time).fract() * self.total_time
            };

            let time_slices = self.total_time / (self.control_points.len() - 1) as Real;
            let curr_time_slice = (rel_t / time_slices).floor() as usize;
            let rel_slice_t = (rel_t / time_slices).fract();

            self.control_points[curr_time_slice] * (1.0 - rel_slice_t)
                + self.control_points[curr_time_slice + 1] * rel_slice_t
        }
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct KinematicAnimations {
    pub linear: Option<KinematicCurve<Vector<Real>>>,
    pub angular: Option<KinematicCurve<AngVector<Real>>>,
}

impl KinematicAnimations {
    pub fn eval(&self, t: Real, base: Isometry<Real>) -> Isometry<Real> {
        let mut result = base;

        if let Some(linear) = &self.linear {
            result.translation.vector = linear.eval(t);
        }

        if let Some(angular) = &self.angular {
            result.rotation = Rotation::new(angular.eval(t));
        }

        result
    }
}
