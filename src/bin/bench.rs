//use flux::prelude::*;
//use std::time::Instant;

fn main() {}

//fn render() {
//	let pal = default_palette();
//	let lights = Lights::new();
//	//lights.ambient_rays = 0;
//	//lights.sun_rays = 1;
//	let mut b = Bakery::new(pal, lights);
//
//	for i in 0..NUM_BLOCKS {
//		let i = i as u8;
//		let start = now();
//		let key = [[0, 0, 0], [0, i, 0], [0, 0, 0]];
//		b.send(key);
//		b.recv(key);
//		report_since(&format!("render {}:", i), start);
//	}
//}

//fn now() -> Instant {
//	Instant::now()
//}
//
//fn report_since(name: &str, start: Instant) {
//	println!("{}: {} s", name, start.elapsed().as_secs_f32());
//}
