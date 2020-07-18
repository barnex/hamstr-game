use std::cmp::Ordering;
use std::cmp::PartialOrd;
use std::default::Default;

pub fn signum(x: i32) -> i32 {
	match x.cmp(&0) {
		Ordering::Equal => 0,
		Ordering::Greater => 1,
		Ordering::Less => -1,
	}
}

pub fn abs(x: i32) -> i32 {
	if x < 0 {
		-x
	} else {
		x
	}
}

pub fn clamp(x: i32, min: i32, max: i32) -> i32 {
	debug_assert!(max >= min);
	let mut x = x;
	if x < min {
		x = min
	}
	if x > max {
		x = max
	}
	x
}

/// Rectify: return x if > 0, 0 otherwise
///
///     use flux::common::prelude::*;
///     assert_eq!(re(1.0), 1.0);
///     assert_eq!(re(-1.0), 0.0);
///
pub fn re<T: PartialOrd + Default>(x: T) -> T {
	if x > T::default() {
		x
	} else {
		T::default()
	}
}

pub fn max<T: PartialOrd>(x: T, y: T) -> T {
	if x > y {
		x
	} else {
		y
	}
}

pub fn min<T: PartialOrd>(x: T, y: T) -> T {
	if x < y {
		x
	} else {
		y
	}
}

pub fn modf(x: f64) -> (f64, f64) {
	let floor = x.floor();
	(floor, x - floor)
}
