use crate::prelude::*;

/// A Ray is a half-line representing a line of light.
#[derive(Copy, Clone)]
pub struct Ray {
	pub start: Vector<f64>,
	pub dir: Vector<f64>,
}

impl Ray {
	#[inline]
	pub fn new(start: Vector<f64>, dir: Vector<f64>) -> Self {
		debug_assert!(start.is_finite());
		debug_assert!(dir.is_normalized());
		Ray { start, dir }
	}

	#[inline]
	pub fn at(self, t: f64) -> Vector<f64> {
		self.start.fmadd(t, self.dir)
	}

	/// is_valid returns true if the ray only contains
	/// non-NaN and non-Infinite numbers, and has an approximatley
	/// normalized direction.
	///    
	///     # use flux::prelude::*;
	///     assert!( Ray{start: Vector(1.,2.,3.),    dir: Vector(1.,0.,0.)}.is_valid());
	///     assert!(!Ray{start: Vector(1.,2.,3.),    dir: Vector(2.,0.,0.)}.is_valid());
	///     assert!(!Ray{start: Vector(0./0.,2.,3.), dir: Vector(1.,0.,0.)}.is_valid());
	///
	/// Intended for use with debug_assert!
	pub fn is_valid(&self) -> bool {
		self.start.is_finite() && self.dir.is_normalized()
	}
}
