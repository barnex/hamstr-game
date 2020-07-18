use crate::prelude::*;
use serde::{Deserialize, Serialize};

/// A 2D grid of blocks, representing
/// the "static" (non-moving) part of a game level.
pub struct Map {
	inner: ByteMap, // maps position -> byte. TODO: rename "blocks"
	goodies: FnvHashMap<Pt, u8>,
	renderer: RefCell<Renderer>, // maps byte -> texture
	block_types: Vec<BlockTyp>,
}

impl Map {
	/// Construct an empty Map.
	pub fn new() -> Self {
		Self::from(
			ByteMap::new(),
			FnvHashMap::<Pt, u8>::default(),
			Lights::new(),
		)
	}

	/// Construct a Map from a sparse 2D byte array representing the blocks.
	/// Used during deserialization.
	pub fn from(bytes: ByteMap, goodies: FnvHashMap<Pt, u8>, lights: Lights) -> Self {
		Self {
			inner: bytes,
			goodies,
			renderer: RefCell::new(Renderer::new(default_palette(), lights)),
			block_types: block_types(),
		}
	}

	// TODO: remove!
	pub fn clone(&self) -> Self {
		Self::from(
			self.inner.clone(),
			self.goodies.clone(),
			self.renderer.borrow().lights(),
		)
	}

	pub fn bytemap(&self) -> &ByteMap {
		&self.inner
	}

	pub fn goodies(&self) -> &FnvHashMap<Pt, u8> {
		&self.goodies
	}

	pub fn lights(&self) -> Lights {
		self.renderer.borrow().lights()
	}

	// Start caching replacement tiles for every block in this map.
	// When the map is actually being rendered, replacement tiles will be available more rapidly.
	pub fn warmup_cache(&self) {
		let mut renderer = self.renderer.borrow_mut();
		for y in 0..self.inner.blocks.len() {
			for x in 0..self.inner.blocks[y].len() {
				let p = Pt(x as i32, y as i32);
				// render but drop result, only to populate the cache.
				let _ = renderer.render_tile(TileKey::with_center(self.inner.at(p)));
			}
		}
	}

	// TODO: return &Texture?
	pub fn texture_at(&self, p: Pt) -> Rc<Texture> {
		let mut renderer = self.renderer.borrow_mut();
		renderer.render_tile(self.tile_key(p))
	}

	pub fn type_at(&self, p: Pt) -> BlockTyp {
		self.type_of(self.at(p))
	}

	pub fn type_of(&self, blk: u8) -> BlockTyp {
		self.block_types[blk as usize]
	}

	#[inline]
	fn at(&self, p: Pt) -> u8 {
		self.inner.at(p)
	}

	pub fn set(&mut self, p: Pt, blk: u8) {
		match self.type_of(blk) {
			BlockTyp::Goody => {
				self.goodies.insert(p, blk);
			}
			_ => {
				self.inner.set(p, blk);
			}
		}
	}

	pub fn goodie_at(&self, p: Pt) -> u8 {
		match self.goodies.get(&p) {
			None => 0,
			Some(g) => *g,
		}
	}

	pub fn set_goodie(&mut self, p: Pt, goodie: u8) {
		match goodie {
			0 => {
				self.goodies.remove(&p);
			}
			g => {
				self.goodies.insert(p, g);
			}
		}
	}

	fn tile_key(&self, grid: Pt) -> TileKey {
		let (ix, iy) = (grid.0, grid.1);
		let mut k = TileKey::new();
		for cy in 0..3 {
			for cx in 0..3 {
				let mx = ix + cx - 1;
				let my = iy + cy - 1;
				let cx = cx as usize;
				let cy = cy as usize;
				k.blocks[cy][cx] = self.at(Pt(mx, my));
			}
		}

		if let Some(g) = self.goodies.get(&grid) {
			k.goody = *g;
		}

		k
	}

	// TODO: use
	pub fn print_stats(&self) {
		self.renderer.borrow().print_stats();
	}
}

/// Infinite 2D array of blocks.
#[derive(Clone, Serialize, Deserialize)]
pub struct ByteMap {
	pub blocks: Vec<Vec<u8>>,
}

impl ByteMap {
	/// New empty map.
	pub fn new() -> Self {
		Self { blocks: Vec::new() }
	}

	#[inline]
	pub fn at(&self, grid: Pt) -> u8 {
		let (x, y) = (grid.0, grid.1);

		// disallow 0 so that we can never bump into negative positions
		// (where pos / GRID does not simply give the grid cell).
		if x <= 1 || y <= 1 {
			return Self::OUT_OF_BOUNDS_BLOCK;
		}
		let x = x as usize;
		let y = y as usize;

		if y >= self.blocks.len() {
			return 0;
		}
		if x >= self.blocks[y].len() {
			return 0;
		}

		self.blocks[y][x]
	}

	/// block returned for the "negative" (x,y <1 ) part of the map.
	const OUT_OF_BOUNDS_BLOCK: u8 = 1; // hydrogen-brick

	/// Set block at position p.
	/// p must be strictly positive.
	pub fn set(&mut self, grid: Pt, b: u8) {
		let (x, y) = (grid.0, grid.1);
		if x < 0 || y < 0 {
			panic!("SparseImg.set: Pt out of bounds: {:?}", (x, y));
		}
		let x = x as usize;
		let y = y as usize;

		if y >= self.blocks.len() {
			self.blocks.reserve(y - self.blocks.len() + 1);
			while y >= self.blocks.len() {
				self.blocks.push(Vec::new());
			}
		}
		if x >= self.blocks[y].len() {
			self.blocks.reserve(x - self.blocks[y].len() + 1);
			while x >= self.blocks[y].len() {
				self.blocks[y].push(0);
			}
		}
		self.blocks[y][x] = b;
	}

	//pub fn replace<F: Fn(Block) -> Block>(&mut self, range: (Pt, Pt), f: F) {
	//	for y in (range.0).1..(range.1).1 {
	//		for x in (range.0).0..(range.1).0 {
	//			let p = Pt(x, y);
	//			let orig = self[p];
	//			let new = f(orig);
	//			if new != orig {
	//				self.set(p, new);
	//			}
	//		}
	//	}
	//}
}

//impl ops::Index<(i32, i32)> for SparseImg<T> {
//	type Output = Block;
//	fn index(&self, p: (i32, i32)) -> &Self::Output {
//		let (x, y) = (p.0, p.1);
//
//		// disallow 0 so that we can never bump into negative positions
//		// (where pos / GRID does not simply give the grid cell).
//		if x <= 0 || y <= 0 {
//			return &1; // Block outside of map
//		}
//		let x = x as usize;
//		let y = y as usize;
//
//		if y >= self.blocks.len() {
//			return &0;
//		}
//		if x >= self.blocks[y].len() {
//			return &0;
//		}
//
//		&self.blocks[y][x]
//	}
//}
//impl Display for ByteMap {
//	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
//		for row in &self.blocks {
//			write!(f, "|")?;
//			for b in row {
//				write!(f, "{} ", b)?;
//			}
//			write!(f, "\n")?;
//		}
//		Ok(())
//	}
//}
