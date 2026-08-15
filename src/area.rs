/// A rectangular terminal region measured in cells.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Area {
    /// Horizontal coordinate of the left edge.
    pub x: u16,
    /// Vertical coordinate of the top edge.
    pub y: u16,
    /// Width in terminal cells.
    pub w: u16,
    /// Height in terminal cells.
    pub h: u16,
}

impl Area {
    /// Creates an area at `(x, y)` with the given width and height.
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }
}

impl From<(u16, u16)> for Area {
    fn from((w, h): (u16, u16)) -> Self {
        Area { x: 0, y: 0, w, h }
    }
}
