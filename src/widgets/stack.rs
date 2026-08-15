use crate::context::Context;
use crate::internal::layout::{LengthConstraint, resolve_lengths};
use std::error::Error;
use std::io::Write;

/// A vertical size constraint for a stack item.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Constraint {
    /// Requests an exact number of rows.
    Exact(u16),
    /// Requests a percentage of the stack height, clamped to `0..=100` and rounded down.
    Percentage(f32),
    /// Receives remaining rows in proportion to the supplied weight.
    Fill(u16),
    /// Reserves a minimum number of rows, then shares remaining rows with weight `1`.
    Min(u16),
    /// Shares remaining rows with weight `1`, up to the supplied maximum.
    Max(u16),
}

/// A callback that renders one stack item.
pub type StackItem<S, W> = dyn for<'a> Fn(&mut Context<'a, S, W>) -> Result<(), Box<dyn Error>>;

/// Constraint-based vertical layout operations for a rendering context.
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
    fn stack(&mut self, items: &[(Constraint, &StackItem<S, W>)]) -> Result<(), Box<dyn Error>>;
}

impl<'a, S, W> Stack<S, W> for Context<'a, S, W>
where
    W: Write,
{
    fn stack(&mut self, items: &[(Constraint, &StackItem<S, W>)]) -> Result<(), Box<dyn Error>> {
        let area = *self.area();

        if area.h == 0 || items.is_empty() {
            return Ok(());
        }

        let lengths = resolve_lengths(
            area.h,
            items
                .iter()
                .map(|(constraint, _)| to_length_constraint(*constraint, area.h)),
        );
        let mut y = area.y;

        for ((_, item), height) in items.iter().zip(lengths) {
            if height == 0 {
                continue;
            }

            let item_area = crate::Area::new(area.x, y, area.w, height);
            self.with_area(item_area, |ctx| item(ctx))?;
            y = y.saturating_add(height);
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
    fn zero_height_entries_do_not_shift_callbacks() -> Result<(), Box<dyn Error>> {
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

        context.stack(&[
            (Constraint::Exact(0), first),
            (Constraint::Exact(1), second),
        ])?;

        assert_eq!(output, b"second");
        Ok(())
    }
}
