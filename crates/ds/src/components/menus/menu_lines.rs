//! What a menu panel lists and how the keyboard moves through it: pure, shared by `Menu`, its
//! submenus and the command palette (design/06-INTERACTIONS.md sections 2.3, 2.4 and 11;
//! design/13-BEHAVIOUR-menus-windows.md section 13.3).
//!
//! A *choice* is a line the selection can rest on: an item or a submenu parent, enabled or
//! not. A header, a status line and a rule are not choices. Choices are numbered in order; the selection, a click and the menu tracker's item path
//! all name a choice by that number. Up and Down skip disabled choices.

use crate::components::menus::menu_filter::MenuFilter;
use crate::components::menus::menu_filter::Typed;
use crate::components::menus::{menu_entry::MenuEntry, menu_match::fuzzy};
use crate::stack::roving::{Edge, Step};
use dioxus::prelude::*;
use ds_core::vocab::Availability;

/// What a key does in a menu panel, before the panel decides what that means for the choice
/// under the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeyAct {
    /// Up or Down.
    Move(Step),
    /// Home or End.
    Edge(Edge),
    /// Enter or Tab.
    Pick,
    /// Right: open the selected parent's submenu.
    Open,
    /// Left: close this level's open submenu, or go back to the parent menu.
    Back,
    /// Escape: close one level.
    Close,
    /// Add to the query.
    Type(String),
    /// Letters typed with no query to add them to: jump to the choice that starts with them.
    Jump(String),
    /// Take the query's last character off.
    Erase,
}

/// A panel's reading of `key` (design/06-INTERACTIONS.md section 2.4, design/13 section
/// 13.3.2); `None` for a key it leaves alone. Typing only counts under a filter that types
/// ([`Filter::Typing`] or [`Filter::Field`]) and with no Ctrl, Alt or Super.
pub(crate) fn key_act(key: &Key, modifiers: Modifiers, filter: &MenuFilter) -> Option<KeyAct> {
    let chord = modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
    match key {
        Key::ArrowDown => Some(KeyAct::Move(Step::Down)),
        Key::ArrowUp => Some(KeyAct::Move(Step::Up)),
        Key::Home => Some(KeyAct::Edge(Edge::First)),
        Key::End => Some(KeyAct::Edge(Edge::Last)),
        Key::ArrowRight => Some(KeyAct::Open),
        Key::ArrowLeft => Some(KeyAct::Back),
        Key::Enter | Key::Tab => Some(KeyAct::Pick),
        Key::Escape => Some(KeyAct::Close),
        Key::Backspace if filter.types() == Typed::Yes => Some(KeyAct::Erase),
        Key::Character(text) if filter.types() == Typed::Yes && !chord => {
            Some(KeyAct::Type(text.clone()))
        }
        Key::Character(text) if !chord && !text.trim().is_empty() => {
            Some(KeyAct::Jump(text.clone()))
        }
        _ => None,
    }
}

/// One line the menu shows, and the title characters the query matched.
pub(crate) struct Line<'a, T> {
    /// The entry.
    pub entry: &'a MenuEntry<T>,
    /// The matched title characters.
    pub marks: Vec<usize>,
}

/// What the menu lists for `query`: every entry as given when it is empty; otherwise only the
/// matching choices, best first, their headers and rules dropped (`S:2086-2088`).
pub(crate) fn lines<'a, T>(entries: &'a [MenuEntry<T>], query: &str) -> Vec<Line<'a, T>> {
    if query.is_empty() {
        return entries
            .iter()
            .map(|entry| Line {
                entry,
                marks: Vec::new(),
            })
            .collect();
    }
    let mut hits: Vec<(f32, Line<'a, T>)> = entries
        .iter()
        .filter_map(|entry| {
            let hit = fuzzy(query, &entry.match_title()?)?;
            let marks = if entry.takes_marks() {
                hit.marks
            } else {
                Vec::new()
            };
            Some((hit.score, Line { entry, marks }))
        })
        .collect();
    hits.sort_by(|a, b| b.0.total_cmp(&a.0));
    hits.into_iter().map(|(_, line)| line).collect()
}

/// What a choice does when it is picked.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Act<T> {
    /// Yield this value.
    Pick(T),
    /// Open this submenu.
    Open(Vec<MenuEntry<T>>),
}

