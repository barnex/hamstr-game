use crate::prelude::*;
extern crate num_cpus;
extern crate rand;
use rand::Rng;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use std::time::Instant;

/// Caching ray tracer.
pub struct Renderer {
	cache: FnvHashMap<TileKey, Rc<Texture>>,
	baking: FnvHashSet<TileKey>,
	empty: Rc<Texture>,

	palette: Arc<Palette>,

	/// Worker pool for async ray-tracing
	bakery: Bakery,
}

// The palette maps block id's (0-225, elements of a Map) to 3D surfaces.
type Palette = Vec<Surface>;

////////////////////////////////////////////////////////////////////////////////////  This is the caching part

impl Renderer {
	pub fn new(palette: Vec<Surface>, lights: Lights) -> Self {
		let palette = Arc::new(palette);
		Renderer {
			cache: FnvHashMap::default(),
			baking: FnvHashSet::default(),
			bakery: Bakery::new(palette.clone(), lights),
			palette,
			empty: Rc::new(Texture::default()),
		}
	}

	pub fn lights(&self) -> Lights {
		self.bakery.shared_data.lights.clone()
	}

	/// Renders and returns the Texture for the central tile in Tilekey.
	/// Returns a low-quality replacement or even empty texture if the texture is not yet done baking.
	pub fn render_tile(&mut self, tilekey: TileKey) -> Rc<Texture> {
		// empty block
		//if tilekey.center_empty() {
		//	return self.empty.clone();
		//}

		// ignore neighboaring surfaces that cannot throw a shadow.
		let tilekey = self.canonicalize(tilekey);

		// already baked
		if let Some(tex) = self.cache.get(&tilekey) {
			return tex.clone();
		}

		// currently baking: check if done
		if self.baking.contains(&tilekey) {
			if let Some(rctex) = self.try_recv(tilekey) {
				return rctex;
			}
		}

		// not yet started: start baking
		if !self.is_baking(tilekey) {
			self.start_baking(tilekey);
		}

		// the requested texture is not availbe yet
		// return a low quality replacement:
		//  - center block ignoring neighbor's shadows, if available
		//  - empty otherwise
		if tilekey.is_center_only() {
			self.empty.clone()
		} else {
			self.render_tile(tilekey.center_and_goody())
		}
	}

	/// Replace TileKey by the simplest TileKey that will render into the same result,
	/// by removing neighboring blocks that can never cast a shadow on the centeral block
	/// because they have strictly lower height.
	///
	/// This significantly reduces the number of tiles to render and keep in memory.
	fn canonicalize(&self, k: TileKey) -> TileKey {
		let c = k.center();
		let mut k = k.clone();
		k.blocks[0][0] = self.canonicalize1(k.blocks[0][0], c);
		k.blocks[0][1] = self.canonicalize1(k.blocks[0][1], c);
		k.blocks[0][2] = self.canonicalize1(k.blocks[0][2], c);
		k.blocks[1][0] = self.canonicalize1(k.blocks[1][0], c);
		k.blocks[1][2] = self.canonicalize1(k.blocks[1][2], c);
		k.blocks[2][0] = self.canonicalize1(k.blocks[2][0], c);
		k.blocks[2][1] = self.canonicalize1(k.blocks[2][1], c);
		k.blocks[2][2] = self.canonicalize1(k.blocks[2][2], c);

		k
	}

	fn canonicalize1(&self, block: u8, center: u8) -> u8 {
		if self.is_below(block, center) {
			0
		} else {
			block
		}
	}

	// is surface a strictly below surface b?
	// if so, a cannot cast a shadow on b.
	fn is_below(&self, surfa: u8, surfb: u8) -> bool {
		self.palette[surfa as usize].hm_max() <= self.palette[surfb as usize].hm_min()
	}

	fn start_baking(&mut self, tilekey: TileKey) {
		self.baking.insert(tilekey); // mark baking
		self.bakery.send(tilekey);
	}

	fn is_baking(&self, tilekey: TileKey) -> bool {
		self.baking.contains(&tilekey)
	}

	fn try_recv(&mut self, tilekey: TileKey) -> Option<Rc<Texture>> {
		match self.bakery.try_recv(tilekey) {
			None => None,
			Some(img) => Some(self.create(tilekey, img)),
		}
	}

