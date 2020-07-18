use crate::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn texture_dir() -> PathBuf {
	PathBuf::from("assets/textures")
}

static NEXT_UID: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
pub struct Texture {
	img: Image<BGRA>,
	uid: usize,
}

impl Texture {
	pub fn new(img: Image<BGRA>) -> Self {
		Self {
			img,
			uid: NEXT_UID.fetch_add(1, Ordering::SeqCst),
		}
	}

	pub fn load(basename: &str) -> Result<Texture> {
		let mut path = texture_dir().join(basename);
		if path.extension() == None {
			path.set_extension("png");
		}
		Ok(Texture::new(Image::<BGRA>::load(&path)?))
	}

	pub fn load_many(basenames: &[&str]) -> Result<Vec<Texture>> {
		//let vec = Vec::with_capacity(basenames.len());
		basenames.iter().map(|x| Self::load(x)).collect()
	}

	pub fn default() -> Self {
		Self {
			uid: 0,
			img: Image::default(),
		}
	}

	pub fn is_none(&self) -> bool {
		self.uid == 0
	}

	pub fn uid(&self) -> usize {
		self.uid
	}

	pub fn dimensions(&self) -> (i32, i32) {
		self.img.dimensions()
	}

	pub fn raw_bgra(&self) -> Vec<u8> {
		self.img.raw_bgra()
	}
}
