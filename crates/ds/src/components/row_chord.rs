//! A palette or menu row's action chord (launcher chords): the keys that run the row's first
//! action, drawn as a plain [`Chord`](crate::Chord) at the end of its trail. Spotlight shows it
//! only on the highlighted row, so the list reads as its data (a file's time) and the hint
//! follows the selection; `Always` keeps it on every row, as a menu's shortcuts are.

use crate::components::vocab::{Selection, Shortcut};

/// When a row's chord shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChordShown {
    /// Only while the row is the selection (`aria-selected`): Spotlight's hint.
    #[default]
    Selected,
    /// On every row.
    Always,
}

/// A row's chord and when it shows. The default is no chord (an empty shortcut draws nothing).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct RowChord {
    /// The keys.
    pub shortcut: Shortcut,
    /// When they show.
    pub shown: ChordShown,
}

impl RowChord {
    /// `shortcut`, shown only on the selected row.
    pub fn on_selected(shortcut: Shortcut) -> Self {
        RowChord {
            shortcut,
            shown: ChordShown::Selected,
        }
    }

    /// `shortcut`, shown on every row.
    pub fn always(shortcut: Shortcut) -> Self {
        RowChord {
            shortcut,
            shown: ChordShown::Always,
        }
    }
}

/// No chord, for the entries that carry none (an `Item`, a `Submenu`).
pub(crate) const NO_CHORD: &RowChord = &RowChord {
    shortcut: Shortcut(Vec::new()),
    shown: ChordShown::Selected,
};

/// The chord a row draws in `selection`: `None` when it has none or it waits for the selection.
pub(crate) fn shown_chord(chord: &RowChord, selection: Selection) -> Option<&Shortcut> {
    let visible = match (chord.shown, selection) {
        (ChordShown::Always, _) | (ChordShown::Selected, Selection::Selected) => true,
        (ChordShown::Selected, Selection::Unselected) => false,
    };
    (visible && !chord.shortcut.0.is_empty()).then_some(&chord.shortcut)
}

#[cfg(test)]
mod tests {
    use super::{RowChord, shown_chord};
    use crate::components::vocab::{Key, Selection, Shortcut};

    #[test]
    fn a_chord_shows_by_its_rule() {
        let keys = Shortcut(vec![Key::Super, Key::Char('r')]);
        let cases = [
            (
                "selected rule, selected",
                RowChord::on_selected(keys.clone()),
                Selection::Selected,
                true,
            ),
            (
                "selected rule, unselected",
                RowChord::on_selected(keys.clone()),
                Selection::Unselected,
                false,
            ),
            (
                "always, unselected",
                RowChord::always(keys.clone()),
                Selection::Unselected,
                true,
            ),
            (
                "always, selected",
                RowChord::always(keys.clone()),
                Selection::Selected,
                true,
            ),
            (
                "empty, selected",
                RowChord::default(),
                Selection::Selected,
                false,
            ),
            (
                "empty always",
                RowChord::always(Shortcut::default()),
                Selection::Unselected,
                false,
            ),
        ];
        for (name, chord, selection, shows) in cases {
            assert_eq!(shown_chord(&chord, selection).is_some(), shows, "{name}");
        }
    }
}
