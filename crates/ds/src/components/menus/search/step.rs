//! What a key does to a search field's suggestions: pure, `(state, key) -> (state, act)`. The
//! keyboard never leaves the field, so these are the field's keys: Up and Down move a cursor over
//! the choices, Enter picks the one under it, Escape takes one step each press (in the order
//! `EscapeOrder` says: the panel then the text, or the text then the window). Tab is not here: it
//! behaves as it always does.

use crate::components::menus::search::model::{Ends, EscapeOrder, Manner, Panel};
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

    /// Whether the panel is up, given how many choices there are: when the field owns the panel
    /// it also needs to be wanted; a host that shows it wants it always.
    pub(crate) fn is_open(self, choices: usize, panel: Panel) -> bool {
        let wanted = panel == Panel::Shown || self.shown == Shown::Visible;
        wanted && choices > 0
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

/// What Escape does: one step, never a press that changes nothing. Clearing closes the panel
/// with the text, so the next Escape is the window's.
/// A panel the host shows stays up: Escape clears the text, or goes on to the window.
fn escape(state: Suggesting, open: bool, text: Text, manner: Manner) -> (Suggesting, Act) {
    match (manner.escape, open, text) {
        (_, _, Text::Empty) if manner.panel == Panel::Shown => (state, Act::Pass),
        (EscapeOrder::ClosePanelFirst, true, _) if manner.panel == Panel::Owned => {
            (Suggesting::closed(), Act::Swallow)
        }
        (_, _, Text::Filled) => (Suggesting::closed(), Act::Clear),
        (_, _, Text::Empty) => (Suggesting::closed(), Act::Pass),
    }
}

/// What `key` does to `state`, over choices with these `live` availabilities, in a field holding
/// `text`, behaving as `manner` says.
pub(crate) fn press(
    state: Suggesting,
    key: SuggestKey,
    live: &[Availability],
    text: Text,
    manner: Manner,
) -> (Suggesting, Act) {
    let open = state.is_open(live.len(), manner.panel);
    let wrap = match manner.ends {
        Ends::Wrap => Wrap::Wraps,
        Ends::Stop => Wrap::Stops,
    };
    match key {
        SuggestKey::Up | SuggestKey::Down => {
            let step = if key == SuggestKey::Up {
                Step::Up
            } else {
                Step::Down
            };
            let from = state.cursor.filter(|_| open);
            let to = match from {
                Some(at) => Some(moved_live(wrap, at, live, step)),
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
        SuggestKey::Escape => escape(state, open, text, manner),
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
    use crate::components::menus::search::model::{Ends, EscapeOrder, Manner, Panel};
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
            ("escape with no choices clears at once", OPEN, Escape, &[], Text::Filled, (Suggesting::closed(), Act::Clear)),
        ];
        for &(name, state, key, live, text, want) in cases {
            assert_eq!(
                press(state, key, live, text, Manner::default()),
                want,
                "{name}"
            );
        }
    }

    #[test]
    fn each_escape_is_one_step_in_either_order() {
        use EscapeOrder::{ClearFirst, ClosePanelFirst};
        let live: &[Availability] = &[E, E];
        let presses = |order, start: Suggesting, text| {
            let (mut state, mut text, mut acts) = (start, text, Vec::new());
            while acts.last() != Some(&Act::Pass) {
                let manner = Manner {
                    escape: order,
                    ..Manner::default()
                };
                let (next, act) = press(state, SuggestKey::Escape, live, text, manner);
                if act == Act::Clear {
                    text = Text::Empty;
                }
                (state, acts) = (next, [acts, vec![act]].concat());
            }
            acts
        };
        assert_eq!(
            presses(ClosePanelFirst, at(0), Text::Filled),
            vec![Act::Swallow, Act::Clear, Act::Pass]
        );
        assert_eq!(
            presses(ClearFirst, at(0), Text::Filled),
            vec![Act::Clear, Act::Pass]
        );
        assert_eq!(presses(ClearFirst, OPEN, Text::Empty), vec![Act::Pass]);
        assert_eq!(
            presses(ClosePanelFirst, OPEN, Text::Empty),
            vec![Act::Swallow, Act::Pass]
        );
    }

    #[test]
    fn a_panel_the_host_shows_keeps_its_rows_on_escape_with_no_text() {
        let live: &[Availability] = &[E, E];
        let shown = Manner {
            panel: Panel::Shown,
            ..Manner::default()
        };
        let closed = Suggesting::closed();
        assert!(closed.is_open(2, Panel::Shown));
        assert!(!closed.is_open(2, Panel::Owned));
        assert_eq!(
            press(at(1), SuggestKey::Escape, live, Text::Empty, shown),
            (at(1), Act::Pass)
        );
        assert_eq!(
            press(closed, SuggestKey::Down, live, Text::Empty, shown).1,
            Act::Swallow
        );
        assert_eq!(
            press(at(1), SuggestKey::Escape, live, Text::Filled, shown).1,
            Act::Clear
        );
    }

    #[test]
    fn the_arrows_stop_at_the_ends_when_asked() {
        let live: &[Availability] = &[E, E, E];
        let stop = Manner {
            ends: Ends::Stop,
            ..Manner::default()
        };
        let moved = |from, key, manner| press(at(from), key, live, Text::Filled, manner).0;
        assert_eq!(moved(2, SuggestKey::Down, stop), at(2));
        assert_eq!(moved(0, SuggestKey::Up, stop), at(0));
        assert_eq!(moved(1, SuggestKey::Down, stop), at(2));
        assert_eq!(moved(2, SuggestKey::Down, Manner::default()), at(0));
    }
}
