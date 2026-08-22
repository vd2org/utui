use crate::context::Context;
use crate::internal::layout::{LengthConstraint, resolve_lengths};
use std::error::Error;
use std::io::Write;

/// A size constraint for a stack item along the chosen layout axis.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Constraint {
    /// Requests an exact number of cells.
    Exact(u16),
    /// Requests a percentage of the available length, clamped to `0..=100` and rounded down.
    Percentage(f32),
    /// Receives remaining cells in proportion to the supplied weight.
    Fill(u16),
    /// Reserves a minimum number of cells, then shares remaining cells with weight `1`.
    Min(u16),
    /// Shares remaining cells with weight `1`, up to the supplied maximum.
    Max(u16),
}

/// A callback that renders one stack item.
pub type StackItem<S, W> = dyn for<'a> Fn(&mut Context<'a, S, W>) -> Result<(), Box<dyn Error>>;

/// Constraint-based layout operations for a rendering context.
pub trait Stack<S, W>
where
    W: Write,
{
    /// Resolves item heights and renders non-empty items from top to bottom.
    ///
    /// Fixed and minimum sizes are reserved first. Remaining rows are distributed among flexible
    /// items. If the requested sizes exceed the available height, later items are clipped or
    /// omitted.
    ///
    /// # Errors
    ///
    /// Returns the first error produced by an item callback.
    fn vertical(&mut self, items: &[(Constraint, &StackItem<S, W>)]) -> Result<(), Box<dyn Error>>;

    /// Resolves item widths and renders non-empty items from left to right.
    ///
    /// Fixed and minimum sizes are reserved first. Remaining columns are distributed among
    /// flexible items. If the requested sizes exceed the available width, later items are clipped
    /// or omitted.
    ///
    /// # Errors
    ///
    /// Returns the first error produced by an item callback.
    fn horizontal(
        &mut self,
        items: &[(Constraint, &StackItem<S, W>)],
    ) -> Result<(), Box<dyn Error>>;
}

impl<'a, S, W> Stack<S, W> for Context<'a, S, W>
where
    W: Write,
{
    fn vertical(&mut self, items: &[(Constraint, &StackItem<S, W>)]) -> Result<(), Box<dyn Error>> {
        let area = *self.area();

        if area.h == 0 || items.is_empty() {
            return Ok(());
        }

        let heights = resolve_lengths(
            area.h,
            items
                .iter()
                .map(|(constraint, _)| to_length_constraint(*constraint, area.h)),
        );
        let mut y = area.y;

        for ((_, item), height) in items.iter().zip(heights) {
            if height == 0 {
                continue;
            }

            let item_area = crate::Area::new(area.x, y, area.w, height);
            self.with_area(item_area, |ctx| item(ctx))?;
            y = y.saturating_add(height);
        }

        Ok(())
    }

    fn horizontal(
        &mut self,
        items: &[(Constraint, &StackItem<S, W>)],
    ) -> Result<(), Box<dyn Error>> {
        let area = *self.area();

        if area.w == 0 || items.is_empty() {
            return Ok(());
        }

        let widths = resolve_lengths(
            area.w,
            items
                .iter()
                .map(|(constraint, _)| to_length_constraint(*constraint, area.w)),
        );
        let mut x = area.x;

        for ((_, item), width) in items.iter().zip(widths) {
            if width == 0 {
                continue;
            }

            let item_area = crate::Area::new(x, area.y, width, area.h);
            self.with_area(item_area, |ctx| item(ctx))?;
            x = x.saturating_add(width);
        }

        Ok(())
    }
}

fn to_length_constraint(constraint: Constraint, available: u16) -> LengthConstraint {
    match constraint {
        Constraint::Exact(length) => LengthConstraint::fixed(length),
        Constraint::Percentage(percent) => LengthConstraint::percentage(available, percent),
        Constraint::Fill(weight) => LengthConstraint::flexible(0, weight, None),
        Constraint::Min(minimum) => LengthConstraint::flexible(minimum, 1, None),
        Constraint::Max(maximum) => LengthConstraint::flexible(0, 1, Some(maximum)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Constraint, Stack, StackItem};
    use crate::{Area, Context};
    use std::error::Error;
    use std::io::Write;

    #[test]
    fn vertical_lays_items_out_from_top_to_bottom() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        let mut context = Context::new(&mut output, Area::new(3, 5, 10, 8), &());
        let item: &StackItem<(), Vec<u8>> = &|context| {
            let area = *context.area();
            write!(
                context.frame().0,
                "{}:{}:{}:{};",
                area.x,
                area.y,
                area.w,
                area.h
            )?;
            Ok(())
        };

        context.vertical(&[(Constraint::Exact(2), item), (Constraint::Fill(1), item)])?;

        assert_eq!(output, b"3:5:10:2;3:7:10:6;");
        Ok(())
    }

    #[test]
    fn horizontal_lays_items_out_from_left_to_right() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        let mut context = Context::new(&mut output, Area::new(3, 5, 10, 8), &());
        let item: &StackItem<(), Vec<u8>> = &|context| {
            let area = *context.area();
            write!(
                context.frame().0,
                "{}:{}:{}:{};",
                area.x,
                area.y,
                area.w,
                area.h
            )?;
            Ok(())
        };

        context.horizontal(&[(Constraint::Exact(3), item), (Constraint::Fill(1), item)])?;

        assert_eq!(output, b"3:5:3:8;6:5:7:8;");
        Ok(())
    }

    #[test]
    fn zero_sized_entries_do_not_shift_callbacks() -> Result<(), Box<dyn Error>> {
        let mut output = Vec::new();
        let mut context = Context::new(&mut output, Area::new(0, 0, 1, 1), &());
        let first: &StackItem<(), Vec<u8>> = &|context| {
            context.frame().0.write_all(b"first")?;
            Ok(())
        };
        let second: &StackItem<(), Vec<u8>> = &|context| {
            context.frame().0.write_all(b"second")?;
            Ok(())
        };
        let items = [
            (Constraint::Exact(0), first),
            (Constraint::Exact(1), second),
        ];

        context.vertical(&items)?;
        context.horizontal(&items)?;

        assert_eq!(output, b"secondsecond");
        Ok(())
    }
}
