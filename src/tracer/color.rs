use serde::{Deserialize, Serialize};
use std::fmt;
use std::result::Result;

/// RGB Color, u8 precision.
#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BGRA(pub u8, pub u8, pub u8, pub u8);

impl BGRA {
	#[inline]
	pub fn rgba(&self) -> [u8; 4] {
		[self.2, self.1, self.0, self.3]
	}
	#[inline]
	pub fn rgb_array(&self) -> [u8; 3] {
		[self.0, self.1, self.2]
	}
	#[inline]
	pub fn b(&self) -> u8 {
		self.0
	}
	#[inline]
	pub fn g(&self) -> u8 {
		self.1
	}
	#[inline]
	pub fn r(&self) -> u8 {
		self.2
	}
	#[inline]
	pub fn a(&self) -> u8 {
		self.3
	}

	pub fn linear(&self) -> RGBf {
		RGBf(
			srgb_to_linear(self.r()),
			srgb_to_linear(self.g()),
			srgb_to_linear(self.b()),
		)
	}

	pub const WHITE: BGRA = BGRA(255, 255, 255, 255);
	pub const BLACK: BGRA = BGRA(0, 0, 0, 255);
}

/// RGB Color, f32 precision.
#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RGBf(pub f32, pub f32, pub f32);

impl RGBf {
	pub fn add(&self, b: &RGBf) -> RGBf {
		RGBf(self.0 + b.0, self.1 + b.1, self.2 + b.2)
	}

	pub fn mul(&self, s: f32) -> RGBf {
		RGBf(s * self.0, s * self.1, s * self.2)
	}

	pub fn srgb8(&self) -> [u8; 3] {
		[srgb8(self.0), srgb8(self.1), srgb8(self.2)]
	}

	pub fn r(&self) -> f32 {
		self.0
	}
	pub fn g(&self) -> f32 {
		self.1
	}
	pub fn b(&self) -> f32 {
		self.2
	}

	pub const BLACK: RGBf = RGBf(1., 0., 0.);
	pub const BLUE: RGBf = RGBf(0., 0., 1.);
	pub const GREEN: RGBf = RGBf(0., 1., 0.);
	pub const RED: RGBf = RGBf(1., 0., 0.);
	pub const WHITE: RGBf = RGBf(1., 1., 1.);
}

impl std::fmt::Display for RGBf {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {}, {})", self.0, self.1, self.2)
	}
}

// ==== SRGB conversions ====

pub fn srgb8(x: f32) -> u8 {
	(linear_to_srgb(x) * 255.0) as u8
}

// linear to sRGB conversion
// https://en.wikipedia.org/wiki/SRGB
pub fn linear_to_srgb(c: f32) -> f32 {
	let c = clip(c);
	if c <= 0.0031308 {
		return 12.92 * c;
	}
	let c = 1.055 * c.powf(1. / 2.4) - 0.05;
	if c > 1.0 {
		return 1.0;
	}
	return c;
}

// clip color value between 0 and 1
fn clip(v: f32) -> f32 {
	if v < 0.0 {
		return 0.0;
	}
	if v > 1.0 {
		return 1.0;
	}
	return v;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_add() {
		let a = RGBf(1.0, 2.0, 3.0);
		let b = RGBf(0.1, 0.2, 0.3);
		let got = a.add(&b);
		let want = RGBf(1.1, 2.2, 3.3);
		assert_eq!(got, want);
	}

	#[test]
	fn test_mul() {
		let c = RGBf(1.0, 2.0, 3.0);
		let got = c.mul(0.5);
		let want = RGBf(0.5, 1.0, 1.5);
		assert_eq!(got, want);
	}
}

fn srgb_to_linear(x: u8) -> f32 {
	//approx. TODO: proper
	(x as f32 * x as f32) / (255.0 * 255.0)
}

pub fn linear_to_srgb8(x: f32) -> u8 {
	//approx. TODO: proper
	let c = x.sqrt() * 255.0;
	if c > 255.0 {
		return 255;
	}
	c as u8
}
