/// Logical key codes, after being mapped from physical keys (by fn keymap).
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Key {
	None = 0,
	Left = 1,
	Right = 2,
	Up = 3,
	Down = 4,
	A = 5,
	B = 6,
	X = 7,
	Pause = 8,
	ZoomIn = 9,
	ZoomOut = 10,
	Save = 11,
	PrevMap = 12,
	Restart = 13,
	NextMap = 14,
}

/// KeyStates records which of the lowest 8 Keys are currently pressed down.
/// these are the "dynamic" keys (movement, jumping) that need to be timed precisely
/// and indepdently of the OS key repeat rate.
#[derive(Copy, Clone, Debug)]
pub struct KeyStates {
	down: [bool; 8],
}

impl KeyStates {
	/// Returns if a Key is currently pressed down.
	pub fn is_down(&self, k: Key) -> bool {
		self.down[k as usize]
	}

	fn new() -> KeyStates {
		KeyStates { down: [false; 8] }
	}

	fn set_down(&mut self, k: Key, down: bool) {
		let k = k as usize;
		if k < self.down.len() {
			self.down[k] = down;
		}
	}

	#[must_use]
	fn merge(self, b: KeyStates) -> KeyStates {
		let mut down = [false; 8];
		for i in 0..down.len() {
			down[i] = self.down[i] || b.down[i];
		}
		KeyStates { down }
	}

	fn clear(&mut self) {
		self.down = [false; 8];
	}
}

/// A KeyDebouncer applies hysterisis to keypresses.
/// With debouncing, a very brief keypress (down and up within before a tick happens)
/// is not lost.
pub struct KeyDebouncer {
	current: KeyStates,
	debounced: KeyStates,
}

impl KeyDebouncer {
	pub fn new() -> Self {
		KeyDebouncer {
			current: KeyStates::new(),
			debounced: KeyStates::new(),
		}
	}

	/// key_down must be called to record that a Key was pressed.
	pub fn key_down(&mut self, k: Key) {
		self.current.set_down(k, true);
		self.debounced.set_down(k, true);
	}

	/// key_up must be called to record that a Key was released.
	pub fn key_up(&mut self, k: Key) {
		self.current.set_down(k, false);
	}

	/// key_states returns which keys are currently down,
	/// or have been down before the last call to clear().
	pub fn key_states(&self) -> KeyStates {
		self.current.merge(self.debounced)
	}

	/// Clear must be called after key_states() has been used.
	/// It forgets the keys that were briefly pressed.
	pub fn clear(&mut self) {
		self.debounced.clear();
	}
}
