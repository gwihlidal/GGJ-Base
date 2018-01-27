use geometry::{Advance, Collide, Point, Vector};
use super::Pigeon;

#[derive(Clone)]
pub struct Coop {
    /// The coop's midpoint
    pub position: Point,
    ///  The direction of pigeon emitting, if interacting
    pub direction : Option<f32>
}

impl Coop {
    /// Create a coop with the given vector
    pub fn new(position: Point) -> Coop {
        Coop { position: position, direction: None }
    }

    /// Clicked on coop?
    pub fn update_mouse_click(&mut self, mouse: Point) {
        if self.position.squared_distance_to(&mouse) < self.radius() * self.radius() {
            // Set to Some basically
            self.direction = Some(0f32);
        }
    }

    /// Emit a pigeon if we release the mouse
    pub fn update_mouse_release(&mut self) -> Option<Pigeon> {
        if let Some(emit_dir) = self.direction {
            let mut pigeon = Pigeon::new(Vector::new(self.position, emit_dir));
            let rad_sum = self.radius() + pigeon.radius();
            pigeon.advance(rad_sum);
            self.direction = None;
            return Some(pigeon);
        }
        None
    }

    /// Update the emitting position if a mouse is around
    pub fn update_mouse_move(&mut self, mouse: Point) {
        // Clicked before? Moving the direction
        if self.direction.is_some() {
            let next_dir = mouse - self.position;
            self.direction = Some(next_dir.y.atan2(next_dir.x));
        }
    }
}

impl ::geometry::Position for Coop {
    fn x(&self) -> f32 { self.position.x }
    fn y(&self) -> f32 { self.position.y }
    fn x_mut(&mut self) -> &mut f32 { &mut self.position.x }
    fn y_mut(&mut self) -> &mut f32 { &mut self.position.y }
}

impl Collide for Coop {
    fn radius(&self) -> f32 { 0.1 }
}