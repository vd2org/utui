use crate::START_TIME;
use crate::context::Context;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use std::error::Error;
use std::io::Write;

/// A sequence of spinner frames and the time between them in milliseconds.
pub struct SimpleSpinnerData<'a> {
    chars: &'a [&'a str],
    interval: u64,
}

impl<'a> SimpleSpinnerData<'a> {
    /// Creates spinner data from a sequence of frames and a frame interval.
    pub const fn new(chars: &'a [&'a str], interval: u64) -> Self {
        Self { chars, interval }
    }

    /// Returns the frame for the current time.
    pub fn frame(&self) -> &'a str {
        let index = (START_TIME.elapsed().as_millis() / self.interval as u128
            % self.chars.len() as u128) as usize;
        self.chars[index]
    }
}

/// A braille-dot spinner preset.
pub static SPINNER_DOTS: SimpleSpinnerData =
    SimpleSpinnerData::new(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"], 100);

/// A rotating-line spinner preset.
pub static SPINNER_LINES: SimpleSpinnerData = SimpleSpinnerData::new(&["|", "/", "-", "\\"], 100);

/// A box spinner preset.
pub static SPINNER_BLOCKS: SimpleSpinnerData =
    SimpleSpinnerData::new(&["▘", "▀", "▝", "▐", "▗", "▄", "▖", "▌"], 100);

/// Single-line text operations for a rendering context.
pub trait Spinner<S, W>
where
    W: Write,
{
    /// Renders the spinner's current frame at the context area's origin.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal output fails.
    fn simple_spinner(&mut self, kind: &SimpleSpinnerData) -> Result<(), Box<dyn Error>>;
}

impl<'a, S, W> Spinner<S, W> for Context<'a, S, W>
where
    W: Write,
{
    fn simple_spinner(&mut self, kind: &SimpleSpinnerData) -> Result<(), Box<dyn Error>> {
        let (writer, area, _) = self.frame();

        queue!(writer, MoveTo(area.x, area.y))?;
        queue!(writer, Print(kind.frame()))?;

        Ok(())
    }
}
