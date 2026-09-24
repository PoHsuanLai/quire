//! What a menu panel lists and how the keyboard moves through it: pure, shared by `Menu`, its
//! submenus and the command palette (design/06-INTERACTIONS.md sections 2.3, 2.4 and 11;
//! design/13-BEHAVIOUR-menus-windows.md section 13.3).
//!
//! A *choice* is a line the selection can rest on: an item or a submenu parent, enabled or
//! not. A header, a status line and a rule are not choices. Choices are numbered in order; the selection, a click and the menu tracker's item path
//! all name a choice by that number. Up and Down skip disabled choices.

use crate::components::menu_entry::{MenuEntry, fuzzy};
use crate::components::vocab::Availability;
use dioxus::prelude::*;

/// How Up and Down move past the ends: a floating menu wraps, the command palette clamps
/// (design/06-INTERACTIONS.md sections 2.3 and 2.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nav {
    /// Past the last is the first.
    Wrap,
    /// Past the last stays on the last.
    Clamp,
}

/// One step of the selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// Down: the next item.
    Down,
    /// Up: the previous item.
    Up,
}

/// The selection after `step` from `selected` among `count` items.
pub(crate) fn moved(nav: Nav, selected: usize, count: usize, step: Step) -> usize {
    if count == 0 {
        return 0;
    }
    let last = count - 1;
    let selected = selected.min(last);
    match (nav, step) {
        (Nav::Wrap, Step::Down) => (selected + 1) % count,
        (Nav::Wrap, Step::Up) => (selected + last) % count,
        (Nav::Clamp, Step::Down) => (selected + 1).min(last),
        (Nav::Clamp, Step::Up) => selected.saturating_sub(1),
    }
}

/// The selection after `step` from `selected`, skipping disabled choices (design/13 section
/// 13.3.3). With no enabled choice the selection stays; clamping at an end with only disabled
/// choices past it stays too.
pub(crate) fn moved_live(nav: Nav, selected: usize, live: &[Availability], step: Step) -> usize {
    let mut at = selected.min(live.len().saturating_sub(1));
    for _ in 0..live.len() {
        let next = moved(nav, at, live.len(), step);
        if next == at {
            return selected;
        }
        if live[next] == Availability::Enabled {
            return next;
        }
        at = next;
    }
    selected
}

/// The enabled choice the selection shows as: `selected` itself, or the first enabled one
/// after it (wrapping), or `selected` when none is enabled.
pub(crate) fn settled(selected: usize, live: &[Availability]) -> usize {
    let count = live.len();
    if count == 0 {
        return 0;
    }
    let start = selected.min(count - 1);
    (0..count)
        .map(|offset| (start + offset) % count)
        .find(|&index| live[index] == Availability::Enabled)
        .unwrap_or(start)
}

/// What a key does in a menu panel, before the panel decides what that means for the choice
/// under the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeyAct {
    /// Up or Down.
    Move(Step),
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
    /// Take the query's last character off.
    Erase,
}

/// Whether typing filters the entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Filter {
    /// Typing filters with the fuzzy ranker and resets the selection.
    Typing,
    /// The entries are fixed.
    #[default]
    None,
}

/// A panel's reading of `key` (design/06-INTERACTIONS.md section 2.4, design/13 section
/// 13.3.2); `None` for a key it leaves alone. Typing only counts with [`Filter::Typing`] and no
/// Ctrl, Alt or Super.
pub(crate) fn key_act(key: &Key, modifiers: Modifiers, filter: Filter) -> Option<KeyAct> {
    let chord = modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
    match key {
        Key::ArrowDown => Some(KeyAct::Move(Step::Down)),
        Key::ArrowUp => Some(KeyAct::Move(Step::Up)),
        Key::ArrowRight => Some(KeyAct::Open),
        Key::ArrowLeft => Some(KeyAct::Back),
        Key::Enter | Key::Tab => Some(KeyAct::Pick),
        Key::Escape => Some(KeyAct::Close),
        Key::Backspace if filter == Filter::Typing => Some(KeyAct::Erase),
        Key::Character(text) if filter == Filter::Typing && !chord => {
            Some(KeyAct::Type(text.clone()))
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
            }),
            MenuEntry::Row(row) => Some(Choice {
                act: Act::Pick(row.value.clone()),
                availability: row.availability,
            }),
            MenuEntry::Submenu {
                children,
                availability,
                ..
            } => Some(Choice {
                act: Act::Open(children.clone()),
                availability: *availability,
            }),
            MenuEntry::Header(_) | MenuEntry::Info { .. } | MenuEntry::Separator => None,
        })
        .collect()
}

/// Each choice's availability, in order.
pub(crate) fn liveness<T>(choices: &[Choice<T>]) -> Vec<Availability> {
    choices.iter().map(|choice| choice.availability).collect()
}

