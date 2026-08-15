#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum LengthConstraint {
    Fixed(u16),
    Flexible {
        minimum: u16,
        weight: u16,
        maximum: Option<u16>,
    },
}

impl LengthConstraint {
    pub(crate) fn fixed(length: u16) -> Self {
        Self::Fixed(length)
    }

    pub(crate) fn percentage(available: u16, percent: f32) -> Self {
        let length = ((available as f32) * percent.clamp(0.0, 100.0) / 100.0).floor() as u16;
        Self::Fixed(length)
    }

    pub(crate) fn flexible(minimum: u16, weight: u16, maximum: Option<u16>) -> Self {
        if weight == 0 {
            Self::Fixed(minimum)
        } else {
            Self::Flexible {
                minimum,
                weight,
                maximum,
            }
        }
    }
}

struct Entry {
    length: u16,
    weight: u16,
    maximum: Option<u16>,
}

impl Entry {
    fn new(constraint: LengthConstraint) -> Self {
        match constraint {
            LengthConstraint::Fixed(length) => Self {
                length,
                weight: 0,
                maximum: Some(length),
            },
            LengthConstraint::Flexible {
                minimum,
                weight,
                maximum,
            } => Self {
                length: minimum,
                weight,
                maximum,
            },
        }
    }

    fn room(&self) -> u16 {
        if self.weight == 0 {
            return 0;
        }

        self.maximum
            .map_or(u16::MAX, |maximum| maximum.saturating_sub(self.length))
    }

    fn grow_by(&mut self, amount: u16) -> u16 {
        let assigned = amount.min(self.room());
        self.length = self.length.saturating_add(assigned);
        assigned
    }
}

fn remaining_length(available: u16, entries: &[Entry]) -> u16 {
    let used = entries
        .iter()
        .map(|entry| u32::from(entry.length))
        .sum::<u32>()
        .min(u32::from(available)) as u16;

    available.saturating_sub(used)
}

fn flexible_indices(entries: &[Entry]) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| (entry.room() > 0).then_some(index))
        .collect()
}

fn distribute_weighted(entries: &mut [Entry], remaining: u16, pending: &[usize]) -> u16 {
    let total_weight = pending
        .iter()
        .map(|index| u32::from(entries[*index].weight))
        .sum::<u32>();

    if total_weight == 0 {
        return 0;
    }

    pending.iter().copied().fold(0u16, |distributed, index| {
        let share = (u32::from(remaining) * u32::from(entries[index].weight) / total_weight) as u16;
        distributed.saturating_add(entries[index].grow_by(share))
    })
}

fn distribute_remainder(entries: &mut [Entry], remaining: &mut u16, pending: &[usize]) -> bool {
    let mut assigned_any = false;

    for index in pending.iter().copied() {
        if *remaining == 0 {
            break;
        }

        let assigned = entries[index].grow_by(1);
        *remaining -= assigned;
        assigned_any |= assigned > 0;
    }

    assigned_any
}

fn grow_flexible_entries(entries: &mut [Entry], available: u16) {
    let mut remaining = remaining_length(available, entries);

    while remaining > 0 {
        let pending = flexible_indices(entries);
        if pending.is_empty() {
            break;
        }

        let distributed = distribute_weighted(entries, remaining, &pending);
        if distributed > 0 {
            remaining -= distributed;
            continue;
        }

        if !distribute_remainder(entries, &mut remaining, &pending) {
            break;
        }
    }
}

pub(crate) fn resolve_lengths(
    available: u16,
    constraints: impl IntoIterator<Item = LengthConstraint>,
) -> Vec<u16> {
    let mut entries: Vec<_> = constraints.into_iter().map(Entry::new).collect();
    grow_flexible_entries(&mut entries, available);

    let mut remaining = available;
    entries
        .into_iter()
        .map(|entry| {
            let length = entry.length.min(remaining);
            remaining -= length;
            length
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{LengthConstraint, resolve_lengths};

    fn fixed(length: u16) -> LengthConstraint {
        LengthConstraint::fixed(length)
    }

    fn fill(weight: u16) -> LengthConstraint {
        LengthConstraint::flexible(0, weight, None)
    }

    #[test]
    fn preserves_an_entry_for_every_constraint() {
        assert_eq!(
            resolve_lengths(3, [fixed(0), fixed(1), fixed(0), fixed(2)]),
            [0, 1, 0, 2]
        );
        assert_eq!(
            resolve_lengths(1, [fixed(0), fixed(2), fixed(3)]),
            [0, 1, 0]
        );
    }

    #[test]
    fn distributes_fills_by_weight() {
        assert_eq!(resolve_lengths(6, [fill(1), fill(2)]), [2, 4]);
        assert_eq!(resolve_lengths(1, [fill(1), fill(1)]), [1, 0]);
    }

    #[test]
    fn redistributes_space_after_a_maximum_is_reached() {
        let capped = LengthConstraint::flexible(0, 1, Some(2));
        assert_eq!(resolve_lengths(6, [capped, fill(1)]), [2, 4]);
    }

    #[test]
    fn grows_minimum_constraints_after_reserving_the_minimum() {
        let minimum = LengthConstraint::flexible(2, 1, None);
        assert_eq!(resolve_lengths(6, [minimum, fill(1)]), [4, 2]);
    }

    #[test]
    fn clamps_percentages_and_rounds_them_down() {
        assert_eq!(
            resolve_lengths(
                10,
                [
                    LengthConstraint::percentage(10, -1.0),
                    LengthConstraint::percentage(10, 25.0),
                    LengthConstraint::percentage(10, 200.0),
                ],
            ),
            [0, 2, 8]
        );
    }

    #[test]
    fn resolved_lengths_never_exceed_the_available_length() {
        for available in 0..=32 {
            for fixed_length in 0..=32 {
                for maximum in 0..=32 {
                    let resolved = resolve_lengths(
                        available,
                        [
                            fixed(fixed_length),
                            fill(1),
                            LengthConstraint::flexible(0, 1, Some(maximum)),
                        ],
                    );

                    assert_eq!(resolved.len(), 3);
                    assert!(resolved.iter().copied().sum::<u16>() <= available);
                    assert!(resolved[2] <= maximum);
                }
            }
        }
    }

    #[test]
    fn an_uncapped_flexible_entry_uses_all_remaining_space() {
        for available in 0..=64 {
            for reserved in 0..=available {
                let resolved = resolve_lengths(available, [fixed(reserved), fill(1)]);
                assert_eq!(resolved, [reserved, available - reserved]);
            }
        }
    }
}
