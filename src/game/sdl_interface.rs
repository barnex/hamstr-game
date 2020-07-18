use crate::editor::prelude::*;
use crate::prelude::*;

use sdl2::event::Event;
use sdl2::mouse;
use sdl2::pixels;
use sdl2::rect;
use sdl2::render::Canvas;
use sdl2::render::TextureCreator;
use sdl2::video::Window;
use sdl2::video::WindowContext;
use std::collections::HashMap;
use std::time;

type SDLTexture = sdl2::render::Texture;

pub fn mainloop(game: &mut Editor) -> Result<()> {
	// (0) initialize sdl window
	let context = sdl2::init()?;
	let window = context
		.video()?
		.window("game", 1920 / 2, 1080 / 2)
		.resizable()
		.position_centered()
		.build()?;
	let canvas = window.into_canvas().accelerated().present_vsync().build()?;
	let texture_creator = canvas.texture_creator();
	let mut event_pump = context.event_pump()?;

	// (1) initialize game logic and display callback (drawback?).
	let mut disp = SDLDisplay::new(canvas, texture_creator);

	// (2) event + render loop
	let mut start = time::Instant::now();
	loop {
		// Advance time. Tick more than once if frames were dropped due to slow rendering.
		// TODO: move timekeeping into game (w/ millisecond resolution time)
		game.tick();
		{
			let el = start.elapsed().as_millis();
			if el > 32 {
				println!("MISSED FRAME after {}ms, catching up", el);
				game.tick();
			}
			if el > 48 {
				println!("Degrading from 30 to 20 FPS");
				game.tick();
			}
			start = time::Instant::now();
		}

		// Event handling
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit { .. } => return Ok(()),
				Event::MouseMotion {
					x, y, mousestate, ..
				} => game.mouse_motion(Pt(x, y), mousestate.left(), mousestate.right()),
				Event::MouseButtonDown {
					x, y, mouse_btn, ..
				} => game.mouse_button(
					Pt(x, y),
					mouse_btn == mouse::MouseButton::Left,
					mouse_btn == mouse::MouseButton::Right,
					true, /*down*/
				),
				Event::MouseButtonUp {
					x, y, mouse_btn, ..
				} => game.mouse_button(
					Pt(x, y),
					mouse_btn == mouse::MouseButton::Left,
					mouse_btn == mouse::MouseButton::Right,
					false, /*up*/
				),
				Event::MouseWheel { x, y, .. } => game.mouse_wheel(x, y),
				Event::KeyDown { keycode, .. } => {
					if let Some(keycode) = keycode {
						game.key_down(keymap(keycode));
					}
				}
				Event::KeyUp { keycode, .. } => {
					if let Some(keycode) = keycode {
						game.key_up(keymap(keycode));
					}
				}
				_ => (),
			}
		}

		game.draw(&mut disp);
		disp.present();
	}
}

/// Display is an abstraction layer over an SDL Canvas and collection of textures,
/// So that none of the game logic needs to be concerned with SDL details.
pub struct SDLDisplay {
	canvas: Canvas<Window>,
	texture_creator: TextureCreator<WindowContext>,
	textures: HashMap<usize, SDLTexture>,
}

impl SDLDisplay {
	pub fn new(mut canvas: Canvas<Window>, texture_creator: TextureCreator<WindowContext>) -> Self {
		canvas.set_blend_mode(sdl2::render::BlendMode::Add);
		SDLDisplay {
			texture_creator,
			canvas,
			textures: HashMap::new(),
		}
	}

	pub fn dimensions(&self) -> (i32, i32) {
		let s = self.canvas.output_size().unwrap();
		(s.0 as i32, s.1 as i32)
	}

	pub fn present(&mut self) {
		self.canvas.present()
	}

	pub fn fill_rect(&mut self, c: BGRA, pos: Pt, (w, h): (i32, i32)) {
		self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
		self.canvas
			.set_draw_color(pixels::Color::RGBA(c.2, c.1, c.0, c.3));
		self.canvas
			.fill_rect(rect::Rect::new(pos.0, pos.1, w as u32, h as u32))
			.unwrap()
	}

	pub fn draw_rect(&mut self, c: BGRA, pos: Pt, (w, h): (i32, i32)) {
		self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
		self.canvas
			.set_draw_color(pixels::Color::RGBA(c.2, c.1, c.0, c.3));
		self.canvas
			.draw_rect(rect::Rect::new(pos.0, pos.1, w as u32, h as u32))
			.unwrap()
	}

	pub fn draw_texture(&mut self, tex: &Texture, pos: Pt, (w, h): (i32, i32), flip: bool) {
		if tex.is_none() {
			return;
		}
		let sdltex = match self.textures.get(&tex.uid()) {
			Some(t) => t,
			None => {
				self.upload_texture(tex);
				&self.textures[&tex.uid()]
			}
		};

		//let (w, h) = tex.dimensions();
		let dst = Some(rect::Rect::new(pos.0, pos.1, w as u32, h as u32));
		self.canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
		self.canvas
			.copy_ex(sdltex, None, dst, 0.0, None, flip, false)
			.unwrap()
	}

	// Copy texture into an SDL texture (on the GPU).
	// Store the handle to the SDL texture under tex.uid().
	fn upload_texture(&mut self, tex: &Texture) {
		let (w, h) = tex.dimensions();
		let pix_bgra = tex.raw_bgra();
		let mut sdltex = self
			.texture_creator
			.create_texture_static(sdl2::pixels::PixelFormatEnum::BGRA32, w as u32, h as u32)
			.unwrap();
		sdltex.set_blend_mode(sdl2::render::BlendMode::Blend);
		sdltex.update(None, &pix_bgra, 4 * w as usize).unwrap();
		self.textures.insert(tex.uid(), sdltex);
	}
}

fn keymap(sdl_key: sdl2::keyboard::Keycode) -> Key {
	use sdl2::keyboard::Keycode;
	match sdl_key {
		Keycode::Left => Key::Left,
		Keycode::S => Key::Left,
		Keycode::J => Key::Left,
		Keycode::Right => Key::Right,
		Keycode::F => Key::Right,
		Keycode::L => Key::Right,
		Keycode::Up => Key::Up,
		Keycode::E => Key::Up,
		Keycode::I => Key::Up,
		Keycode::Down => Key::Down,
		Keycode::D => Key::Down,
		Keycode::K => Key::Down,
		Keycode::Space => Key::A,
		Keycode::LAlt => Key::B,
		Keycode::RAlt => Key::B,
		Keycode::Equals => Key::ZoomIn,
		Keycode::Minus => Key::ZoomOut,
		Keycode::P => Key::Pause,
		Keycode::W => Key::Save,
		Keycode::N => Key::NextMap,
		Keycode::M => Key::PrevMap,
		Keycode::R => Key::Restart,
		_ => Key::None,
	}
}
