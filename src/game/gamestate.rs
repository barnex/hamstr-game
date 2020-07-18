use crate::prelude::*;
use std::borrow::Borrow;

// TODO: rename: TILE_PIXELS
pub const GRID: usize = 64;

pub struct GameState {
	map: Map,
	hamster: Hamster,
	time: i32,
	key_debouncer: KeyDebouncer,
	view_center: Pt,
}

impl GameState {
	pub fn new(map: Map) -> Self {
		map.warmup_cache();
		Self {
			map,
			hamster: Hamster::new(Pt(2, 2) * GRID),
			time: 0,
			key_debouncer: KeyDebouncer::new(),
			view_center: Pt(0, 0),
		}
	}

	pub fn set_view_center(&mut self, center: Pt) {
		self.view_center = center;
	}

	// ------------------------------------------------------------------------------ draw

	pub fn draw(&self, disp: &mut SDLDisplay) {
		let mut disp = Viewport::with_center(disp, self.view_center);
		disp.clear(BGRA(255, 210, 210, 255));

		let grid = GRID as i32;
		//let mut texman = self.renderer.borrow_mut();

		let ((xmin, ymin), (xmax, ymax)) = disp.visible_blocks();
		for iy in ymin..ymax {
			for ix in xmin..xmax {
				let tileid = Pt(ix, iy);
				//let tex = texman.render_tile(self.tile_key(tileid));
				let tex = self.map.texture_at(tileid);
				let pos = Pt(ix * grid, iy * grid);
				disp.draw_texture(tex.borrow(), pos, false);
			}
		}

		self.hamster.draw(&mut disp, self.time);
	}

	pub fn visible_blocks(center: Pt, disp_dim: (i32, i32)) -> ((i32, i32), (i32, i32)) {
		let disp_dim = Pt(disp_dim.0, disp_dim.1);
		let ptmin = center - disp_dim / 2;
		let ptmax = center + disp_dim / 2;
		let grmin: Pt = ptmin / GRID - (1, 1);
		let grmax: Pt = ptmax / GRID + (1, 1);
		(grmin.as_tuple(), grmax.as_tuple())
	}

	pub fn view_origin(center: Pt, disp_dim: (i32, i32)) -> Pt {
		let disp_dim = Pt(disp_dim.0, disp_dim.1);
		center - disp_dim / 2
	}

	// ------------------------------------------------------------------------------------ tick

	pub fn tick(&mut self) {
		self.time += 1;
		if self.time % 16 == 0 {
			self.print_stats();
		}

		let keys = self.key_debouncer.key_states();
		self.key_debouncer.clear();

		self.hamster.tick(&self.map, self.time, &keys);
		self.handle_triggers();

		self.update_view_center();
	}

	fn update_view_center(&mut self) {
		// lookahead
		self.view_center.0 += (self.hamster.speed().0 * 3) / 2;

		// hysteresis
		let ham = self.hamster.center();
		const D: i32 = (GRID as i32) * 2; // TODO;
		self.view_center.0 = clamp(self.view_center.0, ham.0 - D, ham.0 + D);
		self.view_center.1 = clamp(self.view_center.1, ham.1 - D, ham.1 + D);
	}

	fn handle_triggers(&mut self) {
		let grid = self.hamster.center() / GRID;
		if self.map.goodie_at(grid) != 0 {
			self.map.set_goodie(grid, 0)
		}
	}

	// ---------------------------------------------------------------------------- events

	pub fn mouse_down(&mut self, _pos: Pt, _left: bool, _right: bool) {}

	pub fn mouse_motion(&mut self, _pos: Pt, _left: bool, _right: bool) {}

	pub fn mouse_wheel(&mut self, _x: i32, _y: i32) {}

	pub fn key_down(&mut self, k: Key) {
		self.key_debouncer.key_down(k);
	}

	pub fn key_up(&mut self, k: Key) {
		self.key_debouncer.key_up(k);
	}

	// ----------------------------------------------------------------------------- stats

	pub fn print_stats(&self) {
		use std::io::Write;
		std::io::stdout().write_all(b"\x1B[2J\x1B[H").unwrap();
		self.hamster.print_stats();
		self.map.print_stats();
	}
}
