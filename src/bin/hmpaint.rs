use flux::prelude::*;
use std::path::PathBuf;

const W: i32 = flux::game::gamestate::GRID as i32;

type V = Vector2<f64>;

fn main() {
	paint("one", |_| 1.0);

	paint("sphere", |uv| {
		let xy = (uv - Vector2(0.5, 0.5)) * 4.0;
		sqrt(1.0 - xy.dot(xy))
	});

	paint("cylinder", |uv| {
		let Vector2(x, _y) = (uv - Vector2(0.5, 0.5)) * 4.0;
		max(sqrt(1.0 - sqr(x)), sqrt(1.0 - sqr(x)))
	});
}

fn paint<F: Fn(V) -> f64>(name: &str, f: F) {
	let img = Image::<u8>::from_fn((W, W), |x, y| {
		let u = (x as f64 + 0.5) / (W as f64);
		let v = (y as f64 + 0.5) / (W as f64);
		let c = (clip(f(Vector2(u, v))) * 255.0) as u8;
		//println!("{},{}: {} => {}", x, y, f(u, v), c);
		c
	});
	let path = PathBuf::from("assets/textures").join(PathBuf::from(name).with_extension("hm.png"));
	img.save(&path).expect("saving image");
	println!("wrote {}", &path.to_string_lossy());
}

fn sqrt(x: f64) -> f64 {
	if x < 0.0 {
		return 0.0;
	}
	f64::sqrt(x)
}

fn sqr(x: f64) -> f64 {
	x * x
}

fn clip(x: f64) -> f64 {
	if x > 1.0 {
		return 1.0;
	}
	if x < 0.0 {
		return 0.0;
	}
	if x != x {
		// NaN
		return 0.0;
	}
	x
}
