use std::cmp::PartialEq;
use std::fmt;
use std::ops::*;

// TODO: remove, should be Pt
pub type Int2 = Vector2<i32>;

#[inline]
#[allow(non_snake_case)]
pub fn Int2(x: i32, y: i32) -> Int2 {
	Vector2(x, y)
}

pub type Usize2 = Vector2<usize>;

#[inline]
#[allow(non_snake_case)]
pub fn Usize2(x: usize, y: usize) -> Usize2 {
	Vector2(x, y)
}

/// Vector2 is a 2-component vector.
/// Used to represent either points in space or 3D vectors.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Vector2<T>(pub T, pub T);

pub type Vec2 = Vector2<f64>;

#[inline]
#[allow(non_snake_case)]
pub fn Vec2(x: f64, y: f64) -> Vec2 {
	Vector2(x, y)
}

impl Vector2<f64> {
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
		(self.len() - 1.0).abs() < 1e-6
	}

	pub fn is_finite(&self) -> bool {
		self.0.is_finite() && self.1.is_finite()
	}

	/// Shorthand for Vector2(0.0, 0.0).
	pub const ZERO: Self = Vector2(0.0, 0.0);

	/// Shorthand for Vector2(1.0, 0.0).
	pub const EX: Self = Vector2(1.0, 0.0);

	/// Shorthand for Vector2(0.0, 1.0).
	pub const EY: Self = Vector2(0.0, 1.0);
}

impl<T> Vector2<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy,
{
	#[inline]
	pub fn fmadd(self, t: T, b: Self) -> Self {
		self + (b * t)
	}

	#[inline]
	pub fn dot(self, rhs: Self) -> T {
		self.0 * rhs.0 + self.1 * rhs.1
	}

	#[inline]
	pub fn x(&self) -> T {
		self.0
	}
	#[inline]
	pub fn y(&self) -> T {
		self.1
	}
}

impl Vector2<f32> {
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
		(self.len() - 1.0).abs() < 1e-5
	}
}

impl<T> Default for Vector2<T>
where
	T: Default,
{
	fn default() -> Self {
		Vector2(T::default(), T::default())
	}
}

impl<T> Add for Vector2<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn add(self, rhs: Vector2<T>) -> Self::Output {
		Vector2(self.0 + rhs.0, self.1 + rhs.1)
	}
}

impl<T> Add<(T, T)> for Vector2<T>
where
	T: Add<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn add(self, rhs: (T, T)) -> Self::Output {
		Vector2(self.0 + rhs.0, self.1 + rhs.1)
	}
}

impl<T> AddAssign for Vector2<T>
where
	T: AddAssign + Copy,
{
	#[inline]
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
		self.1 += rhs.1;
	}
}

impl<T> fmt::Display for Vector2<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "({}, {})", self.0, self.1)
	}
}

//impl<T> Index<usize> for Vector<T> {
//	type Output = T;
//	#[inline]
//	fn index(&self, idx: usize) -> &Self::Output {
//		&self.el[idx]
//	}
//}
//
//impl<T> IndexMut<usize> for Vector<T> {
//	#[inline]
//	fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
//		&mut self.el[idx]
//	}
//}

impl<T> Mul<T> for Vector2<T>
where
	T: Mul<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn mul(self, rhs: T) -> Self::Output {
		Vector2(self.0 * rhs, self.1 * rhs)
	}
}

impl<T> MulAssign<T> for Vector2<T>
where
	T: MulAssign + Copy,
{
	#[inline]
	fn mul_assign(&mut self, rhs: T) {
		self.0 *= rhs;
		self.1 *= rhs;
	}
}

impl Mul<Vector2<f64>> for f64 {
	type Output = Vector2<f64>;
	#[inline]
	fn mul(self, rhs: Vector2<f64>) -> Self::Output {
		rhs.mul(self)
	}
}

impl Mul<Vector2<f32>> for f32 {
	type Output = Vector2<f32>;
	#[inline]
	fn mul(self, rhs: Vector2<f32>) -> Self::Output {
		rhs.mul(self)
	}
}

impl<T> Neg for Vector2<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Self;
	#[inline]
	fn neg(self) -> Self::Output {
		Vector2(-self.0, -self.1)
	}
}

impl<T> Sub for Vector2<T>
where
	T: Sub<T, Output = T> + Copy,
{
	type Output = Self;

	#[inline]
	fn sub(self, rhs: Vector2<T>) -> Self::Output {
		Vector2(self.0 - rhs.0, self.1 - rhs.1)
	}
}

impl<T> SubAssign for Vector2<T>
where
	T: SubAssign + Copy,
{
	#[inline]
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
		self.1 -= rhs.1;
	}
}
