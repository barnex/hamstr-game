use super::prelude::*;
use crate::prelude::*;

/// The Toolbar allows the user to pick bloks in edit mode.
pub struct Toolbar {
	screen_pos: Pt,
	pub columns: i32,
	buttons: Vec<Texture>,
	tex_selected: Texture,
	selected: usize,
}

impl Toolbar {
	pub fn new(screen_pos: Pt, palette: Vec<Texture>) -> Self {
		Self {
			screen_pos,
			columns: 4,
			buttons: palette,
			tex_selected: Texture::load("selected").unwrap(), // TODO: could be deduped
			selected: 0,
		}
	}

	pub fn dimensions(&self) -> (i32, i32) {
		let w = self.columns * (GRID as i32);
		let h = ((self.buttons.len() as i32 - 1) / self.columns + 1) * (GRID as i32);
		(w, h)
	}

	pub fn button_click(&mut self, pos: Pt) {
		// position in internal grid
		let pos = (pos - self.screen_pos) / (GRID as i32);
		if pos.is_neg() {
			return;
		}

		let button = pos.x() as usize + pos.y() as usize * self.columns as usize;
		if button < self.buttons.len() {
			self.selected = button;
		}
	}

	// tests wheter a mouse position is inside this Pane.
	pub fn is_inside(&self, pos: Pt) -> bool {
		Rect::new(self.screen_pos, self.dimensions()).is_inside(pos)
	}

	pub fn selected(&self) -> usize {
		self.selected
	}

	pub fn draw(&self, disp: &mut SDLDisplay) {
		let mut disp = Viewport::with_origin(disp, -self.screen_pos);

		// background with 1-pixel margin
		let (w, h) = self.dimensions();
		disp.fill_rect(BGRA(128, 128, 128, 255), Pt(-1, -1), (w + 2, h + 2));
		// buttons
		for (i, b) in self.buttons.iter().enumerate() {
			// Clear background first
			// only needed for non-opaque sprites.
			let pos = self.button_pos(i);
			disp.fill_rect(Editor::BG, pos, (GRID as i32, GRID as i32));
			disp.draw_texture(b, pos, false);
		}
		disp.draw_texture(&self.tex_selected, self.button_pos(self.selected), false);
	}

	// relative position within the toolaber, in pixels, of the i'th button.
	pub fn button_pos(&self, i: usize) -> Pt {
		let i = i as i32;
		Pt(i % self.columns as i32, i / self.columns as i32) * (GRID as i32)
	}
}
