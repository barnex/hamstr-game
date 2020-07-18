use super::prelude::*;
use crate::prelude::*;

pub struct Editor {
	// left-hand side toolbars
	brush_bar: Toolbar,
	palette_bar: Toolbar,

	// map drawing area
	path: PathBuf,
	map: Map,
	view_origin: Pt,
	view_zoom: i32,
	selection_start: Pt,
	selection_end: Pt,

	// None in edit mode, Some game in play mode
	game: Option<GameState>,
}

impl Editor {
	/// Editor with empty map.
	pub fn new() -> Self {
		Self::from_data(LevelData::new(), PathBuf::from("level.json"))
	}

	/// Editor with map loaded from a JSON file.
	pub fn load(p: PathBuf) -> Result<Editor> {
		let data = LevelData::load(&p)?;
		Ok(Self::from_data(data, p))
	}

	fn from_data(data: LevelData, path: PathBuf) -> Self {
		let goodies = data.goodies_map();
		let (map, lights) = (data.map_bytes, data.lights);
		Self {
			path,
			view_origin: Pt(0, 0),
			view_zoom: 1,
			selection_start: Pt(0, 0),
			selection_end: Pt(0, 0),
			brush_bar: Toolbar::new(
				Pt(5, 2),
				Texture::load_many(&["pencil", "pencil_rect"]).unwrap(),
			),
			palette_bar: Toolbar::new(
				Pt(5, GRID as i32 + 2 + 5),
				Self::init_palette(default_palette(), &ED_PALETTE),
			),
			map: Map::from(map, goodies, lights),
			game: None,
		}
	}

	fn save(&self) -> Result<()> {
		LevelData::save(
			&self.path,
			&self.map.bytemap(),
			&self.map.goodies(),
			&self.map.lights(),
		)
	}

	pub const BG: BGRA = BGRA(255, 255, 255, 255);

	// ray trace surface palette into textures,
	// with very simple lighting.
	// textures used as toolbar buttons and to draw the map.
	fn init_palette(palette: Vec<Surface>, block_defs: &[BlockDef]) -> Vec<Texture> {
		let boring_lights = Lights {
			sun_dir: Vec3(1.0, -1.0, 3.0).normalized(),
			sun_intens: RGBf(0.5, 0.5, 0.6),
			sun_angle: 0.0,
			sun_rays: 1,
			ambient: RGBf(0.0, 0.0, 0.0),
			ambient_rays: 0,
			fake_ambient: RGBf(0.5, 0.5, 0.6),
			invert_dm: false,
		};

		let n = palette.len();
		let renderer = SharedData::new(Arc::new(palette), boring_lights);
		let mut texs = Vec::<Texture>::with_capacity(n);
		for i in 0..n {
			let uid = block_defs[i].uid;
			//println!("editor: rendering block {}", uid);
			let tex = Texture::new(renderer.render_central_block(TileKey::with_center(uid)));
			texs.push(tex);
		}

		texs
	}

	// ------------------------------------------------------------------------------ draw

	pub fn draw(&self, disp: &mut SDLDisplay) {
		if !self.is_paused() {
			self.game.as_ref().unwrap().draw(disp);
			return;
		}

		let mut view = Viewport::with_zoom(disp, self.view_origin, self.view_zoom);
		view.clear(Self::BG);

		let grid = GRID as i32;
		let ((xmin, ymin), (xmax, ymax)) = view.visible_blocks();
		for iy in ymin..ymax {
			for ix in xmin..xmax {
				let pos = Pt(ix * grid, iy * grid);
				let tex = self.map.texture_at(Pt(ix, iy));
				view.draw_texture(&tex, pos, false);
				view.draw_rect(Self::GRID_COLOR, pos, (GRID as i32 + 1, GRID as i32 + 1));
			}
		}

		self.draw_selection(&mut view);
		self.brush_bar.draw(disp);
		self.palette_bar.draw(disp);
	}

