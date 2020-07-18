use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::iter::FromIterator;

/// Serialization / Deserialization format for a single game level.
#[derive(Serialize, Deserialize)]
pub struct LevelData {
	/// 2D map of blocks, each represented by a number 0-255.
	pub map_bytes: ByteMap,
	pub goodies: Vec<(Pt, u8)>,
	pub lights: Lights,
}

// TODO: embed in editor?
// or return tuple, save(&map, &...)?
impl LevelData {
	/// Empty level.
	pub fn new() -> Self {
		Self {
			map_bytes: ByteMap::new(),
			lights: Lights::new(),
			goodies: Vec::new(),
		}
	}

	/// Save as JSON.
	pub fn save(p: &Path, map: &ByteMap, goodies: &FnvHashMap<Pt, u8>, l: &Lights) -> Result<()> {
		let data = Self::from(map, goodies, l);
		let f = File::create(p)?;
		let mut b = BufWriter::new(f);
		serde_json::to_writer(&mut b, &data)?;
		b.flush()?;
		println!("wrote {}", p.to_string_lossy());
		Ok(())
	}

	/// Load from JSON, e.g. "assets/levels/level1.json".
	pub fn load(p: &Path) -> Result<Self> {
		check_exists(p)?;
		let f = File::open(p)?;
		let b = BufReader::new(f);
		let mut data: Self = serde_json::from_reader(b)?;
		data.lights.sun_dir.normalize(); // in case it got hand-edited
		Ok(data)
	}

	fn from(map_bytes: &ByteMap, goodies: &FnvHashMap<Pt, u8>, lights: &Lights) -> Self {
		Self {
			map_bytes: map_bytes.clone(), // TODO: don't clone
			lights: lights.clone(),
			goodies: Self::map_to_vec(goodies),
		}
	}

	/// return the goodies as a hashmap.
	pub fn goodies_map(&self) -> FnvHashMap<Pt, u8> {
		FnvHashMap::from_iter(self.goodies.iter().map(|x| x.clone()))
	}

	// copy a map into a list of (key, value) pairs.
	fn map_to_vec<K: Clone, V: Clone>(map: &FnvHashMap<K, V>) -> Vec<(K, V)> {
		map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
	}
}
