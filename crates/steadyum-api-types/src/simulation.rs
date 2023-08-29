use rapier::geometry::Aabb;
use rapier::math::{Point, Real, DIM};
use rapier::na::vector;
use std::cmp::Ordering;
use std::process::Command;

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SimulationBounds {
    pub mins: [i64; DIM],
    pub maxs: [i64; DIM],
}

impl Default for SimulationBounds {
    fn default() -> Self {
        Self {
            mins: [-10_000; DIM],
            maxs: [10_000; DIM],
        }
    }
}

impl PartialOrd for SimulationBounds {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for k in 0..DIM {
            if self.mins[k] < other.mins[k] {
                return Some(Ordering::Less);
            } else if self.mins[k] > other.mins[k] {
                return Some(Ordering::Greater);
            }
        }

        Some(Ordering::Equal)
    }
}

impl Ord for SimulationBounds {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl SimulationBounds {
    pub const DEFAULT_WIDTH: u64 = 100;

    pub fn from_aabb(aabb: &Aabb, region_width: u64) -> Self {
        Self::from_point(aabb.maxs, region_width)
    }

    pub fn from_point(point: Point<Real>, region_width: u64) -> Self {
        let mins = point
            .coords
            .map(|e| (e / region_width as Real).floor() as i64)
            * (region_width as i64);
        let maxs = mins.add_scalar(region_width as i64);

        Self {
            mins: mins.into(),
            maxs: maxs.into(),
        }
    }

    pub fn intersecting_aabb(aabb: Aabb, region_width: u64) -> Vec<Self> {
        let mut result = vec![];
        let min_region_id = aabb
            .mins
            .coords
            .map(|e| (e / region_width as Real).floor() as i64);
        let max_region_id = aabb
            .maxs
            .coords
            .map(|e| (e / region_width as Real).ceil() as i64);

        #[cfg(feature = "dim2")]
        for i in min_region_id.x..max_region_id.x {
            for j in min_region_id.y..max_region_id.y {
                let mins = vector![i, j] * region_width as i64;
                let maxs = mins.add_scalar(region_width as i64);
                result.push(Self {
                    mins: mins.into(),
                    maxs: maxs.into(),
                });
            }
        }

        #[cfg(feature = "dim3")]
        for i in min_region_id.x..max_region_id.x {
            for j in min_region_id.y..max_region_id.y {
                for k in min_region_id.z..max_region_id.z {
                    let mins = vector![i, j, k] * region_width as i64;
                    let maxs = mins.add_scalar(region_width as i64);
                    result.push(Self {
                        mins: mins.into(),
                        maxs: maxs.into(),
                    });
                }
            }
        }

        result
    }

    pub fn aabb(&self) -> Aabb {
        Aabb {
            mins: Point::from(self.mins).cast::<Real>(),
            maxs: Point::from(self.maxs).cast::<Real>(),
        }
    }

    pub fn is_in_smaller_region(&self, aabb: &Aabb) -> bool {
        Self::from_aabb(aabb, Self::DEFAULT_WIDTH) < *self
    }

    pub fn intersects_master_region(&self, aabb: &Aabb) -> bool {
        Self::from_aabb(aabb, Self::DEFAULT_WIDTH) > *self
    }

    #[cfg(feature = "dim2")]
    pub fn zenoh_queue_key(&self) -> String {
        format!(
            "runner/{}_{}__{}_{}",
            self.mins[0], self.mins[1], self.maxs[0], self.maxs[1]
        )
    }

    #[cfg(feature = "dim3")]
    pub fn zenoh_queue_key(&self) -> String {
        format!(
            "runner/{}_{}_{}__{}_{}_{}",
            self.mins[0], self.mins[1], self.mins[2], self.maxs[0], self.maxs[1], self.maxs[2]
        )
    }

    pub fn watch_kvs_key(&self) -> String {
        format!("watch/{}", self.zenoh_queue_key())
    }

    pub fn runner_key(&self) -> String {
        self.zenoh_queue_key()
    }

    pub fn command(&self, exe_file: &str, curr_step: u64, port: u32) -> Command {
        #[cfg(feature = "dim2")]
        let args = [
            "--xmin".to_owned(),
            format!("{}", self.mins[0]),
            "--ymin".to_owned(),
            format!("{}", self.mins[1]),
            "--xmax".to_owned(),
            format!("{}", self.maxs[0]),
            "--ymax".to_owned(),
            format!("{}", self.maxs[1]),
            "--time-origin".to_owned(),
            format!("{}", curr_step),
            "--port".to_owned(),
            format!("{port}"),
        ];
        #[cfg(feature = "dim3")]
        let args = [
            "--xmin".to_owned(),
            format!("{}", self.mins[0]),
            "--ymin".to_owned(),
            format!("{}", self.mins[1]),
            "--zmin".to_owned(),
            format!("{}", self.mins[2]),
            "--xmax".to_owned(),
            format!("{}", self.maxs[0]),
            "--ymax".to_owned(),
            format!("{}", self.maxs[1]),
            "--zmax".to_owned(),
            format!("{}", self.maxs[2]),
            "--time-origin".to_owned(),
            format!("{}", curr_step),
            "--port".to_owned(),
            format!("{port}"),
        ];
        let mut command = Command::new(exe_file);
        command.args(args);
        command
    }

    #[cfg(feature = "dim2")]
    pub fn neighbors_to_watch(&self) -> [Self; 3] {
        let mut result = [*self; 3];
        let mut curr = 0;

        for i in 0..=1 {
            for j in 0..=1 {
                if i == 0 && j == 0 {
                    continue; // Exclude self.
                }

                let width = [
                    (self.maxs[0] - self.mins[0]) * i,
                    (self.maxs[1] - self.mins[1]) * j,
                ];

                let adj_region = Self {
                    mins: [self.mins[0] + width[0], self.mins[1] + width[1]],
                    maxs: [self.maxs[0] + width[0], self.maxs[1] + width[1]],
                };

                result[curr] = adj_region;
                curr += 1;
            }
        }

        result
    }

    #[cfg(feature = "dim3")]
    pub fn neighbors_to_watch(&self) -> [Self; 13] {
        let mut result = [*self; 13];
        let mut curr = 0;

        for i in -1..=1 {
            for j in -1..=1 {
                for k in -1..=1 {
                    if i == 0 && j == 0 && k == 0 {
                        continue; // Exclude self.
                    }

                    let width = [
                        (self.maxs[0] - self.mins[0]) * i,
                        (self.maxs[1] - self.mins[1]) * j,
                        (self.maxs[2] - self.mins[2]) * k,
                    ];

                    let adj_region = Self {
                        mins: [
                            self.mins[0] + width[0],
                            self.mins[1] + width[1],
                            self.mins[2] + width[2],
                        ],
                        maxs: [
                            self.maxs[0] + width[0],
                            self.maxs[1] + width[1],
                            self.maxs[2] + width[2],
                        ],
                    };

                    if adj_region > *self {
                        result[curr] = adj_region;
                        curr += 1;
                    }
                }
            }
        }

        result
    }
}
