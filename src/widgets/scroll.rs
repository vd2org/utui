use crate::area::Area;
use crate::context::Context;
use std::cmp::min;
use std::error::Error;
use std::io::Write;

/// The direction in which increasing item indices are laid out.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ScrollDirection {
    /// Places increasing indices from top to bottom.
    Forward,
    /// Places increasing indices from bottom to top.
    Backward,
}

/// A callback returning `(item_count, selected_index)` from application state.
pub type ScrollLength<S> = dyn Fn(&S) -> Result<(usize, usize), Box<dyn Error>>;
/// A callback that receives an item context, its index, and whether it is selected.
pub type ScrollItem<S, W> =
    dyn for<'a> Fn(&mut Context<'a, S, W>, usize, bool) -> Result<(), Box<dyn Error>>;

/// Scrollable list operations for a rendering context.
pub trait Scroll<S, W>
where
    W: Write,
{
    /// Renders the visible list items, keeping the selected item in view.
    ///
    /// `size` is the height of each item in rows. Only complete items are rendered. The selected
    /// index is clamped to the list and centred when enough items exist on both sides.
    ///
    /// # Errors
    ///
    /// Returns an error if either callback fails.
    fn scroll(
        &mut self,
        size: u16,
        direction: ScrollDirection,
        length: &ScrollLength<S>,
        item: &ScrollItem<S, W>,
    ) -> Result<(), Box<dyn Error>>;
}

impl<'a, S, W> Scroll<S, W> for Context<'a, S, W>
where
    W: Write,
{
    fn scroll(
        &mut self,
        size: u16,
        direction: ScrollDirection,
        length: &ScrollLength<S>,
        item: &ScrollItem<S, W>,
    ) -> Result<(), Box<dyn Error>> {
        if size == 0 {
            return Ok(());
        }

        let area = *self.area();
        let (len, selected) = (length)(self.state())?;
        let visible = min((area.h / size) as usize, len);

        if visible == 0 {
            return Ok(());
        }

        let selected = selected.min(len.saturating_sub(1));
        let start = if len <= visible {
            0
        } else {
            // Keep selected in view, and center it when both sides have enough items.
            selected.saturating_sub(visible / 2).min(len - visible)
        };

        for i in 0..visible {
            let idx = start + i;
            let is_selected = idx == selected;
            let offset = i as u16 * size;
            let y = match direction {
                ScrollDirection::Forward => area.y + offset,
                ScrollDirection::Backward => area.y + area.h - offset - size,
            };
            let item_area = Area::new(area.x, y, area.w, size);
            self.with_area(item_area, |ctx| item(ctx, idx, is_selected))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Scroll, ScrollDirection, ScrollItem, ScrollLength};
    use crate::{Area, Context};
    use std::error::Error;
    use std::io::Write;

    fn render(direction: ScrollDirection) -> Result<String, Box<dyn Error>> {
        let mut output = Vec::new();
        let mut context = Context::new(&mut output, Area::new(0, 10, 8, 6), &());
        let length: &ScrollLength<()> = &|_| Ok((6, 4));
        let item: &ScrollItem<(), Vec<u8>> = &|context, index, selected| {
            let area = *context.area();
            write!(
                context.frame().0,
                "{index}:{selected}:{}:{};",
                area.y,
                area.h
            )?;
            Ok(())
        };

        context.scroll(2, direction, length, item)?;
        Ok(String::from_utf8(output)?)
    }

    #[test]
    fn keeps_the_selected_item_visible_and_centred() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            render(ScrollDirection::Forward)?,
            "3:false:10:2;4:true:12:2;5:false:14:2;"
        );
        Ok(())
    }

    #[test]
    fn backward_scrolling_anchors_items_to_the_bottom() -> Result<(), Box<dyn Error>> {
        assert_eq!(
            render(ScrollDirection::Backward)?,
            "3:false:14:2;4:true:12:2;5:false:10:2;"
        );
        Ok(())
    }

    #[test]
    fn empty_and_zero_sized_scrolls_do_not_render_items() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        let mut context = Context::new(&mut output, Area::new(0, 0, 8, 6), &());
        let empty: &ScrollLength<()> = &|_| Ok((0, 0));
        let populated: &ScrollLength<()> = &|_| Ok((3, 0));
        let item: &ScrollItem<(), Vec<u8>> = &|context, _, _| {
            context.frame().0.write_all(b"item")?;
            Ok(())
        };

        context.scroll(1, ScrollDirection::Forward, empty, item)?;
        context.scroll(0, ScrollDirection::Forward, populated, item)?;

        assert!(output.is_empty());
        Ok(())
    }
}
