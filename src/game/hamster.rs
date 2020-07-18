use crate::prelude::*;

pub struct Hamster {
	pos: Pt,
	/// absolute position of top left corner, pixels
	v_speed_unclamped: i32,
	jump_state: JumpState,
	h_speed: i32,
	look_left: bool,

	textures: [Texture; 2],
}

#[derive(Copy, Clone, Debug)]
enum JumpState {
	Standing,
	JumpingSince(i32),
	Falling,
	Landed,
}

use JumpState::*;

impl Hamster {
	pub fn new(pos: Pt) -> Self {
		Self {
			pos,
			v_speed_unclamped: 0,
			jump_state: Falling,
			h_speed: 0,
			look_left: false,
			textures: [
				Texture::load("hamster3").unwrap(),
				Texture::load("hamster3").unwrap(), // TODO
			],
		}
	}

	pub fn pos(&self) -> Pt {
		self.pos
	}

	pub fn center(&self) -> Pt {
		self.rect().center()
	}

	pub fn speed(&self) -> Pt {
		Pt(self.h_speed, self.vertical_delta())
	}

	// ----------------------------------------------------------------------------- tick

	pub fn tick(&mut self, map: &Map, now: i32, keys: &KeyStates) {
		self.update_jump_state(map, now, keys);
		let dy = self.vertical_delta();

		self.update_walk_state(keys);
		let dx = self.horiz_delta();

		self.update_look_dir(keys);
		self.try_move(map, Pt(dx, dy));
	}

	fn update_look_dir(&mut self, keys: &KeyStates) {
		// look where you're going, or keep look direction.
		match (keys.is_down(Key::Left), keys.is_down(Key::Right)) {
			(false, true) => self.look_left = false,
			(true, false) => self.look_left = true,
			_ => (),
		}
	}

	// ------------------------------------------------------------------------------- walk

	// Maximum horizontal speed (pixels per tick)
	const WALK_PIX_PER_TICK: i32 = (GRID as i32) / 10;
	// Horizontal accelleration (pixels per tick per tick);
	const WALK_ACCEL: i32 = 1;
	// Coast until aligned with this number of pixels.
	// Makes aiming for an empty space between blocks easier
	// and avoids stopping nearly entirely over an edge.
	const WALK_ALIGN: i32 = (GRID as i32) / 2;

	fn update_walk_state(&mut self, keys: &KeyStates) {
		let left = keys.is_down(Key::Left);
		let right = keys.is_down(Key::Right);
		let keydir = match (left, right) {
			(false, false) => 0,
			(true, false) => -1,
			(false, true) => 1,
			(true, true) => 0,
		};

		let currdir = signum(self.h_speed);

		// sudden break
		if currdir * keydir == -1 {
			self.h_speed = 0;
			return;
		}

		// accellerate
		if keydir != 0 {
			self.h_speed = clamp(
				self.h_speed + Self::WALK_ACCEL * keydir,
				-Self::WALK_PIX_PER_TICK,
				Self::WALK_PIX_PER_TICK,
			);
		}

		// coast until aligned
		if keydir == 0 {
			self.h_speed = if self.center().0 % Self::WALK_ALIGN == 0 {
				0
			} else if self.center().0 % 2 == 1 {
				currdir
			} else {
				currdir * 2
			}
		}
	}

	fn horiz_delta(&self) -> i32 {
		self.h_speed
	}

	// -------------------------------------------------------------------------------- jump

	const JUMP_MAX_TICKS: i32 = 20; // should be ~3 blocks in ~500 ms
	const JUMP_MIN_TICKS: i32 = 7; // should be ~1 block
	const JUMP_PIX_PER_TICK: i32 = (3 * GRID as i32) / Self::JUMP_MAX_TICKS;
	const JUMP_V_INIT: i32 = (3 * GRID as i32) / Self::JUMP_MAX_TICKS;
	const JUMP_G: i32 = 2;

	fn update_jump_state(&mut self, map: &Map, now: i32, keys: &KeyStates) {
		let onfeet = self.onfeet(map);
		let jumpy = keys.is_down(Key::A);

		match self.jump_state {
			Standing => {
				if onfeet && jumpy {
					self.v_speed_unclamped = -Self::JUMP_V_INIT;
					self.jump_state = JumpingSince(now);
				}
				if !onfeet {
					self.jump_state = Falling;
				}
			}
			JumpingSince(t) => {
				if now - t > Self::JUMP_MAX_TICKS || (!jumpy && now - t > Self::JUMP_MIN_TICKS) {
					self.jump_state = Falling;
				}
				// bumped into ceiling
				if !self.can_move(map, Pt(0, -1)) {
					self.jump_state = Falling;
					self.v_speed_unclamped = 0;
				}
			}
			Falling => {
				self.v_speed_unclamped += Self::JUMP_G;
				if onfeet {
					self.v_speed_unclamped = 0;
					self.jump_state = Landed;
				}
			}
			Landed => {
				self.v_speed_unclamped = 0;
				if !jumpy {
					self.jump_state = Standing;
				}
				if !onfeet {
					self.jump_state = Falling;
				}
			}
		};
	}

	fn vertical_delta(&self) -> i32 {
		clamp(
			self.v_speed_unclamped,
			-Self::JUMP_PIX_PER_TICK,
			Self::JUMP_PIX_PER_TICK,
		)
	}

	// TODO: simplify
	fn try_move(&mut self, map: &Map, delta: Pt) {
		for _i in 0..abs(delta.0) {
			self.try_move_partial(&map, Pt(signum(delta.0), 0));
		}

		for _i in 0..abs(delta.1) {
			self.try_move_partial(&map, Pt(0, signum(delta.1)));
		}

		assert!(self.can_move(map, Pt(0, 0)));
	}

	fn try_move_partial(&mut self, map: &Map, dir: Pt) {
		if self.can_move(&map, dir) {
			self.pos += dir;
		}
	}

	fn can_move(&self, map: &Map, delta: Pt) -> bool {
		// new bounding box after move.
		let newrect = self.rect().transl(delta);

		// cannot move into a brick
		for vertex in &newrect.vertices_incl() {
			if map.type_at(*vertex / GRID) == BlockTyp::Brick {
				return false;
			}
		}

		// can jump onto a ledge, but not fall trough
		let oldvert = self.rect().vertices_bottom();
		for (i, newvert) in newrect.vertices_bottom().iter().enumerate() {
			let oldy = oldvert[i].1 / (GRID as i32);
			let newy = newvert.1 / (GRID as i32);
			// moving down into a new grid cell that is a ledge.
			if newy > oldy && map.type_at(*newvert / GRID) == BlockTyp::Ledge {
				return false;
			}
		}

		// only wall remains, can move into.
		true
	}

	fn onfeet(&self, map: &Map) -> bool {
		!self.can_move(map, Pt(0, 1))
	}

	fn rect(&self) -> Rect {
		//let margin = 4; // TODO: Rect::shrink(margin)
		Rect::new(self.pos, self.textures[0].dimensions())
	}

	pub fn draw(&self, disp: &mut Viewport, time: i32) {
		let i = if time % 16 > 7 { 1 } else { 0 };
		disp.draw_texture(&self.textures[i], self.pos, self.look_left);
	}

	// ----------------------------------------------------------------------------------- debug

	pub fn print_stats(&self) {
		println!("hamster: pos={}, center={}", self.pos(), self.center());
		println!("         rect={:?}", self.rect());
		println!("         h_speed={}", self.h_speed);
		println!(
			"         jump_state={:?}, v_speed_unclamped={}",
			self.jump_state, self.v_speed_unclamped
		);
	}
}
