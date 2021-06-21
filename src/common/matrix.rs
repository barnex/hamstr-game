use crate::prelude::*;
use std::ops::*;

/// 3x3 matrix intended for linear transformations.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Matrix<T>(pub Vector<T>, pub Vector<T>, pub Vector<T>);

impl<T> Matrix<T>
where
	T: Default + Copy,
{
	pub fn new() -> Self {
		Matrix(Vector::default(), Vector::default(), Vector::default())
	}
}

impl<T> Index<usize> for Matrix<T> {
	type Output = Vector<T>;

	/// Index returns column i as vector.
	#[inline]
	fn index(&self, i: usize) -> &Self::Output {
		match i {
			0 => &self.0,
			1 => &self.1,
			2 => &self.2,
			_ => panic!("Matrix index out of bounds: {}", i),
		}
	}
}

impl<T> IndexMut<usize> for Matrix<T> {
	/// Index returns column i as vector.
	#[inline]
	fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
		match idx {
			0 => &mut self.0,
			1 => &mut self.1,
			2 => &mut self.2,
			_ => panic!("Matrix index out of bounds: {}", idx),
		}
	}
}

impl<T> Mul<Matrix<T>> for Matrix<T>
where
	T: AddAssign + Add<T, Output = T> + Mul<T, Output = T> + Copy + Default,
{
	type Output = Matrix<T>;

	/// Matrix-Matrix multiplication.
	fn mul(self, rhs: Self) -> Self::Output {
		let mut c = Matrix::new();
		for i in 0..3 {
			for j in 0..3 {
				for k in 0..3 {
					c[i][j] += rhs[i][k] * self[k][j]
				}
			}
		}
		c
	}
}

impl<T> Mul<Vector<T>> for Matrix<T>
where
	T: AddAssign + Add<T, Output = T> + Mul<T, Output = T> + Copy + Default,
{
	type Output = Vector<T>;

	/// Matrix-Vector multiplication.
	fn mul(self, rhs: Vector<T>) -> Self::Output {
		Vector(
			self[0][0] * rhs[0] + self[1][0] * rhs[1] + self[2][0] * rhs[2],
			self[0][1] * rhs[0] + self[1][1] * rhs[1] + self[2][1] * rhs[2],
			self[0][2] * rhs[0] + self[1][2] * rhs[1] + self[2][2] * rhs[2],
		)
	}
}

impl<T> Mul<T> for Matrix<T>
where
	T: AddAssign + Add<T, Output = T> + Mul<T, Output = T> + Copy + Default,
{
	type Output = Matrix<T>;

	/// Matrix-scalar multiplication.
	fn mul(self, rhs: T) -> Self::Output {
		Matrix(self.0 * rhs, self.1 * rhs, self.2 * rhs)
	}
}

/*
// Inverse returns the inverse matrix.
func (m *Matrix) Inverse() Matrix {
	a := m[0][0]
	b := m[1][0]
	c := m[2][0]
	d := m[0][1]
	e := m[1][1]
	f := m[2][1]
	g := m[0][2]
	h := m[1][2]
	i := m[2][2]

	A := e*i - f*h
	B := f*g - d*i
	C := d*h - e*g
	inv := Matrix{
		{e*i - f*h, f*g - d*i, d*h - e*g},
		{c*h - b*i, a*i - c*g, b*g - a*h},
		{b*f - c*e, c*d - a*f, a*e - b*d},
	}
	det := a*A + b*B + c*C
	return inv.Mulf(1 / det)
}
*/

/*
func ExampleMatrix_Mul() {
	theta := 45 * math.Pi / 180
	c := math.Cos(theta)
	s := math.Sin(theta)
	a := Matrix{{c, s, 0}, {-s, c, 0}, {0, 0, 1}}
	fmt.Printf("% 4.1f", a.Mul(&a))

	//Output:
	// [[ 0.0  1.0  0.0] [-1.0  0.0  0.0] [ 0.0  0.0  1.0]]
}

func ExampleMatrix_Mul_2() {
	R := Matrix{{0, 1, 0}, {-1, 0, 0}, {0, 0, 1}}
	F := Matrix{{-1, 0, 0}, {0, 1, 0}, {0, 0, 1}}
	fmt.Printf("% 4.1f\n", R.Mul(&F))
	fmt.Printf("% 4.1f\n", F.Mul(&R))

	//Output:
	// [[ 0.0 -1.0  0.0] [-1.0  0.0  0.0] [ 0.0  0.0  1.0]]
	// [[ 0.0  1.0  0.0] [ 1.0  0.0  0.0] [ 0.0  0.0  1.0]]
}

func ExampleMatrix_MulVec() {
	theta := 30 * math.Pi / 180
	c := math.Cos(theta)
	s := math.Sin(theta)

	m := Matrix{{c, s, 0}, {-s, c, 0}, {0, 0, 1}}
	fmt.Printf("% 3f\n", m.MulVec(Vec{1, 0, 0}))
	fmt.Printf("% 3f\n", m.MulVec(Vec{0, 1, 0}))
	fmt.Printf("% 3f\n", m.MulVec(Vec{0, 0, 1}))

	//Output:
	// [ 0.866025  0.500000  0.000000]
	// [-0.500000  0.866025  0.000000]
	// [ 0.000000  0.000000  1.000000]
}

func ExampleMatrix_Inverse() {
	m := Matrix{{1, 2, 3}, {3, -1, 2}, {2, 3, -1}}
	inv := m.Inverse()
	check := inv.Mul(&m)

	for i := range check {
		for j, v := range check[i] {
			if math.Abs(v) < 1e-9 {
				check[i][j] = 0
			}
		}
	}
	fmt.Printf("% 4.3f", check)

	//Output:
	// [[ 1.000  0.000  0.000] [ 0.000  1.000  0.000] [ 0.000  0.000  1.000]]
}
*/