	fn draw_selection(&self, view: &mut Viewport) {
		let grid = GRID as i32;
		let min = self.selection().min;
		let (w, h) = self.selection().dimensions();
		view.fill_rect(Self::SELECTION_BG, min * grid, (w * grid, h * grid));
		view.draw_rect(Self::SELECTION_FG, min * grid, (w * grid, h * grid));
	}

	const GRID_COLOR: BGRA = BGRA(196, 128, 128, 32);
	const SELECTION_BG: BGRA = BGRA(255, 128, 128, 64);
	const SELECTION_FG: BGRA = BGRA(255, 128, 128, 224);

	// ------------------------------------------------------------------------------- tick

	pub fn tick(&mut self) {
		if !self.is_paused() {
			self.game.as_mut().unwrap().tick();
		}
	}

	// ------------------------------------------------------------------------------- events

	pub fn mouse_button(&mut self, pos: Pt, left: bool, right: bool, down: bool) {
		// dispatch to relevant toolbar...
		for bar in &mut [&mut self.brush_bar, &mut self.palette_bar] {
			if bar.is_inside(pos) {
				if down {
					bar.button_click(pos);
				}
				return;
			}
		}
		// ...or drawing area
		match self.brush_bar.selected() {
			0 => self.mouse_button_pencil(pos, left, right, down),
			1 => self.mouse_button_pencil_rect(pos, left, right, down),
			_ => panic!("unhandled brush button"),
		}
	}

	pub fn mouse_motion(&mut self, pos: Pt, left: bool, right: bool) {
		// dispatch to relevant toolbar...
		for bar in &[&self.brush_bar, &self.palette_bar] {
			if bar.is_inside(pos) {
				return;
			}
		}
		// ...or drawing area
		match self.brush_bar.selected() {
			0 => self.mouse_motion_pencil(pos, left, right),
			1 => self.mouse_motion_pencil_rect(pos, left, right),
			_ => panic!("unhandled brush button"),
		}
	}

	// ---------------------------------------------------------------------------- pencil mode

	// mouse event in drawing area, while in "pencil" mode: draw single block.
	fn mouse_button_pencil(&mut self, pos: Pt, left: bool, right: bool, down: bool) {
		if down {
			self.mouse_motion_pencil(pos, left, right)
		}
	}

	// mouse event in drawing area, while in "pencil" mode: draw single block.
	fn mouse_motion_pencil(&mut self, pos: Pt, left: bool, right: bool) {
		if let Some(pos) = self.pix_to_grid(pos) {
			if left {
				self.set(pos, self.selected_block());
			}
			if right {
				if self.map.goodie_at(pos) != 0 {
					self.map.set_goodie(pos, 0);
				} else {
					self.set(pos, 0);
				}
			}
		}
	}

	// screen pixel position to grid index, if valid.
	fn pix_to_grid(&self, pos: Pt) -> Option<Pt> {
		let pos: Pt = self.view_origin + pos * self.view_zoom;
		let grid = pos / GRID;
		if grid.0 < 1 || grid.1 < 1 {
			None
		} else {
			Some(grid)
		}
	}

	// ------------------------------------------------------------------------------ rectangle mode

	// mouse event in drawing area, while in "pencil_rect" mode: fill a rectangle
	fn mouse_button_pencil_rect(&mut self, pos: Pt, left: bool, right: bool, down: bool) {
		if down && left {
			self.mouse_down_pencil_rect(pos)
		}
		if !down && left {
			self.mouse_up_pencil_rect(pos)
		}
		// right tap: delete, as in pencil mode
		// bit of a hack to easily remove goodies
		if down && right {
			self.mouse_button_pencil(pos, left, right, down)
		}
	}

	fn mouse_down_pencil_rect(&mut self, pos: Pt) {
		if let Some(grid) = self.pix_to_grid(pos) {
			self.selection_start = grid;
			self.selection_end = grid;
		}
	}
	fn mouse_up_pencil_rect(&mut self, pos: Pt) {
		if let Some(grid) = self.pix_to_grid(pos) {
			self.selection_end = grid;
		}
		self.fill_rect(self.selection(), self.selected_block());
		self.clear_selection();
	}

