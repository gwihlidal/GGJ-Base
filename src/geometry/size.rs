use rand::Rng;
use super::Point;

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32
}

impl Size {
    pub fn new(width: f32, height: f32) -> Size {
        Size { width: width, height: height }
    }

    pub fn contains(&self, point: Point) -> bool {
        0.0 <= point.x && point.x <= self.width
            && 0.0 <= point.y && point.y <= self.height
    }

    pub fn random_x<R: Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(0.0, self.width)
    }

    pub fn random_y<R: Rng>(&self, rng: &mut R) -> f32 {
        rng.gen_range(0.0, self.height)
    }
}