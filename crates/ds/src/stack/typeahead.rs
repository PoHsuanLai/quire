//! Type-to-select (design/30-CATALOGUE.md section 1.4): letters typed in quick succession make
//! a buffer, and the selection jumps to the next item whose label starts with it. The buffer is
//! forgotten once `TypeaheadReset` has passed since the last letter. A single letter typed again
//! and again steps through the items that start with it.

use ds_style::tokens::delay::DelayToken;
use std::time::Instant;

/// The letters typed so far and when the last one came.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Typeahead {
    buffer: String,
    last: Option<Instant>,
}

impl Typeahead {
    /// Type `text` at `now` with the selection at `from`, among items labelled `labels`: the
    /// next state, and the item to select, if any starts with the buffer. Each label is
    /// compared without regard to case.
    pub fn typed(
        self,
        text: &str,
        now: Instant,
        labels: &[&str],
        from: usize,
    ) -> (Self, Option<usize>) {
        let stale = self.last.is_none_or(|last| {
            now.saturating_duration_since(last) >= DelayToken::TypeaheadReset.delay()
        });
        let mut buffer = if stale { String::new() } else { self.buffer };
        buffer.push_str(&text.to_lowercase());
        let count = labels.len();
        let first = buffer.chars().next();
        let repeated = buffer.chars().count() > 1 && buffer.chars().all(|c| Some(c) == first);
        // One letter, or one letter again and again, moves on from the selection; a longer word
        // narrows from it.
        let (needle, skip) = if repeated {
            (
                buffer.chars().next().map(String::from).unwrap_or_default(),
                1,
            )
        } else if buffer.chars().count() == 1 {
            (buffer.clone(), 1)
        } else {
            (buffer.clone(), 0)
        };
        let found = (0..count)
            .map(|offset| (from + skip + offset) % count.max(1))
            .find(|&index| {
                labels
                    .get(index)
                    .is_some_and(|label| label.to_lowercase().starts_with(&needle))
            });
        (
            Typeahead {
                buffer,
                last: Some(now),
            },
            found,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Typeahead;
    use ds_style::tokens::delay::DelayToken;
    use std::time::{Duration, Instant};

    const LABELS: &[&str] = &["Archive", "Move", "Mute", "Music", "Reply"];

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn a_letter_jumps_to_the_next_label_that_starts_with_it() {
        let start = Instant::now();
        let (_, found) = Typeahead::default().typed("m", start, LABELS, 0);
        assert_eq!(found, Some(1));
        let (_, found) = Typeahead::default().typed("R", start, LABELS, 0);
        assert_eq!(found, Some(4), "case is ignored");
        let (_, found) = Typeahead::default().typed("z", start, LABELS, 0);
        assert_eq!(found, None);
    }

    #[test]
    fn the_same_letter_again_steps_through_its_labels_and_wraps() {
        let start = Instant::now();
        let mut state = Typeahead::default();
        let mut at = 0;
        let mut seen = Vec::new();
        for n in 0..4 {
            let (next, found) = state.typed("m", start + ms(n * 100), LABELS, at);
            state = next;
            at = found.unwrap_or(at);
            seen.push(at);
        }
        assert_eq!(seen, [1, 2, 3, 1]);
    }

    #[test]
    fn letters_in_a_row_narrow_the_word_from_the_selection() {
        let start = Instant::now();
        let (state, found) = Typeahead::default().typed("m", start, LABELS, 0);
        assert_eq!(found, Some(1));
        let (state, found) = state.typed("u", start + ms(200), LABELS, 1);
        assert_eq!(found, Some(2), "mu: Mute, the next from Move");
        let (_, found) = state.typed("s", start + ms(400), LABELS, 2);
        assert_eq!(found, Some(3), "mus: Music");
    }

    #[test]
    fn the_buffer_is_forgotten_after_the_reset_delay() {
        let start = Instant::now();
        let (state, _) = Typeahead::default().typed("m", start, LABELS, 0);
        let late = start + DelayToken::TypeaheadReset.delay();
        let (_, found) = state.typed("r", late, LABELS, 1);
        assert_eq!(found, Some(4), "r alone, not mr");
    }

    #[test]
    fn no_labels_selects_nothing() {
        let (_, found) = Typeahead::default().typed("a", Instant::now(), &[], 0);
        assert_eq!(found, None);
    }
}
