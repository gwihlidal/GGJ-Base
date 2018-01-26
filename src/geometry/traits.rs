use std::f32;

use super::{Point, Size};

pub trait Position {
    fn x(&self) -> f32;

    fn x_mut(&mut self) -> &mut f32;

    fn y(&self) -> f32;

    fn y_mut(&mut self) -> &mut f32;

    fn position(&self) -> Point {
        Point::new(self.x(), self.y())
    }
}

pub trait Advance: Position {
    fn direction(&self) -> f32;

    fn direction_mut(&mut self) -> &mut f32;

    fn point_to(&mut self, target: Point) {
        let m = (self.y() - target.y) / (self.x() - target.x);

        *self.direction_mut() = if target.x > self.x() {
            m.atan()
        } else {
            m.atan() + f32::consts::PI
        };
    }

    fn advance(&mut self, units: f32) {
        *self.x_mut() += self.direction().cos() * units;
        *self.y_mut() += self.direction().sin() * units;
    }

    fn advance_wrapping(&mut self, units: f32, bounds: Size) {
        self.advance(units);

        fn wrap(k: &mut f32, bound: f32) {
            if *k < 0.0 {
                *k += bound;
            } else if *k >= bound {
                *k -= bound;
            }
        }

        wrap(self.x_mut(), bounds.width);
        wrap(self.y_mut(), bounds.height);
    }
}

pub trait Collide: Position {
    fn radius(&self) -> f32;

    fn diameter(&self) -> f32 {
        self.radius() * 2.0
    }

    fn collides_with<O: Collide>(&self, other: &O) -> bool {
        let radii = self.radius() + other.radius();
        self.position().squared_distance_to(&other.position()) < radii * radii
    }
}