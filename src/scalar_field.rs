use image::*;
use geometry::point::{Point};

pub struct ScalarField {
	pub values: Vec<f32>,
	pub width: usize,
	pub height: usize,
}

pub fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
	if x < lo {
		lo
	} else if x > hi {
		hi
	} else {
		x
	}
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
	let x = clamp((x - edge0) / (edge1 - edge0), 0f32, 1f32); 
	return x * x * (3f32 - 2f32 * x)
}

impl ScalarField {
	pub fn new(w: usize, h: usize) -> ScalarField {
		ScalarField {
			values: vec![0f32; w * h],
			width: w,
			height: h,
		}
	}

	pub fn splat(&mut self, splat_x: u32, splat_y: u32, radius: f32) {
		for y in 0..self.height {
			for x in 0..self.width {
				let xd = x as f32 - splat_x as f32;
				let yd = y as f32 - splat_y as f32;
				self.values[y * self.width + x] += smoothstep(radius, 0.0f32, (xd * xd + yd * yd).sqrt());
			}
		}
	}

	pub fn to_image_buffer(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
		let mut res = vec![0u8; (self.width * self.height * 4) as usize];
		for i in 0..self.width * self.height {
			res[i * 4 + 0] = (smoothstep(0.0, 1.0, self.values[i]) * 0.8f32 * 255f32) as u8;
			res[i * 4 + 1] = (smoothstep(0.0, 0.5, self.values[i]) * 0.8f32 * 255f32) as u8;
			res[i * 4 + 3] = 255u8;
		}
		ImageBuffer::from_raw(self.width as u32, self.height as u32, res).unwrap()
	}

	pub fn sample_gradient(&self, x: f32, y: f32) -> Point {
        let x0 = (x * self.width as f32) as usize;
        let y0 = (y * self.height as f32) as usize;

        let h00 = self.values[y0 * self.width + x0];
        let h01 = self.values[y0 * self.width + x0 + 1];
        let h10 = self.values[y0 * self.width + x0 + self.width];
        //let h11 = self.values[y0 * self.width + x0 + self.width + 1];

		Point::new(h00 - h01, h00 - h10)
	}
}