/// One choice: what it does, and whether it can.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Choice<T> {
    /// Pick or open.
    pub act: Act<T>,
    /// Enabled or disabled.
    pub availability: Availability,
    /// What the choice is called: what typing a letter matches.
    pub title: String,
}

/// The choices of `lines`, in order: what a choice number names.
pub(crate) fn choices<T: Clone>(lines: &[Line<'_, T>]) -> Vec<Choice<T>> {
    lines
        .iter()
        .filter_map(|line| match line.entry {
            MenuEntry::Item {
                value,
                availability,
                ..
            } => Some(Choice {
                act: Act::Pick(value.clone()),
                availability: *availability,
                title: line.entry.match_title().unwrap_or_default(),
            }),
            MenuEntry::Row(row) => Some(Choice {
                act: Act::Pick(row.value.clone()),
                availability: row.availability,
                title: line.entry.match_title().unwrap_or_default(),
            }),
            MenuEntry::Submenu {
                children,
                availability,
                ..
            } => Some(Choice {
                act: Act::Open(children.clone()),
                availability: *availability,
                title: line.entry.match_title().unwrap_or_default(),
            }),
            MenuEntry::Header(_) | MenuEntry::Info { .. } | MenuEntry::Separator => None,
        })
        .collect()
}

/// How many of `lines` are choices.
pub(crate) fn choices_len<T>(lines: &[Line<'_, T>]) -> usize {
    lines
        .iter()
        .filter(|line| {
            matches!(
                line.entry,
                MenuEntry::Item { .. } | MenuEntry::Row(_) | MenuEntry::Submenu { .. }
            )
        })
        .count()
}

/// Each choice's availability, in order.
pub(crate) fn liveness<T>(choices: &[Choice<T>]) -> Vec<Availability> {
    choices.iter().map(|choice| choice.availability).collect()
}

#[cfg(test)]
mod tests {
    use super::{KeyAct, key_act};
    use crate::components::menus::menu_filter::MenuFilter;
    use crate::stack::roving::{Edge, Step};
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn menu_keys_follow_section_2_4_and_13_3() {
        let none = Modifiers::empty();
        let a = Key::Character("a".to_string());
        #[rustfmt::skip]
        let cases: Vec<(Key, Modifiers, MenuFilter, Option<KeyAct>)> = vec![
            (Key::ArrowDown, none, MenuFilter::None, Some(KeyAct::Move(Step::Down))),
            (Key::ArrowUp, none, MenuFilter::None, Some(KeyAct::Move(Step::Up))),
            (Key::Home, none, MenuFilter::None, Some(KeyAct::Edge(Edge::First))),
            (Key::End, none, MenuFilter::Typing, Some(KeyAct::Edge(Edge::Last))),
            (Key::ArrowRight, none, MenuFilter::None, Some(KeyAct::Open)),
            (Key::ArrowLeft, none, MenuFilter::Typing, Some(KeyAct::Back)),
            (Key::Enter, none, MenuFilter::None, Some(KeyAct::Pick)),
            (Key::Tab, none, MenuFilter::None, Some(KeyAct::Pick)),
            (Key::Escape, none, MenuFilter::None, Some(KeyAct::Close)),
            (a.clone(), none, MenuFilter::Typing, Some(KeyAct::Type("a".to_string()))),
            (a.clone(), Modifiers::SHIFT, MenuFilter::Typing, Some(KeyAct::Type("a".to_string()))),
            (a.clone(), Modifiers::CONTROL, MenuFilter::Typing, None),
            (a, none, MenuFilter::None, Some(KeyAct::Jump("a".to_string()))),
            (Key::Character(" ".to_string()), none, MenuFilter::None, None),
            (Key::Backspace, none, MenuFilter::Typing, Some(KeyAct::Erase)),
            (Key::Backspace, none, MenuFilter::None, None),
            (Key::Character("b".to_string()), none, field(), Some(KeyAct::Type("b".to_string()))),
            (Key::Backspace, none, field(), Some(KeyAct::Erase)),
            (Key::ArrowDown, none, field(), Some(KeyAct::Move(Step::Down))),
        ];
        fn field() -> MenuFilter {
            MenuFilter::Field {
                placeholder: "Filter…".to_string(),
            }
        }
        for (key, modifiers, filter, want) in cases {
            assert_eq!(
                key_act(&key, modifiers, &filter),
                want,
                "{key:?} {modifiers:?}"
            );
        }
    }
}
