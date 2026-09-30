//! What the command palette lists and how its keys read: pure, beside `command_palette`.

use crate::components::content::text_runs::TextLine;
use crate::components::menus::menu_match::fuzzy;
use crate::components::menus::palette::palette_group::PaletteRow;
use crate::stack::roving::Step;
use dioxus::prelude::Key;

/// What a key in the search field does to the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteKey {
    /// Up or Down: move the cursor, clamped.
    Move(Step),
    /// Left (`Step::Up`) or Right (`Step::Down`): move the cursor inside an emoji grid only.
    Side(Step),
    /// Close, then run the selection.
    Run,
    /// Close.
    Close,
}

/// The palette's reading of a key (design/06-INTERACTIONS.md section 2.3).
pub(crate) fn palette_key(key: &Key) -> Option<PaletteKey> {
    match key {
        Key::ArrowDown => Some(PaletteKey::Move(Step::Down)),
        Key::ArrowUp => Some(PaletteKey::Move(Step::Up)),
        Key::ArrowLeft => Some(PaletteKey::Side(Step::Up)),
        Key::ArrowRight => Some(PaletteKey::Side(Step::Down)),
        Key::Enter => Some(PaletteKey::Run),
        Key::Escape => Some(PaletteKey::Close),
        _ => None,
    }
}

/// A row and the title characters the query matched.
pub(crate) struct MarkedRow<'a, T> {
    /// The row as given.
    pub row: &'a PaletteRow<T>,
    /// The matched title characters, by index; none for a title the caller marked itself.
    pub marks: Vec<usize>,
}

/// `rows`, each plain title marked where `query` matches it.
pub(crate) fn marked<'a, T>(rows: &'a [PaletteRow<T>], query: &str) -> Vec<MarkedRow<'a, T>> {
    rows.iter()
        .map(|row| MarkedRow {
            row,
            marks: match &row.title {
                TextLine::Plain(title) => {
                    fuzzy(query, title).map(|hit| hit.marks).unwrap_or_default()
                }
                TextLine::Runs(_) => Vec::new(),
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PaletteKey, palette_key};
    use crate::stack::roving::Step;
    use dioxus::prelude::Key;

    #[test]
    fn palette_keys_follow_section_2_3() {
        #[rustfmt::skip]
        let cases: Vec<(Key, Option<PaletteKey>)> = vec![
            (Key::ArrowDown, Some(PaletteKey::Move(Step::Down))),
            (Key::ArrowUp, Some(PaletteKey::Move(Step::Up))),
            (Key::ArrowLeft, Some(PaletteKey::Side(Step::Up))),
            (Key::ArrowRight, Some(PaletteKey::Side(Step::Down))),
            (Key::Enter, Some(PaletteKey::Run)),
            (Key::Escape, Some(PaletteKey::Close)),
            // Tab is the menu's pick, not the palette's.
            (Key::Tab, None),
            (Key::Character("a".to_string()), None),
        ];
        for (key, want) in cases {
            assert_eq!(palette_key(&key), want, "{key:?}");
        }
    }
}
