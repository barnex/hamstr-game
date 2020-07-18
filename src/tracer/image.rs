extern crate image;
use crate::prelude::*;
use std::io;
use std::ops::{Index, IndexMut};

/// An Image is a rectangular 2D array of color values
/// (RGB, grayscale, ...)
#[derive(Debug, PartialEq, Clone)]
pub struct Image<C> {
	dim: (usize, usize),
	values: Vec<C>,
}

impl<'a, C> Image<C>
where
	C: Copy + Default,
{
	/// new constructs an image with given width and height.
	pub fn new((w, h): (i32, i32)) -> Image<C> {
		Image {
			dim: (w as usize, h as usize),
			values: vec![C::default(); w as usize * h as usize],
		}
	}

	pub fn from_fn<F: Fn(i32, i32) -> C>((w, h): (i32, i32), f: F) -> Self {
		let mut img = Self::new((w, h));
		for iy in 0..(h as usize) {
			for ix in 0..(w as usize) {
				img[iy][ix] = f(ix as i32, iy as i32);
			}
		}
		img
	}

	pub fn at(&self, p: (i32, i32)) -> C {
		self[p.1 as usize][p.0 as usize]
	}

	/// width of the image, in pixels
	pub fn width(&self) -> usize {
		self.dim.0
	}

	/// height of the image, in pixels
	pub fn height(&self) -> usize {
		self.dim.1
	}

	/// width and height of the image
	pub fn dimensions(&self) -> (i32, i32) {
		(self.dim.0 as i32, self.dim.1 as i32)
	}

	/// pixels in row-major order, iterable.
	pub fn pixels(&self) -> &[C] {
		&self.values
	}

	/// pixels in row-major order, iterable.
	pub fn pixels_mut(&mut self) -> &mut [C] {
		&mut self.values
	}
}

impl<C> Default for Image<C>
where
	C: Copy + Default,
{
	fn default() -> Self {
		Self {
			dim: (0, 0),
			values: Vec::new(),
		}
	}
}

impl<C> Index<usize> for Image<C>
where
	C: Copy + Default,
{
	type Output = [C];

	fn index(&self, i: usize) -> &[C] {
		let l = i * self.width();
		let h = l + self.width();
		&self.values[l..h]
	}
}

impl<C> IndexMut<usize> for Image<C>
where
	C: Copy + Default,
{
	fn index_mut(&mut self, i: usize) -> &mut [C] {
		let l = i * self.width();
		let h = l + self.width();
		&mut self.values[l..h]
	}
}

impl Image<BGRA> {
	/// Save image to file. E.g.:
	/// 	img.save("file.png")?;
	pub fn save<P: AsRef<Path>>(&self, p: P) -> io::Result<()> {
		let w = self.width() as u32;
		let h = self.height() as u32;
		let img = image::ImageBuffer::from_fn(w, h, |x, y| {
			image::Rgba(self[y as usize][x as usize].rgba())
		});
		img.save(p)
	}

	pub fn load<P: AsRef<Path>>(p: P) -> Result<Self> {
		check_exists(p.as_ref())?;
		let src = image::io::Reader::open(p)?.decode()?.into_rgba();
		let mut dst = Self::new((src.width() as i32, src.height() as i32));
		for (x, y, c) in src.enumerate_pixels() {
			dst[y as usize][x as usize] = BGRA(c[0], c[1], c[2], c[3]);
		}
		Ok(dst)
	}

	pub fn raw_bgra(&self) -> Vec<u8> {
		let (w, h) = self.dimensions();
		let mut raw = Vec::with_capacity((w * h * 4) as usize);
		for iy in 0..h {
			for ix in 0..w {
				let c = self[iy as usize][ix as usize];
				raw.push(c.2);
				raw.push(c.1);
				raw.push(c.0);
				raw.push(c.3);
			}
		}
		raw
	}
}

impl Image<RGBf> {
	/// Save image to file. E.g.:
	/// 	img.save("file.png")?;
	pub fn save<P: AsRef<Path>>(&self, p: P) -> io::Result<()> {
		let w = self.width() as u32;
		let h = self.height() as u32;
		let img = image::ImageBuffer::from_fn(w, h, |x, y| {
			image::Rgb(self[y as usize][x as usize].srgb8())
		});
		img.save(p)
	}

	pub fn raw_bgra(&self) -> Vec<u8> {
		let (w, h) = self.dimensions();
		let mut raw = Vec::with_capacity((w * h * 4) as usize);
		for iy in 0..h {
			for ix in 0..w {
				let c = self[iy as usize][ix as usize];
				raw.push(srgb8(c.2)); //b
				raw.push(srgb8(c.1)); //g
				raw.push(srgb8(c.0)); //r
				raw.push(255); //a
			}
		}
		raw
	}
}

impl Image<u8> {
	pub fn load<P: AsRef<Path>>(p: P) -> Result<Self> {
		check_exists(p.as_ref())?;
		let src = image::io::Reader::open(p)?.decode()?.into_luma();
		let mut dst = Image::<u8>::new((src.width() as i32, src.height() as i32));
		for (x, y, c) in src.enumerate_pixels() {
			dst[y as usize][x as usize] = c[0];
		}
		Ok(dst)
	}

	/// Save image to file. E.g.:
	/// 	img.save("file.png")?;
	pub fn save<P: AsRef<Path>>(&self, p: P) -> io::Result<()> {
		let w = self.width() as u32;
		let h = self.height() as u32;
		let img = image::ImageBuffer::from_fn(w, h, |x, y| {
			let c = self[y as usize][x as usize];
			image::Rgb([c, c, c])
		});
		img.save(p)
	}

	/// Convert to BGRA
	pub fn to_bgra(&self) -> Image<BGRA> {
		Image::<BGRA>::from_fn(self.dimensions(), |x, y| {
			let c = self.at((x, y));
			BGRA(c, c, c, 255)
		})
	}
}
