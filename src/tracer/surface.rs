use crate::prelude::*;

/// 3D texture.
pub struct Surface {
	/// Diffuse map
	pub dm: Image<BGRA>,

	/// Height map
	pub hm: Image<u8>,

	/// Minimum and maximum of hm (Cached because it's used a lot during ray tracing).
	hm_min: u8,
	hm_max: u8,
}

impl Surface {
	/// Construct a surface from heightmap and diffuse map.
	pub fn new(hm: Image<u8>, dm: Image<BGRA>) -> Self {
		let hm_max = hm.pixels().iter().fold(0, |p, x| max(p, *x));
		let hm_min = hm.pixels().iter().fold(255, |p, x| min(p, *x));
		Self {
			hm,
			dm,
			hm_min,
			hm_max,
		}
	}

	/// Load a surface from heightmap and diffuse map files with given base name.
	/// ".hm.png" and ".dm.png" will be appended to find the heightmap and diffuse map
	/// files, respectively.
	pub fn load(base: &Path) -> Result<Self> {
		let hm = Image::<u8>::load(base.with_extension("hm.png"))?;
		let dm = Image::<BGRA>::load(base.with_extension("dm.png"))?;
		Ok(Self::new(hm, dm))
	}

	/// Surface with heightmap from function (heights between 0 and 255),
	/// and uninitialized diffuse map.
	/// Used for testing.
	pub fn from_fn<F: Fn(i32, i32) -> u8>(dim: (i32, i32), f: F) -> Self {
		let hm = Image::from_fn(dim, f);
		let dm = Image::new(dim);
		Self::new(hm, dm)
	}

	/// Maximum height of heightmap.
	/// Used by the ray tracer to stop early when the ray has escaped above the suface.
	pub fn hm_max(&self) -> f64 {
		(self.hm_max as f64) * Self::HM_SCALE
	}

	/// Minimum height of heightmap.
	/// Used by the ray tracer to eliminate surface that cannot cast shadows
	/// because they are fully below other surfaces.
	pub fn hm_min(&self) -> f64 {
		(self.hm_min as f64) * Self::HM_SCALE
	}

	/// Width and height, in pixels.
	#[inline]
	pub fn dimensions(&self) -> (i32, i32) {
		self.hm.dimensions()
	}

	pub fn diffuse_at(&self, pix: Int2) -> BGRA {
		self.dm.at((pix.x(), pix.y()))
	}

	pub const HM_MAX: f64 = 0.5;

	/// Height map: Pixel value (0-255) to physical height scaling.
	const HM_SCALE: f64 = (Self::HM_MAX / 255.0);

	/// The height map's height at pixel (x,y).
	#[inline]
	pub fn height_at(&self, pix: Int2) -> f64 {
		(self.hm[pix.y() as usize][pix.x() as usize] as f64) * Self::HM_SCALE
	}

	/// The height map's height at a UV position (between 0 and 1),
	/// using nearest-neighbor interpolation.
	///
	/// TODO: add bilinear interpolation?
	#[inline]
	pub fn height_at_uv(&self, uv: Vec2) -> f64 {
		let (w, h) = self.dimensions();
		let x = (uv.x() * w as f64) as i32;
		let x = clamp(x, w);
		let y = (uv.y() * h as f64) as i32;
		let y = clamp(y, h);
		self.height_at(Int2(x, y))
	}

	/// Normal vector at pixel (x, y).
	pub fn normal_at(&self, pix: Int2) -> Vec3 {
		let (w, h) = self.dimensions();
		let (x, y) = (pix.0, pix.1);

		let yminus = max(y - 1, 0);
		let yplus = min(y + 1, h - 1);
		let xminus = max(x - 1, 0);
		let xplus = min(x + 1, w - 1);

		let partialy =
			(self.height_at(Int2(x, yplus)) - self.height_at(Int2(x, yminus))) / (2.0 / w as f64);
		let partialx =
			(self.height_at(Int2(xplus, y)) - self.height_at(Int2(xminus, y))) / (2.0 / h as f64);
		Vec3(-partialx, -partialy, 1.0).normalized()
	}

	//pub fn pix_to_uv(&self, (x, y): (i32, i32)) -> (f64, f64) {
	//	let (w, h) = self.dimensions();
	//	(x as f64 / w as f64, y as f64 / h as f64)
	//}
}

fn clamp(x: i32, max: i32) -> i32 {
	if x >= max {
		return max - 1;
	}
	if x < 0 {
		return 0;
	}
	return x;
}

impl Default for Surface {
	fn default() -> Self {
		Self {
			hm: Image::default(),
			dm: Image::default(),
			hm_min: 0,
			hm_max: 0,
		}
	}
}
