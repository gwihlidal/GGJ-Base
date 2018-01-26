use image::*;

pub struct ScalarField {
	pub values: Vec<f32>,
	pub width: usize,
	pub height: usize,
}

impl ScalarField {
	pub fn new(w: usize, h: usize) -> ScalarField {
		ScalarField {
			values: vec![0f32; w * h],
			width: w,
			height: h,
		}
	}

	pub fn to_image_buffer(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
		let mut res = vec![255u8; (self.width * self.height * 4) as usize];
		for i in 0..self.width * self.height {
			// Leave just green (test)
			res[i * 4 + 0] = 0u8;
			res[i * 4 + 2] = 0u8;
		}
		ImageBuffer::from_raw(self.width as u32, self.height as u32, res).unwrap()
	}
}
