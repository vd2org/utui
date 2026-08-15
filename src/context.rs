use crate::area::Area;
use std::io::Write;

/// The writer, drawing area, and application state available while rendering.
///
/// Built-in widgets are extension traits implemented for this type. Custom widgets can use
/// [`Context::frame`] to write terminal commands or [`Context::with_area`] to render children.
pub struct Context<'a, S, W>
where
    W: Write,
{
    writer: &'a mut W,
    area: Area,
    state: &'a S,
}

impl<'a, S, W> Context<'a, S, W>
where
    W: Write,
{
    /// Creates a rendering context.
    ///
    /// Applications normally receive contexts from [`crate::Tui`]; this constructor is useful
    /// when implementing custom renderers or tests.
    pub fn new(writer: &'a mut W, area: Area, state: &'a S) -> Self {
        Self {
            writer,
            area,
            state,
        }
    }

    /// Returns the area assigned to this context.
    pub fn area(&self) -> &Area {
        &self.area
    }

    /// Returns the immutable application state for the current frame.
    pub fn state(&self) -> &S {
        self.state
    }

    /// Returns direct access to the writer, along with a copy of the area and the state.
    ///
    /// Output written through the returned writer is not clipped to the area.
    pub fn frame(&mut self) -> (&mut W, Area, &S) {
        (self.writer, self.area, self.state)
    }

    /// Renders a child with the same writer and state in another area.
    ///
    /// The supplied area is not checked against the parent area.
    pub fn with_area<R, F>(&mut self, area: Area, draw: F) -> R
    where
        F: for<'b> FnOnce(&mut Context<'b, S, W>) -> R,
    {
        let mut ctx = Context::new(&mut *self.writer, area, self.state);
        draw(&mut ctx)
    }
}