#[cfg(test)]
mod tests {
    use super::{Filter, KeyAct, Nav, Step, key_act, moved, moved_live, settled};
    use crate::components::vocab::Availability::{Disabled as D, Enabled as E};
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn a_menu_wraps_and_the_palette_clamps() {
        // (nav, selected, count, step, want)
        #[rustfmt::skip]
        const CASES: &[(Nav, usize, usize, Step, usize)] = &[
            (Nav::Wrap, 0, 3, Step::Down, 1),
            (Nav::Wrap, 2, 3, Step::Down, 0),
            (Nav::Wrap, 0, 3, Step::Up, 2),
            (Nav::Wrap, 1, 3, Step::Up, 0),
            (Nav::Wrap, 0, 1, Step::Down, 0),
            (Nav::Wrap, 0, 0, Step::Down, 0),
            (Nav::Wrap, 7, 3, Step::Down, 0),
            (Nav::Clamp, 0, 3, Step::Down, 1),
            (Nav::Clamp, 2, 3, Step::Down, 2),
            (Nav::Clamp, 0, 3, Step::Up, 0),
            (Nav::Clamp, 2, 3, Step::Up, 1),
            (Nav::Clamp, 0, 0, Step::Up, 0),
        ];
        for &(nav, selected, count, step, want) in CASES {
            assert_eq!(
                moved(nav, selected, count, step),
                want,
                "{nav:?} {step:?} from {selected} of {count}"
            );
        }
    }

    #[test]
    fn up_and_down_skip_disabled_choices() {
        // (nav, selected, live, step, want)
        #[rustfmt::skip]
        let cases: &[(Nav, usize, &[_], Step, usize)] = &[
            (Nav::Wrap, 0, &[E, D, E], Step::Down, 2),
            (Nav::Wrap, 2, &[E, D, E], Step::Up, 0),
            (Nav::Wrap, 2, &[D, E, E], Step::Down, 1),
            (Nav::Wrap, 0, &[E, D, D], Step::Down, 0),
            (Nav::Wrap, 0, &[D, D, D], Step::Down, 0),
            (Nav::Clamp, 1, &[E, E, D], Step::Down, 1),
            (Nav::Clamp, 2, &[E, D, E], Step::Up, 0),
            (Nav::Wrap, 0, &[], Step::Down, 0),
        ];
        for &(nav, selected, live, step, want) in cases {
            assert_eq!(
                moved_live(nav, selected, live, step),
                want,
                "{nav:?} {step:?} from {selected} in {live:?}"
            );
        }
    }

    #[test]
    fn the_selection_settles_on_an_enabled_choice() {
        #[rustfmt::skip]
        let cases: &[(usize, &[_], usize)] = &[
            (0, &[E, E], 0),
            (0, &[D, E], 1),
            (1, &[E, D], 0),
            (0, &[D, D], 0),
            (5, &[E, D, E], 2),
            (0, &[], 0),
        ];
        for &(selected, live, want) in cases {
            assert_eq!(settled(selected, live), want, "{selected} in {live:?}");
        }
    }

    #[test]
    fn menu_keys_follow_section_2_4_and_13_3() {
        let none = Modifiers::empty();
        let a = Key::Character("a".to_string());
        #[rustfmt::skip]
        let cases: Vec<(Key, Modifiers, Filter, Option<KeyAct>)> = vec![
            (Key::ArrowDown, none, Filter::None, Some(KeyAct::Move(Step::Down))),
            (Key::ArrowUp, none, Filter::None, Some(KeyAct::Move(Step::Up))),
            (Key::ArrowRight, none, Filter::None, Some(KeyAct::Open)),
            (Key::ArrowLeft, none, Filter::Typing, Some(KeyAct::Back)),
            (Key::Enter, none, Filter::None, Some(KeyAct::Pick)),
            (Key::Tab, none, Filter::None, Some(KeyAct::Pick)),
            (Key::Escape, none, Filter::None, Some(KeyAct::Close)),
            (a.clone(), none, Filter::Typing, Some(KeyAct::Type("a".to_string()))),
            (a.clone(), Modifiers::SHIFT, Filter::Typing, Some(KeyAct::Type("a".to_string()))),
            (a.clone(), Modifiers::CONTROL, Filter::Typing, None),
            (a, none, Filter::None, None),
            (Key::Backspace, none, Filter::Typing, Some(KeyAct::Erase)),
            (Key::Backspace, none, Filter::None, None),
        ];
        for (key, modifiers, filter, want) in cases {
            assert_eq!(
                key_act(&key, modifiers, filter),
                want,
                "{key:?} {modifiers:?}"
            );
        }
    }
}
