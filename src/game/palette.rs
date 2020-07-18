use crate::prelude::*;

/// Block definitions in the order as they will appear in the editor.
pub const ED_PALETTE: [BlockDef; 29] = [
	BlockDef {
		uid: 0, // empty
		srf: "",
		walk: Wall,
	},
	BlockDef {
		uid: 28,
		srf: "seed",
		walk: Goody,
	},
	// hydrogen
	BlockDef {
		uid: 17,
		srf: "hydrogen-grass",
		walk: Wall,
	},
	BlockDef {
		uid: 8,
		srf: "hydrogen-wall-deep",
		walk: Wall,
	},
	BlockDef {
		uid: 2,
		srf: "hydrogen-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 27,
		srf: "hydrogen-wall-red",
		walk: Wall,
	},
	BlockDef {
		uid: 3,
		srf: "hydrogen-ledge",
		walk: Ledge,
	},
	BlockDef {
		uid: 26,
		srf: "hydrogen-brick-deep",
		walk: Brick,
	},
	BlockDef {
		uid: 1,
		srf: "hydrogen-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 4,
		srf: "hydrogen-top",
		walk: Brick,
	},
	// lithium
	BlockDef {
		uid: 23,
		srf: "lithium-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 24,
		srf: "lithium-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 25,
		srf: "lithium-top",
		walk: Brick,
	},
	// magnesium
	BlockDef {
		uid: 12,
		srf: "magnesium-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 13,
		srf: "magnesium-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 21,
		srf: "magnesium-top",
		walk: Brick,
	},
	// iron
	BlockDef {
		uid: 18,
		srf: "fe-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 19,
		srf: "fe-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 20,
		srf: "fe-top",
		walk: Brick,
	},
	BlockDef {
		uid: 22,
		srf: "fe-grass",
		walk: Wall,
	},
	// helium
	BlockDef {
		uid: 11,
		srf: "helium-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 9,
		srf: "helium-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 10,
		srf: "helium-top",
		walk: Brick,
	},
	// xenon
	BlockDef {
		uid: 7,
		srf: "xenon-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 5,
		srf: "xenon-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 6,
		srf: "xenon-top",
		walk: Brick,
	},
	// silicon
	BlockDef {
		uid: 14,
		srf: "silicon-wall",
		walk: Wall,
	},
	BlockDef {
		uid: 15,
		srf: "silicon-brick",
		walk: Brick,
	},
	BlockDef {
		uid: 16,
		srf: "silicon-top",
		walk: Brick,
	},
];

#[derive(Copy, Clone, Default)]
pub struct BlockDef {
	pub uid: u8,
	pub walk: BlockTyp,
	srf: &'static str,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BlockTyp {
	Wall,
	Ledge,
	Brick,
	Goody,
}

impl Default for BlockTyp {
	fn default() -> Self {
		Wall
	}
}

use BlockTyp::*;

pub fn block_types() -> Vec<BlockTyp> {
	let mut s = zero_vec(ED_PALETTE.len());
	for def in &ED_PALETTE {
		s[def.uid as usize] = def.walk;
	}
	s
}

pub fn default_palette() -> Vec<Surface> {
	load_palette(&texture_dir()).unwrap()
}

fn load_palette(texture_dir: &Path) -> Result<Vec<Surface>> {
	let mut s = zero_vec(ED_PALETTE.len());

	// surface 0 is fully transparent.
	let dim = (GRID as i32, GRID as i32);
	s[0] = Surface::new(Image::<u8>::new(dim), Image::<BGRA>::new(dim));

	for def in ED_PALETTE.iter().skip(1) {
		let base = texture_dir.join(def.srf);
		s[def.uid as usize] = Surface::load(&base)?;
	}
	Ok(s)
}

fn zero_vec<T: Default>(len: usize) -> Vec<T> {
	let mut s = Vec::with_capacity(len);
	for _ in 0..len {
		s.push(T::default());
	}
	s
}

#[test]
fn test_load_palette() {
	load_palette(&PathBuf::from("assets/textures")).expect("loading palette");
}
