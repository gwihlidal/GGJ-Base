use geometry::{Collide, Vector};
use geometry::point::{Point};
use std::f32;

#[derive(Clone)]
pub struct Trajectory {
    pub points: Vec<Point>
}

#[derive(Clone)]
pub struct Pigeon {
    pub vector: Vector,
    pub trajectory: Option<Trajectory>,
    trajectory_pos: usize,
}

#[derive(PartialEq)]
pub enum PigeonStatus {
    JustPigeoning,
    ReachedDestination,
}

derive_position_direction!(Pigeon);

impl Pigeon {
    /// Create a pigeon with the given vector
    pub fn new(vector: Vector) -> Pigeon {
        Pigeon { vector, trajectory: None, trajectory_pos: 0 }
    }

    pub fn calculate_rotation(move_vec: Point) -> f32 {
        f32::atan2(move_vec.y, move_vec.x) + f32::consts::PI * -0.5f32
    }

    /// Update the pigeon's position
    pub fn update(&mut self, units: f32) -> PigeonStatus {
        let prev_pos = self.vector.position;
        let mut units = units;

        if let Some(ref traj) = self.trajectory {
            let mut target = self.vector.position;

            while self.trajectory_pos < traj.points.len() {
                target = traj.points[self.trajectory_pos];
                let dist = (target - self.vector.position).length();
                if dist < units {
                    self.vector.position = target;
                    units -= dist;
                    self.trajectory_pos += 1;
                } else {
                    break;
                }
            }

            target = target - self.vector.position;
            let dist = target.length();
            if dist > units {
                target = target * units / dist;
            }

            self.vector.position = self.vector.position + target;

            if (prev_pos - self.vector.position).length() < 1e-3 {
                PigeonStatus::ReachedDestination
            } else {
                self.vector.direction = Pigeon::calculate_rotation(target);
                PigeonStatus::JustPigeoning
            }
        } else {
            PigeonStatus::JustPigeoning
        }
    }
}

impl Collide for Pigeon {
    fn radius(&self) -> f32 { 0.1 }
}