	fn create(&mut self, tilekey: TileKey, img: Image<BGRA>) -> Rc<Texture> {
		self.baking.remove(&tilekey);
		self.cache.insert(tilekey, Rc::new(Texture::new(img)));
		self.cache[&tilekey].clone()
	}

	// -------------------------------------------------------------------------------- debug
	pub fn print_stats(&self) {
		println!(
			"texture_manager: baking: {}, inuse: {}",
			self.baking.len(),
			self.cache.len()
		);
		self.bakery.print_stats();
	}
}

/// A 3x3 piece of a Map.
/// Bakery can render the central block, considering shadows from its neighbors.
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct TileKey {
	pub blocks: [[u8; 3]; 3],
	pub goody: u8,
}

impl TileKey {
	pub fn new() -> Self {
		Self::with_center(0)
	}
	pub fn with_center(block: u8) -> Self {
		TileKey {
			blocks: [[0, 0, 0], [0, block, 0], [0, 0, 0]],
			goody: 0,
		}
	}
	pub fn center(self) -> u8 {
		self.blocks[1][1]
	}
	//fn center_only(self) -> Self {
	//	Self::with_center(self.center())
	//}
	fn center_and_goody(self) -> Self {
		TileKey {
			blocks: [[0, 0, 0], [0, self.center(), 0], [0, 0, 0]],
			goody: self.goody,
		}
	}
	fn is_center_only(self) -> bool {
		let mut masked = self;
		masked.blocks[1][1] = 0;
		masked.blocks == Self::new().blocks
	}
	//fn center_empty(&self) -> bool {
	//	self.blocks[1][1] == 0 && self.goody == 0
	//}
}

/// Bakery asynchronously renders ("bakes") lighting effects.
struct Bakery {
	to_work: Sender<TileKey>,
	from_work: Receiver<DoneItem>,
	shared_data: Arc<SharedData>,
	outbox: FnvHashMap<TileKey, Image<BGRA>>,
	num_baking: i32,
}

/// TileKey + rendered image, sent back by worker threads.
type DoneItem = (TileKey, Image<BGRA>);

/// Read-only data needed by worker threads for rendering.
/// TODO: pub only for Editor.
pub struct SharedData {
	palette: Arc<Palette>,
	lights: Lights,
	cpu_millis: AtomicI64,
}

////////////////////////////////////////////////////////////////////////////////////  This is the async part

impl Bakery {
	fn new(palette: Arc<Palette>, lights: Lights) -> Self {
		let shared_data = Arc::new(SharedData::new(palette, lights));
		let (to_work, from_bakery) = mpmc_channel::<TileKey>();
		let (to_bakery, from_work) = channel::<DoneItem>();

		for _i in 0..Self::num_render_threads() {
			// thread-local Arc/channel clones
			let shared_data = Arc::clone(&shared_data);
			let from_bakery = from_bakery.clone();
			let to_bakery = to_bakery.clone();
			spawn(move || {
				for tilekey in from_bakery {
					let img = shared_data.render_central_block(tilekey);
					if to_bakery.send((tilekey, img)).is_err() {
						break;
					}
				}
			});
		}

		Self {
			to_work,
			from_work,
			shared_data,
			outbox: FnvHashMap::default(),
			num_baking: 0,
		}
	}

	fn num_render_threads() -> usize {
		// number of cores - 1 (to leave one free for rendering etc)
		// but no less than 1 core, of course.
		max(num_cpus::get() - 1, 1)
	}

	/// Send work to the Bakery: start asynchronously rendering
	/// TileKey's central tile. The baked image can later be retrieved
	/// through recv() or try_recv().
	fn send(&mut self, tilekey: TileKey) {
		self.num_baking += 1;
		self.to_work.send(tilekey).unwrap();
	}

	/// Return the rendered image corresponding to TileKey if ready, None otherwise.
	/// The TileKey must have been sent(), exactly once, earlier.
	///
	/// TODO: panic if TileKey was not sent earlier, instead of perpetually returning None.
	fn try_recv(&mut self, tilekey: TileKey) -> Option<Image<BGRA>> {
		// move completed items to outbox, if any.
		for item in self.from_work.try_iter() {
			self.num_baking -= 1;
			self.outbox.insert(item.0, item.1);
		}
		// return item from outbox, if present.
		self.outbox.remove(&tilekey)
	}

