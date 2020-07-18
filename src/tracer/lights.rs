use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Lights {
	pub sun_dir: Vec3,
	pub sun_intens: RGBf,
	pub sun_angle: f64,
	pub sun_rays: usize,
	pub ambient: RGBf,
	pub ambient_rays: usize,
	pub fake_ambient: RGBf,
	pub invert_dm: bool,
}

impl Lights {
	pub fn new() -> Self {
		Self {
			sun_dir: Vec3(1.0, -1.0, 0.7).normalized(),
			sun_intens: RGBf(0.7, 0.4, 0.2).mul(0.4),
			sun_angle: 0.25,
			sun_rays: 7,
			ambient: RGBf(0.5, 0.6, 0.9).mul(0.6),
			ambient_rays: 31,
			fake_ambient: RGBf(0.5, 0.5, 0.6).mul(0.0),
			invert_dm: false,
		}
	}

	pub fn sample_sun_dir(&self, (u, v): (f64, f64)) -> Vector<f64> {
		let (x, y) = uniform_disk((u, v));
		let dir = make_basis(self.sun_dir) * Vec3(x, y, 1.0) * self.sun_angle + self.sun_dir;
		dir.normalized()
	}
}
