use rand::Rng;
use super::Size;
use std::ops::{Add, Sub, Mul, Div};

#[derive(Clone, Default, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x: x, y: y }
    }

    pub fn random<R: Rng>(rng: &mut R, bounds: Size) -> Point {
        Point {
            x: rng.gen_range(0.0, bounds.width),
            y: rng.gen_range(0.0, bounds.height)
        }
    }

    pub fn squared_distance_to(&self, target: &Point) -> f32 {
        (self.x - target.x) * (self.x - target.x)
            + (self.y - target.y) * (self.y - target.y)
    }

    pub fn rotate(mut self, radians: f32) -> Point {
        let radius = (self.x * self.x + self.y * self.y).sqrt();
        let point_angle = (self.y / self.x).atan();
        let final_angle = point_angle + radians;
        self.x = final_angle.cos() * radius;
        self.y = final_angle.sin() * radius;
        self
    }

    pub fn translate(mut self, other: &Point) -> Point {
        self.x += other.x;
        self.y += other.y;
        self
    }

    pub fn intersect_circle(self, center: &Point, radius: f32) -> bool {
        (self.x - center.x).powi(2) +
            (self.y - center.y).powi(2) < radius.powi(2)
    }

    pub fn dot(&self, other: &Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn length(&self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(&self) -> Point {
        self.div(self.length())
    }

    pub fn lerp(&self, other: &Point, t: f32) -> Point {
        Point {
            x: (1.0 - t) * self.x + t * other.x,
            y: (1.0 - t) * self.y + t * other.y
        }
    }
}

impl PartialEq for Point {
    fn eq (&self, _rhs: &Self) -> bool {
        (self.x == _rhs.x) && (self.y == _rhs.y)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, _rhs: Point) -> Point {
        Point {
            x: self.x + _rhs.x,
            y: self.y + _rhs.y,
        }
    }
}

impl Add<f32> for Point {
    type Output = Point;

    fn add(self, _rhs: f32) -> Point {
        Point {
            x: self.x + _rhs,
            y: self.y + _rhs,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, _rhs: Point) -> Point {
        Point {
            x: self.x - _rhs.x,
            y: self.y - _rhs.y,
        }
    }
}

impl Sub<f32> for Point {
    type Output = Point;

    fn sub(self, _rhs: f32) -> Point {
        Point {
            x: self.x - _rhs,
            y: self.y - _rhs,
        }
    }
}

impl Mul for Point {
    type Output = Point;

    fn mul(self, _rhs: Point) -> Point {
        Point {
            x: self.x * _rhs.x,
            y: self.y * _rhs.y,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;

    fn mul(self, _rhs: f32) -> Point {
        Point {
            x: self.x * _rhs,
            y: self.y * _rhs,
        }
    }
}

impl Div for Point {
    type Output = Point;

    fn div(self, _rhs: Point) -> Point {
        assert!(_rhs.x != 0f32);
        assert!(_rhs.y != 0f32);
        Point {
            x: self.x / _rhs.x,
            y: self.y / _rhs.y,
        }
    }
}

impl Div<f32> for Point {
    type Output = Point;

    fn div(self, _rhs: f32) -> Point {
        assert!(_rhs != 0f32);
        Point {
            x: self.x / _rhs,
            y: self.y / _rhs,
        }
    }
}