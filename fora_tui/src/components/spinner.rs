/// Animated braille spinner for loading states.
pub struct Spinner {
    frames: &'static [char],
    tick: usize,
}

const BRAILLE_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: BRAILLE_FRAMES,
            tick: 0,
        }
    }

    /// Advance the spinner by one frame.
    pub fn tick(&mut self) {
        self.tick = (self.tick + 1) % self.frames.len();
    }

    /// Return the current frame character.
    pub fn frame(&self) -> char {
        self.frames[self.tick % self.frames.len()]
    }
}
