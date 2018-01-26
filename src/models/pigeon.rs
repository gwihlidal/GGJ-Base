use geometry::{Advance, Collide, Vector};

#[derive(Clone)]
pub struct Pigeon {
    vector: Vector
}

derive_position_direction!(Pigeon);

impl Pigeon {
    /// Create a pigeon with the given vector
    pub fn new(vector: Vector) -> Pigeon {
        Pigeon { vector: vector }
    }

    /// Update the pigeon's position
    pub fn update(&mut self, units: f32) {
        self.advance(units);
    }
}

impl Collide for Pigeon {
    fn radius(&self) -> f32 { 3.0 }
}