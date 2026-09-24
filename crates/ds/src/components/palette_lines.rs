//! What the command palette lists and how its keys read: pure, beside `command_palette`.

use crate::components::menu_entry::{MenuEntry, fuzzy};
use crate::components::menu_lines::{Line, Step};
use dioxus::prelude::Key;

/// What a key in the search field does to the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaletteKey {
    /// Move the selection, clamped.
    Move(Step),
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
        Key::Enter => Some(PaletteKey::Run),
        Key::Escape => Some(PaletteKey::Close),
        _ => None,
    }
}

/// The groups as one list: each non-empty group's title as a header, then its items.
pub(crate) fn headed<T: Clone>(groups: &[(String, Vec<MenuEntry<T>>)]) -> Vec<MenuEntry<T>> {
    groups
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .flat_map(|(title, entries)| {
            std::iter::once(MenuEntry::Header(title.clone())).chain(entries.iter().cloned())
        })
        .collect()
}

/// `entries` as menu lines, each item's title marked where `query` matches it.
pub(crate) fn marked<'a, T>(entries: &'a [MenuEntry<T>], query: &str) -> Vec<Line<'a, T>> {
    entries
        .iter()
        .map(|entry| Line {
            entry,
            marks: match entry {
                MenuEntry::Item { title, .. } | MenuEntry::Submenu { title, .. } => {
                    fuzzy(query, title).map(|hit| hit.marks).unwrap_or_default()
                }
                MenuEntry::Header(_) | MenuEntry::Info { .. } | MenuEntry::Separator => Vec::new(),
            },
        })
        .collect()
}

/// For each choice (item or submenu parent, in order), the line it is drawn on: the element a
/// choice's rect is read from is the one at that line.
pub(crate) fn choice_lines<T>(entries: &[MenuEntry<T>]) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| matches!(entry, MenuEntry::Item { .. } | MenuEntry::Submenu { .. }))
        .map(|(line, _)| line)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{PaletteKey, choice_lines, headed, palette_key};
    use crate::components::menu_entry::{MenuEntry, Trail};
    use crate::components::menu_lines::Step;
    use crate::components::vocab::Availability;
    use dioxus::prelude::Key;

    #[test]
    fn palette_keys_follow_section_2_3() {
        #[rustfmt::skip]
        let cases: Vec<(Key, Option<PaletteKey>)> = vec![
            (Key::ArrowDown, Some(PaletteKey::Move(Step::Down))),
            (Key::ArrowUp, Some(PaletteKey::Move(Step::Up))),
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

    fn item(value: u8) -> MenuEntry<u8> {
        MenuEntry::Item {
            value,
            title: format!("item {value}"),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: None,
            availability: Availability::Enabled,
        }
    }

    #[test]
    fn each_choice_names_the_line_it_is_drawn_on() {
        // (group sizes, the line of each choice); an empty group draws no header.
        const CASES: &[(&[u8], &[usize])] = &[
            (&[2], &[1, 2]),
            (&[1, 2], &[1, 3, 4]),
            (&[0, 2], &[1, 2]),
            (&[], &[]),
        ];
        for (sizes, want) in CASES {
            let groups: Vec<(String, Vec<MenuEntry<u8>>)> = sizes
                .iter()
                .map(|&size| (format!("{size}"), (0..size).map(item).collect()))
                .collect();
            assert_eq!(choice_lines(&headed(&groups)), *want, "{sizes:?}");
        }
    }
}
