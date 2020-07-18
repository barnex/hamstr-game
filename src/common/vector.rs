use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::fmt;
use std::ops::*;

/// Vector is a 3-component vector.
/// Used to represent either points in space or 3D vectors.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Vector<T>(pub T, pub T, pub T);

pub type Vec3 = Vector<f64>;

#[inline]
#[allow(non_snake_case)]
pub fn Vec3(x: f64, y: f64, z: f64) -> Vec3 {
	Vector(x, y, z)
}

impl Vector<f64> {
	#[inline]
	/// Len returns the length (norm).
	pub fn len(self) -> f64 {
		self.dot(self).sqrt()
	}

	/// Normalized returns a vector with the same direction
	/// but unit length.
	#[inline]
	pub fn normalized(self) -> Self {
		self * (1. / self.len())
	}

	#[inline]
	pub fn normalize(&mut self) {
		*self = self.normalized()
	}

	pub fn is_normalized(&self) -> bool {
		(self.len() - 1.0).abs() < 1e-5
	}

	pub fn is_finite(&self) -> bool {
		self.0.is_finite() && self.1.is_finite() && self.2.is_finite()
	}

	/// Shorthand for Vector(0.0, 0.0, 0.0).
	pub const ZERO: Self = Vector(0.0, 0.0, 0.0);

	/// Shorthand for Vector(1.0, 0.0, 0.0).
	pub const EX: Self = Vector(1.0, 0.0, 0.0);

	/// Shorthand for Vector(0.0, 1.0, 0.0).
	pub const EY: Self = Vector(0.0, 1.0, 0.0);

	/// Shorthand for new(0.0, 0.0, 1.0).
	pub const EZ: Self = Vector(0.0, 0.0, 1.0);
}

impl<T> Vector<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy,
{
	#[inline]
	pub fn fmadd(self, t: T, b: Self) -> Self {
		self + (b * t)
	}

	#[inline]
	pub fn dot(self, rhs: Self) -> T {
		self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2
	}

	#[inline]
	pub fn cross(self, rhs: Self) -> Self {
		Vector(
			self.1 * rhs.2 - self.2 * rhs.1,
			self.2 * rhs.0 - self.0 * rhs.2,
			self.0 * rhs.1 - self.1 * rhs.0,
		)
	}

	#[inline]
	pub fn xy(&self) -> Vector2<T> {
		Vector2(self.x(), self.y())
	}

	#[inline]
	pub fn x(&self) -> T {
		self.0
	}
	#[inline]
	pub fn y(&self) -> T {
		self.1
	}
	#[inline]
	pub fn z(&self) -> T {
		self.2
	}
}

impl Vector<f32> {
	#[inline]
	/// Len returns the length (norm).
	pub fn len(self) -> f32 {
		self.dot(self).sqrt()
	}

	/// Normalized returns a vector with the same direction
	/// but unit length.
	#[inline]
	pub fn normalized(self) -> Self {
		self * (1. / self.len())
	}

	pub fn is_normalized(&self) -> bool {
		(self.len() - 1.0).abs() < 1e-6
	}
}

impl<T> Default for Vector<T>
where
	T: Default,
{
	fn default() -> Self {
		Vector(T::default(), T::default(), T::default())
	}
}

impl<T> Add for Vector<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn add(self, rhs: Vector<T>) -> Self::Output {
		Vector(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
	}
}

impl<T> Add<(T, T, T)> for Vector<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn add(self, rhs: (T, T, T)) -> Self::Output {
		Vector(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
	}
}

impl<T> AddAssign for Vector<T>
where
	T: AddAssign + Copy,
{
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
		self.1 += rhs.1;
		self.2 += rhs.2;
	}
}

impl<T> fmt::Display for Vector<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
		write!(f, "({}, {}, {})", self.0, self.1, self.2)
	}
}

impl<T> Index<usize> for Vector<T> {
	type Output = T;
	#[inline]
	fn index(&self, idx: usize) -> &Self::Output {
		match idx {
			0 => &self.0,
			1 => &self.1,
			2 => &self.2,
			_ => panic!(format!("Vector index out of bounds: {}", idx)),
		}
	}
}

impl<T> IndexMut<usize> for Vector<T> {
	#[inline]
	fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
		match idx {
			0 => &mut self.0,
			1 => &mut self.1,
			2 => &mut self.2,
			_ => panic!(format!("Vector index out of bounds: {}", idx)),
		}
	}
}

impl<T> Mul<T> for Vector<T>
where
	T: Mul<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn mul(self, rhs: T) -> Self::Output {
		Vector(self.0 * rhs, self.1 * rhs, self.2 * rhs)
	}
}

impl<T> MulAssign<T> for Vector<T>
where
	T: MulAssign + Copy,
{
	#[inline]
	fn mul_assign(&mut self, rhs: T) {
		self.0 *= rhs;
		self.1 *= rhs;
		self.2 *= rhs;
	}
}

impl Mul<Vector<f64>> for f64 {
	type Output = Vector<f64>;
	#[inline]
	fn mul(self, rhs: Vector<f64>) -> Self::Output {
		rhs.mul(self)
	}
}

impl Mul<Vector<f32>> for f32 {
	type Output = Vector<f32>;
	#[inline]
	fn mul(self, rhs: Vector<f32>) -> Self::Output {
		rhs.mul(self)
	}
}

impl<T> Neg for Vector<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Self;
	#[inline]
	fn neg(self) -> Self::Output {
		Vector(-self.0, -self.1, -self.2)
	}
}

impl<T> Sub for Vector<T>
where
	T: Sub<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Vector<T>) -> Self::Output {
		Vector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
	}
}

impl<T> SubAssign for Vector<T>
where
	T: SubAssign + Copy,
{
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
		self.1 -= rhs.1;
		self.2 -= rhs.2;
	}
}
