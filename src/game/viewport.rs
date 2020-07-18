use crate::prelude::*;

pub struct Viewport<'a> {
	disp: &'a mut SDLDisplay,
	origin: Pt,
	zoom: i32,
}

impl<'a> Viewport<'a> {
	pub fn with_center(disp: &'a mut SDLDisplay, center: Pt) -> Self {
		let (w, h) = disp.dimensions();
		let origin = center - Pt(w / 2, h / 2);
		Self::with_origin(disp, origin)
	}

	pub fn with_origin(disp: &'a mut SDLDisplay, origin: Pt) -> Self {
		Self::with_zoom(disp, origin, 1)
	}

	pub fn with_zoom(disp: &'a mut SDLDisplay, origin: Pt, zoom: i32) -> Self {
		Self { disp, origin, zoom }
	}

	pub fn draw_texture(&mut self, tex: &Texture, pos: Pt, flip: bool) {
		let pos = self.to_screen(pos);
		let (w, h) = self.scale_dim(tex.dimensions());
		self.disp.draw_texture(tex, pos, (w, h), flip)
	}

	pub fn fill_rect(&mut self, c: BGRA, pos: Pt, dim: (i32, i32)) {
		let pos = self.to_screen(pos);
		let dim = self.scale_dim(dim);
		self.disp.fill_rect(c, pos, dim);
	}

	pub fn draw_rect(&mut self, c: BGRA, pos: Pt, dim: (i32, i32)) {
		let pos = self.to_screen(pos);
		let dim = self.scale_dim(dim);
		self.disp.draw_rect(c, pos, dim);
	}

	// TODO: remove
	pub fn clear(&mut self, color: BGRA) {
		self.disp.fill_rect(color, Pt(0, 0), self.disp.dimensions());
	}

	fn to_screen(&self, rel: Pt) -> Pt {
		(rel - self.origin) / self.zoom
	}

	fn scale_dim(&self, dim: (i32, i32)) -> (i32, i32) {
		(dim.0 / self.zoom, dim.1 / self.zoom)
	}

	pub fn visible_blocks(&self) -> ((i32, i32), (i32, i32)) {
		let dim = self.disp.dimensions();
		let dim = Pt(dim.0, dim.1) * self.zoom;
		let ptmin = self.origin;
		let ptmax = ptmin + dim;
		let grmin: Pt = ptmin / GRID - (1, 1);
		let grmax: Pt = ptmax / GRID + (1, 1);
		(grmin.as_tuple(), grmax.as_tuple())
	}
}
