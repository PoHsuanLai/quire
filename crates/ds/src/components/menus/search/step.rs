//! What a key does to a search field's suggestions: pure, `(state, key) -> (state, act)`. The
//! keyboard never leaves the field, so these are the field's keys: Up and Down move a cursor over
//! the choices, Enter picks the one under it, Escape closes the panel first and clears the field
//! second. Tab is not here: it behaves as it always does.

use crate::stack::roving::{Step, Wrap, moved_live};
use ds_core::vocab::{Availability, Shown};

/// Whether the suggestions are showing, and which choice the cursor is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Suggesting {
    /// Whether the panel is wanted: it also needs choices to show.
    pub shown: Shown,
    /// The highlighted choice, numbered as a menu numbers its choices; none until a key or the
    /// pointer asks for one, so Enter submits the text.
    pub cursor: Option<usize>,
}

/// A key the field hears while it has suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SuggestKey {
    Up,
    Down,
    Enter,
    Escape,
}

/// Whether the field holds any text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Text {
    Empty,
    Filled,
}

/// What the field does besides moving the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Act {
    /// The key is the field's own: caret movement, a submit, or Escape reaching the window.
    Pass,
    /// The key is taken here; nothing more happens.
    Swallow,
    /// The choice under the cursor was picked.
    Pick(usize),
    /// The text is cleared.
    Clear,
    /// The text is submitted.
    Submit,
}

impl Suggesting {
    /// The panel is closed and nothing is highlighted.
    pub(crate) fn closed() -> Self {
        Suggesting {
            shown: Shown::Hidden,
            cursor: None,
        }
    }

    /// The panel is wanted with nothing highlighted: the caret arrived, or the text changed.
    pub(crate) fn open() -> Self {
        Suggesting {
            shown: Shown::Visible,
            cursor: None,
        }
    }

    /// Whether the panel is up, given how many choices there are.
    pub(crate) fn is_open(self, choices: usize) -> bool {
        self.shown == Shown::Visible && choices > 0
    }
}

/// The first enabled choice going down, the last going up: where the cursor lands from nowhere.
fn landing(live: &[Availability], step: Step) -> Option<usize> {
    let enabled = |index: &usize| live[*index] == Availability::Enabled;
    match step {
        Step::Down => (0..live.len()).find(enabled),
        Step::Up => (0..live.len()).rev().find(enabled),
    }
}

/// What `key` does to `state`, over choices with these `live` availabilities, in a field holding
/// `text`.
pub(crate) fn press(
    state: Suggesting,
    key: SuggestKey,
    live: &[Availability],
    text: Text,
) -> (Suggesting, Act) {
    let open = state.is_open(live.len());
    match key {
        SuggestKey::Up | SuggestKey::Down => {
            let step = if key == SuggestKey::Up {
                Step::Up
            } else {
                Step::Down
            };
            let from = state.cursor.filter(|_| open);
            let to = match from {
                Some(at) => Some(moved_live(Wrap::Wraps, at, live, step)),
                None => landing(live, step),
            };
            match to {
                Some(cursor) => (
                    Suggesting {
                        shown: Shown::Visible,
                        cursor: Some(cursor),
                    },
                    Act::Swallow,
                ),
                None => (state, Act::Pass),
            }
        }
        SuggestKey::Escape => match (open, text) {
            (true, _) => (Suggesting::closed(), Act::Swallow),
            (false, Text::Filled) => (state, Act::Clear),
            (false, Text::Empty) => (state, Act::Pass),
        },
        SuggestKey::Enter => match state.cursor.filter(|_| open) {
            Some(at) if live.get(at) == Some(&Availability::Enabled) => {
                (Suggesting::closed(), Act::Pick(at))
            }
            Some(_) | None => (Suggesting::closed(), Act::Submit),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Act, SuggestKey, Suggesting, Text, press};
    use ds_core::vocab::Availability::{Disabled as D, Enabled as E};
    use ds_core::vocab::{Availability, Shown};

    const OPEN: Suggesting = Suggesting {
        shown: Shown::Visible,
        cursor: None,
    };

    fn at(cursor: usize) -> Suggesting {
        Suggesting {
            shown: Shown::Visible,
            cursor: Some(cursor),
        }
    }

    #[test]
    fn keys_move_the_cursor_pick_and_close_in_order() {
        use SuggestKey::{Down, Enter, Escape, Up};
        type Case = (
            &'static str,
            Suggesting,
            SuggestKey,
            &'static [Availability],
            Text,
            (Suggesting, Act),
        );
        let hidden = Suggesting::closed();
        #[rustfmt::skip]
        let cases: &[Case] = &[
            ("down from nothing lands on the first", OPEN, Down, &[E, E, E], Text::Filled, (at(0), Act::Swallow)),
            ("up from nothing lands on the last", OPEN, Up, &[E, E, E], Text::Filled, (at(2), Act::Swallow)),
            ("down skips a disabled choice", at(0), Down, &[E, D, E], Text::Filled, (at(2), Act::Swallow)),
            ("down from the last wraps", at(2), Down, &[E, E, E], Text::Filled, (at(0), Act::Swallow)),
            ("down opens a closed panel on the first", hidden, Down, &[E, E], Text::Filled, (at(0), Act::Swallow)),
            ("no choices: the arrows are the caret's", OPEN, Down, &[], Text::Filled, (OPEN, Act::Pass)),
            ("enter picks the highlighted choice", at(1), Enter, &[E, E], Text::Filled, (hidden, Act::Pick(1))),
            ("enter with nothing highlighted submits", OPEN, Enter, &[E, E], Text::Filled, (hidden, Act::Submit)),
            ("enter on a closed panel submits", Suggesting { shown: Shown::Hidden, cursor: Some(0) }, Enter, &[E], Text::Filled, (hidden, Act::Submit)),
            ("enter on a disabled choice submits", at(0), Enter, &[D], Text::Filled, (hidden, Act::Submit)),
            ("escape closes the panel first", at(1), Escape, &[E, E], Text::Filled, (hidden, Act::Swallow)),
            ("escape then clears the text", hidden, Escape, &[E, E], Text::Filled, (hidden, Act::Clear)),
            ("escape with no panel and no text goes on", hidden, Escape, &[E, E], Text::Empty, (hidden, Act::Pass)),
            ("escape with no choices clears at once", OPEN, Escape, &[], Text::Filled, (OPEN, Act::Clear)),
        ];
        for &(name, state, key, live, text, want) in cases {
            assert_eq!(press(state, key, live, text), want, "{name}");
        }
    }
}