	// ------------------------------------------------------------------------- debug
	fn print_stats(&self) {
		let cpusecs = self.shared_data.cpu_millis.load(SeqCst) as f64 / 1000.0;
		println!(
			"bakery: baking: {}, outbox: {}, CPU: {} s",
			self.num_baking,
			self.outbox.len(),
			cpusecs,
		);
	}
}

//////////////////////////////////////////////////////////////////////////////////// This is the render part

impl SharedData {
	pub fn new(palette: Arc<Palette>, lights: Lights) -> Self {
		Self {
			palette,
			lights,
			cpu_millis: AtomicI64::new(0),
		}
	}

	pub fn render_central_block(&self, chunk: TileKey) -> Image<BGRA> {
		let start = Instant::now();
		let w = GRID as i32;
		let img = Image::from_fn((w, w), |x, y| self.shade_pix(chunk, Int2(x, y)));
		self.cpu_millis
			.fetch_add(start.elapsed().as_millis() as i64, SeqCst);
		img
	}

	fn shade_pix(&self, chunk: TileKey, pix: Int2) -> BGRA {
		// TODO: not correct w/ goodies
		// TODO: method: empty()
		//if chunk.blocks[1][1] == 0 {
		//	return BGRA(0, 0, 0, 0);
		//}

		//let centre_surf = &self.palette[chunk.center() as usize];

		//let mut dm = centre_surf.diffuse_at(pix);
		let mut dm = self.diffuse_at(chunk, pix);
		//if dm.a() == 0 {
		//	return BGRA(0, 0, 0, 0);
		//}

		if self.lights.invert_dm {
			dm.0 = 255 - dm.0;
			dm.1 = 255 - dm.1;
			dm.2 = 255 - dm.2;
		}

		//let normal = centre_surf.normal_at(pix);
		let normal = self.normal_at(chunk, pix);

		let xy = Self::to_abs_pos(Usize2(1, 1), pix);
		let z = self.height_at(chunk, pix);

		let pos = Vector(xy.x(), xy.y(), z) + 0.02 * normal;

		let mut rng = rand::thread_rng();
		let rnd = (rng.gen::<f64>(), rng.gen::<f64>());

		let ambient = self
			.lights
			.ambient
			.mul(self.ambient_fraction(chunk, pos, normal, rnd) as f32);

		let sunlight = self
			.lights
			.sun_intens
			.mul(self.sun_fraction(chunk, pos, normal, rnd) as f32);

		let total_light = ambient.add(&sunlight).add(&self.lights.fake_ambient);

		let dml = dm.linear();
		//let alpha = dm.a() as f32 / 255.0;
		BGRA(
			linear_to_srgb8(dml.b() * total_light.0),
			linear_to_srgb8(dml.g() * total_light.1),
			linear_to_srgb8(dml.r() * total_light.2),
			dm.a(),
		)
	}

	fn ambient_fraction(&self, chunk: TileKey, pos: Vec3, normal: Vec3, rand: (f64, f64)) -> f64 {
		let mut total_light = 0.0;
		let n = self.lights.ambient_rays;
		for i in 0..n {
			let (u, v) = halton23_scrambled(i, rand);
			let dir = cosine_sphere((u, v), normal);
			let r = Ray::new(pos, dir);
			if !self.intersects(chunk, &r) {
				total_light += 1.0 / (n as f64);
			}
		}

		total_light
	}

	fn sun_fraction(&self, chunk: TileKey, pos: Vec3, normal: Vec3, rand: (f64, f64)) -> f64 {
		let mut total_light = 0.0;
		let n = self.lights.sun_rays;
		for i in 0..n {
			let (u, v) = halton23_scrambled(i, rand);
			let dir = self.lights.sample_sun_dir((u, v));
			let r = Ray::new(pos, dir);
			if !self.intersects(chunk, &r) {
				total_light += re(normal.dot(dir)) / (n as f64);
			}
		}
		total_light
	}

