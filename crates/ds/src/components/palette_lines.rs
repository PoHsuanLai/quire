//! What the command palette lists and how its keys read: pure, beside `command_palette`.

use crate::components::menu_entry::{MenuEntry, fuzzy};
use crate::components::menu_lines::{Line, Step};
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

/// `entries` as menu lines, each item's title marked where `query` matches it.
pub(crate) fn marked<'a, T>(entries: &'a [MenuEntry<T>], query: &str) -> Vec<Line<'a, T>> {
    entries
        .iter()
        .map(|entry| Line {
            entry,
            marks: match (entry.takes_marks(), entry.match_title()) {
                (true, Some(title)) => fuzzy(query, &title)
                    .map(|hit| hit.marks)
                    .unwrap_or_default(),
                (_, _) => Vec::new(),
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PaletteKey, palette_key};
    use crate::components::menu_lines::Step;
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