	fn fill_rect(&mut self, rect: Rect, blk: u8) {
		let Pt(xmin, ymin) = rect.min;
		let Pt(xmax, ymax) = rect.max;
		for iy in ymin..ymax {
			for ix in xmin..xmax {
				self.set(Pt(ix, iy), blk);
			}
		}
	}

	fn set(&mut self, pos: Pt, blk: u8) {
		self.map.set(pos, blk);
	}

	fn clear_selection(&mut self) {
		self.selection_start = Pt(0, 0);
		self.selection_end = Pt(0, 0);
	}

	// mouse event in drawing area, while in "pencil_rect" mode: fill a rectangle
	fn mouse_motion_pencil_rect(&mut self, pos: Pt, left: bool, _right: bool) {
		if left {
			if let Some(grid) = self.pix_to_grid(pos) {
				self.selection_end = grid;
			}
		}
	}

	//
	fn selection(&self) -> Rect {
		if (self.selection_start, self.selection_end) == (Pt(0, 0), Pt(0, 0)) {
			return Rect::new(Pt(0, 0), (0, 0)); // empty
		}
		let xmin = min(self.selection_start.0, self.selection_end.0);
		let ymin = min(self.selection_start.1, self.selection_end.1);
		let xmax = max(self.selection_start.0, self.selection_end.0);
		let ymax = max(self.selection_start.1, self.selection_end.1);
		Rect {
			min: Pt(xmin, ymin),
			max: Pt(xmax, ymax) + Pt(1, 1),
		}
	}

	// -------------------------------------------------------------------------------

	fn selected_block(&self) -> u8 {
		ED_PALETTE[self.palette_bar.selected()].uid
	}

	pub fn mouse_wheel(&mut self, x: i32, y: i32) {
		if !self.is_paused() {
			return;
		}
		self.pan_view(Pt(-x, -y));
	}

	pub fn key_down(&mut self, k: Key) {
		match self.is_paused() {
			true => self.key_down_editing(k),
			false => self.key_down_playing(k),
		}
	}

	fn key_down_editing(&mut self, k: Key) {
		match k {
			Key::Left => self.pan_view(Pt(-1, 0)),
			Key::Right => self.pan_view(Pt(1, 0)),
			Key::Up => self.pan_view(Pt(0, -1)),
			Key::Down => self.pan_view(Pt(0, 1)),
			Key::Pause => self.toggle_pause(),
			Key::ZoomIn => self.zoom_in(),
			Key::ZoomOut => self.zoom_out(),
			_ => (),
		}
	}

	fn key_down_playing(&mut self, k: Key) {
		if k == Key::Pause {
			self.toggle_pause();
		} else {
			self.game.as_mut().unwrap().key_down(k);
		}
	}

	pub fn key_up(&mut self, k: Key) {
		if !self.is_paused() {
			self.game.as_mut().unwrap().key_up(k);
			return;
		}
	}

	fn zoom_in(&mut self) {
		self.view_zoom = max(1, self.view_zoom / 2);
	}

	fn zoom_out(&mut self) {
		self.view_zoom = min(4, self.view_zoom * 2);
	}

	fn toggle_pause(&mut self) {
		if self.is_paused() {
			self.save().expect("saving level");
		}
		self.game = match self.game {
			None => Some(GameState::new(self.map.clone())), // TODO: translate map
			Some(_) => None,
		}
	}

	/// Is the editor in "paused" (i.e. "editing") mode?
	/// Not paused means we're playing the game.
	fn is_paused(&self) -> bool {
		match self.game {
			None => true,
			Some(_) => false,
		}
	}

	/// In editing mode, move the viewport by a number of grid steps.
	fn pan_view(&mut self, delta: Pt) {
		self.view_origin += delta * GRID;
	}

	// ------------------------------------------------------------------------------ stats
	//fn print_stats(&self) {
	//	use std::io::Write;
	//	std::io::stdout().write_all(b"\x1B[2J\x1B[H").unwrap();
	//	println!("view_center: {}", self.view_center);
	//}
}