	fn intersects(&self, chunk: TileKey, r: &Ray) -> bool {
		debug_assert!(r.start.x() >= 0.9 && r.start.x() <= 2.1);
		debug_assert!(r.start.y() >= 0.9 && r.start.y() <= 2.1);
		debug_assert!(r.start.z() >= 0.0 && r.start.z() <= 1.0);

		// a ray pointing down will eventually hit something for sure.
		if r.dir.z() <= 0.0 {
			return true;
		}

		// ray marching stride so that we advance 0.7 pixels per step in the XY plane.
		// steeper rays have larger absolute strides.
		let stride = 0.7 / (r.dir.z().cos() * GRID as f64);
		let mut t = stride;

		let maxh = self.max_height(chunk);
		assert!(maxh <= Surface::HM_MAX);
		let n = GRID - 2;
		for _i in 0..n {
			t += stride;
			let p = r.at(t);
			if p.z() > maxh {
				return false;
			}
			if self.height_at_pos(chunk, p.xy()) > p.z() {
				return true;
			}
		}
		false
	}

	// maximum hight of all blocks in this tile.
	// used to abort ray tracing when the ray progresses above this height.
	fn max_height(&self, k: TileKey) -> f64 {
		let mut mx = k
			.blocks
			.iter()
			.flatten()
			.fold(0.0, |b, x| max(b, self.palette[*x as usize].hm_max()));
		if k.goody != 0 {
			mx = max(mx, self.palette[k.goody as usize].hm_max());
		}
		mx
	}

	fn to_abs_pos(tile: Usize2, pix: Int2) -> Vec2 {
		let x = (tile.0 as f64) + (pix.0 as f64 / GRID as f64);
		let y = (tile.1 as f64) + (pix.1 as f64 / GRID as f64);
		Vec2(x, y)
	}

	fn diffuse_at(&self, chunk: TileKey, pix: Int2) -> BGRA {
		let centre_surf = &self.palette[chunk.center() as usize];
		let dm = centre_surf.diffuse_at(pix);

		// https://en.wikipedia.org/wiki/Alpha_compositing
		fn blend(c1: u8, a1: u8, c2: u8, a2: u8) -> u8 {
			let c1 = (c1 as f32) / 255.0;
			let a1 = (a1 as f32) / 255.0;
			let c2 = (c2 as f32) / 255.0;
			let a2 = (a2 as f32) / 255.0;

			(((c1 * a1 + c2 * a2 * (1.0 - a1)) / (a1 + a2 * (1.0 - a1))) * 255.0) as u8
		}

		if chunk.goody != 0 {
			let fg = self.palette[chunk.goody as usize].diffuse_at(pix);
			BGRA(
				blend(fg.b(), fg.a(), dm.b(), dm.a()),
				blend(fg.g(), fg.a(), dm.g(), dm.a()),
				blend(fg.r(), fg.a(), dm.r(), dm.a()),
				max(fg.a(), dm.a()),
			)
		} else {
			dm
		}
	}

	fn normal_at(&self, chunk: TileKey, pix: Int2) -> Vec3 {
		if chunk.goody != 0 {
			let dm = self.palette[chunk.goody as usize].diffuse_at(pix);
			if dm.a() != 0 {
				return self.palette[chunk.goody as usize].normal_at(pix);
			}
		}

		let centre_surf = &self.palette[chunk.center() as usize];
		centre_surf.normal_at(pix)
		// TODO: goodie.
	}

	fn height_at(&self, chunk: TileKey, pix: Int2) -> f64 {
		if chunk.goody != 0 {
			let dm = self.palette[chunk.goody as usize].diffuse_at(pix);
			if dm.a() != 0 {
				return self.palette[chunk.goody as usize].height_at(pix);
			}
		}
		let centre_surf = &self.palette[chunk.center() as usize];
		centre_surf.height_at(pix)
		// TODO: goodie.
	}

