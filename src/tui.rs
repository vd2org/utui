use crate::area::Area;
use crate::context::Context;
use crossterm::SynchronizedUpdate;
use crossterm::queue;
use crossterm::terminal::{Clear, ClearType};
use std::error::Error;
use std::io::Write;

/// A root rendering callback.
pub type Root<S, W> = dyn for<'a> Fn(&mut Context<'a, S, W>) -> Result<(), Box<dyn Error>>;

/// An immediate-mode terminal renderer driven by a root callback.
///
/// `Tui` renders frames but does not manage raw mode, alternate screens, input events, or terminal
/// restoration. Applications retain control of that lifecycle.
pub struct Tui<'a, S, W>
where
    W: Write,
{
    root: &'a Root<S, W>,
}

impl<'a, S, W> Tui<'a, S, W>
where
    W: Write,
{
    /// Creates a renderer using `root` as its view.
    pub fn new(root: &'a Root<S, W>) -> Self {
        Self { root }
    }

    /// Clears the terminal and renders one synchronized frame.
    ///
    /// The root callback receives an immutable `state` and the requested `area`. Output is flushed
    /// when the synchronized update ends.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal output or the root callback fails.
    pub fn draw(&self, writer: &mut W, area: &Area, state: &S) -> Result<(), Box<dyn Error>> {
        writer.sync_update(|writer| {
            queue!(writer, Clear(ClearType::All))?;
            let mut ctx = Context::new(writer, *area, state);
            (self.root)(&mut ctx)
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::Tui;
    use crate::Area;
    use std::error::Error;
    use std::io::Write;

    #[test]
    fn clears_the_terminal_before_rendering_the_frame() -> Result<(), Box<dyn Error>> {
        let root = |context: &mut crate::Context<'_, (), Vec<u8>>| {
            context.frame().0.write_all(b"frame")?;
            Ok(())
        };
        let tui = Tui::new(&root);
        let mut output = Vec::new();

        let area = Area::new(0, 0, 1, 1);
        tui.draw(&mut output, &area, &())?;
        tui.draw(&mut output, &area, &())?;

        let positions = |needle: &[u8]| {
            output
                .windows(needle.len())
                .enumerate()
                .filter_map(|(index, window)| (window == needle).then_some(index))
                .collect::<Vec<_>>()
        };
        let clears = positions(b"\x1b[2J");
        let frames = positions(b"frame");
        assert_eq!(clears.len(), 2);
        assert_eq!(frames.len(), 2);
        assert!(
            clears
                .into_iter()
                .zip(frames)
                .all(|(clear, frame)| clear < frame)
        );
        Ok(())
    }
}
