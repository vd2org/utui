use crate::context::Context;
use cli_truncate::truncate;
use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::Print;
use std::error::Error;
use std::io::Write;

/// A callback that derives one rendered line from application state.
///
/// Returned ANSI formatting is passed through to the terminal.
pub type TextLine<S> = dyn for<'a> Fn(&'a S) -> Result<String, Box<dyn Error>>;

/// Single-line text operations for a rendering context.
pub trait Text<S, W>
where
    W: Write,
{
    /// Renders text at the context area's origin.
    ///
    /// Newlines, carriage returns, and tabs are replaced with spaces. Text is truncated to the
    /// area's display width with an ellipsis, accounting for wide characters and ANSI sequences.
    ///
    /// # Errors
    ///
    /// Returns an error if the extractor or terminal output fails.
    fn line(&mut self, item: &TextLine<S>) -> Result<(), Box<dyn Error>>;
}

impl<'a, S, W> Text<S, W> for Context<'a, S, W>
where
    W: Write,
{
    fn line(&mut self, extractor: &TextLine<S>) -> Result<(), Box<dyn Error>> {
        let (writer, area, state) = self.frame();
        let text = (extractor)(state)?;

        let max_width = area.w as usize;
        let sanitized = text.replace("\r\n", " ").replace(['\n', '\r', '\t'], " ");
        let display_text = truncate(&sanitized, max_width);

        queue!(writer, MoveTo(area.x, area.y))?;
        queue!(writer, Print(display_text))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Text, TextLine};
    use crate::{Area, Context};
    use std::error::Error;

    fn render(text: String, width: u16) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut output = Vec::new();
        let mut context = Context::new(&mut output, Area::new(0, 0, width, 1), &());
        let extractor: &TextLine<()> = &move |_| Ok(text.clone());
        context.line(extractor)?;
        Ok(output)
    }

    #[test]
    fn truncates_by_terminal_display_width() -> Result<(), Box<dyn Error>> {
        let output = render("古池や蛙".to_owned(), 6)?;
        assert!(output.ends_with("古池…".as_bytes()));
        Ok(())
    }

    #[test]
    fn preserves_ansi_formatting_from_extractors() -> Result<(), Box<dyn Error>> {
        let styled = "\x1b[31mred\x1b[0m";
        let output = render(styled.to_owned(), 3)?;
        assert!(output.ends_with(styled.as_bytes()));
        Ok(())
    }

    #[test]
    fn normalises_multiline_text_to_one_line() -> Result<(), Box<dyn Error>> {
        let output = render("one\r\ntwo\nthree\tfour".to_owned(), 32)?;
        assert!(output.ends_with(b"one two three four"));
        Ok(())
    }
}