	/// Surface height at absolute position.
	#[inline]
	fn height_at_pos(&self, chunk: TileKey, pos: Vec2) -> f64 {
		let (tile, uv) = Self::pos_to_tile(pos);
		let blk = chunk.blocks[tile.1][tile.0];

		if chunk.goody != 0 && tile == Usize2(1, 1) {
			let bg = self.palette[blk as usize].height_at_uv(uv);
			let fg = self.palette[chunk.goody as usize].height_at_uv(uv);
			return max(fg, bg);
		}

		//if blk == 0 {
		//	return 0.0;
		//}
		self.palette[blk as usize].height_at_uv(uv)
	}

	fn pos_to_tile(p: Vec2) -> (Usize2, Vec2) {
		let (tx, remx) = modf(p.x());
		let (ty, remy) = modf(p.y());
		(Usize2(tx as usize, ty as usize), Vec2(remx, remy))
	}
}

// --------------------------------------------------------------------------- tests

#[test]
fn test_pos_to_tile() {
	assert_eq!(
		SharedData::pos_to_tile(Vec2(1.25, 4.125)),
		(Usize2(1, 4), Vec2(0.25, 0.125))
	);
}

#[test]
fn test_intersects() {
	let w = GRID as i32;
	let pal: Vec<Surface> = vec![
		Surface::default(),
		Surface::from_fn((w, w), |x, _y| if x > w / 2 { 255 } else { 0 }),
	];

	let b = SharedData::new(Arc::new(pal), Lights::new());
	let mut chunk = TileKey::new();
	chunk.blocks[0][0] = 1;
	chunk.blocks[1][1] = 1;
	//([[1, 0, 0], [0, 1, 0], [0, 0, 0]]);

	let start = Vector(1.25, 1.5, 0.0);
	assert!(!b.intersects(chunk, &Ray::new(start, Vec3(0.0, 0.0, 1.0))));
	assert!(b.intersects(chunk, &Ray::new(start, Vec3(0.0, 1.0, 0.0)))); // note: degenerate
	assert!(b.intersects(chunk, &Ray::new(start, Vec3(1.0, 0.0, 0.0))));
	assert!(b.intersects(chunk, &Ray::new(start, Vec3(1.0, 0.0, 0.01).normalized())));
	assert!(!b.intersects(chunk, &Ray::new(start, Vec3(1.0, 0.0, 2.0).normalized())));
}

//fn test_shade_pix() {
//	let pal = default_palette();
//	let lights = Lights::new();
//	let b = SharedData::new(Arc::new(pal), lights);
//	let chunk = TileKey([[2, 2, 2], [2, 1, 2], [2, 2, 2]]);
//
//	let w = GRID as i32;
//	let img = Image::from_fn((w, w), |x, y| b.shade_pix(chunk, Int2(x, y)));
//
//	img.save("testdata/output/test_shade_pix.png").unwrap();
//}

#[test]
fn test_try_recv() {
	let pal = default_palette();
	let mut lights = Lights::new();
	lights.sun_rays = 1;
	lights.ambient_rays = 1;

	let mut b = Bakery::new(Arc::new(pal), lights);

	let key = TileKey::with_center(2);
	b.send(key);

	if let Some(_img) = b.try_recv(key) {
		panic!("received too early");
	}

	for _i in 0..100 {
		std::thread::sleep(std::time::Duration::from_millis(10));
		if let Some(_img) = b.try_recv(key) {
			return;
		}
	}
	panic!("did not receive");
}

//#[test]
//fn test_normalmap() {
//
//	let w = GRID as i32;
//	let pal: Vec<Surface> = vec![
//		Surface::default(),
//		Surface::from_fn((w, w), |x, _y| if x > w / 2 { 255 } else { 0 }),
//	];
//
//	let b = Bakery::new(pal, Lights::new());
//	let chunk = [[0, 0, 0], [0, 1, 0], [0, 0, 0]];
//
//
//
//	// a 45 degree ramp along X
//	for iy in 0..h {
//		for ix in 0..w {
//			hm[iy as usize][ix as usize] = ix as f32 + 5.0;
//		}
//	}
//
//	let nm = normal_map(&hm);
//	let sqrt2_2 = f32::sqrt(2.0) / 2.0;
//	for iy in 1..(h - 1) {
//		for ix in 1..(w - 1) {
//			let n = nm[iy as usize][ix as usize];
//			assert_eq!(n, Vec32::new(-sqrt2_2, 0.0, sqrt2_2));
//		}
//	}
//}
