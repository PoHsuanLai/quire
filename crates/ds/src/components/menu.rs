//! Menu: "ONE MENU: every list of choices in the window uses this" (design/04-COMPONENTS.md
//! section 20). Filtering and ranking are pure Rust (design/06-INTERACTIONS.md section 11).
//!
//! The menu takes focus when it opens and handles its own keys wherever it opens
//! (design/06-INTERACTIONS.md section 2.4): Up and Down wrap, Enter or Tab picks, Escape
//! closes the topmost layer only, and with [`Filter::Typing`] typing filters with the fuzzy
//! ranker and resets the selection.

use crate::components::menu_entry::{ItemView, MenuEntry, Row, fuzzy, item};
use crate::components::popover::{
    Dismiss, Stacking, escape_closes, position_style, use_entrance, use_float,
};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::vocab::Selection;
use crate::geometry::{Align, Anchor, Placement, Point, Px, Rect, Side};
use crate::motion::anim::Anim;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// Which menu shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuKind {
    /// 280 wide, 34 px tiles, title and help and shortcut.
    Rich,
    /// 220 wide, 22 px tiles.
    Slim,
    /// C's check-column menu, anchored under its button's right edge.
    Dropdown,
    /// Anchored at the pointer.
    Context,
}

impl MenuKind {
    /// The `data-kind` word.
    fn slug(self) -> &'static str {
        match self {
            MenuKind::Rich => "rich",
            MenuKind::Slim => "slim",
            MenuKind::Dropdown => "dropdown",
            MenuKind::Context => "context",
        }
    }

    /// Where the menu goes against its anchor: S's floating menus at `left - 8`, 6 below
    /// (`S:2060-2065`); C's dropdown under the trigger's right edge, 7 below; a context menu
    /// at the pointer (design/06-INTERACTIONS.md section 4).
    fn placement(self, anchor: Rect) -> (Rect, Placement, Px) {
        match self {
            MenuKind::Rich | MenuKind::Slim => (
                Rect {
                    origin: Point {
                        x: anchor.origin.x - Px(8.0),
                        ..anchor.origin
                    },
                    ..anchor
                },
                Placement::new(Side::Bottom, Align::Start),
                Px(6.0),
            ),
            MenuKind::Dropdown => (anchor, Placement::new(Side::Bottom, Align::End), Px(7.0)),
            MenuKind::Context => (anchor, Placement::new(Side::Bottom, Align::Start), Px(0.0)),
        }
    }

    /// The entrance: `menu-in` for C's dropdown, `menu-pop` for the rest.
    fn entrance(self) -> Anim {
        match self {
            MenuKind::Dropdown => Anim::MenuIn,
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Anim::MenuPop,
        }
    }

    /// The item layout.
    pub(crate) fn row(self) -> Row {
        match self {
            MenuKind::Dropdown => Row::Checked,
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Row::Tiled,
        }
    }
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

/// What a key does to a floating menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MenuKey {
    /// Move the selection.
    Move(Step),
    /// Close, then pick the selection.
    Pick,
    /// Close the menu.
    Close,
    /// Add to the query.
    Type(String),
    /// Take the query's last character off.
    Erase,
}

/// The menu's reading of `key` (design/06-INTERACTIONS.md section 2.4); `None` for a key the
/// menu leaves alone. Typing only counts with [`Filter::Typing`] and no Ctrl, Alt or Super.
pub(crate) fn menu_key(key: &Key, modifiers: Modifiers, filter: Filter) -> Option<MenuKey> {
    let chord = modifiers.intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
    match key {
        Key::ArrowDown => Some(MenuKey::Move(Step::Down)),
        Key::ArrowUp => Some(MenuKey::Move(Step::Up)),
        Key::Enter | Key::Tab => Some(MenuKey::Pick),
        Key::Escape => Some(MenuKey::Close),
        Key::Backspace if filter == Filter::Typing => Some(MenuKey::Erase),
        Key::Character(text) if filter == Filter::Typing && !chord => {
            Some(MenuKey::Type(text.clone()))
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
/// matching items, best first, their headers and rules dropped (`S:2086-2088`).
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
        .filter_map(|entry| match entry {
            MenuEntry::Item { title, .. } => fuzzy(query, title).map(|hit| {
                (
                    hit.score,
                    Line {
                        entry,
                        marks: hit.marks,
                    },
                )
            }),
            MenuEntry::Header(_) | MenuEntry::Separator => None,
        })
        .collect();
    hits.sort_by(|a, b| b.0.total_cmp(&a.0));
    hits.into_iter().map(|(_, line)| line).collect()
}

/// The item values of `lines`, in order: what a selection index picks.
pub(crate) fn values<T: Clone>(lines: &[Line<'_, T>]) -> Vec<T> {
    lines
        .iter()
        .filter_map(|line| match line.entry {
            MenuEntry::Item { value, .. } => Some(value.clone()),
            MenuEntry::Header(_) | MenuEntry::Separator => None,
        })
        .collect()
}

/// Draw `lines`, the item at `selected` highlighted. `onpick(i)` and `onpoint(i)` name items by
/// their place among the items.
pub(crate) fn render_lines<T>(
    lines: &[Line<'_, T>],
    row: Row,
    selected: usize,
    onpick: EventHandler<usize>,
    onpoint: EventHandler<usize>,
) -> Element {
    let mut index = 0usize;
    let drawn: Vec<Element> = lines
        .iter()
        .map(|line| match line.entry {
            MenuEntry::Header(text) => rsx! {
                SectionHeader { kind: HeaderKind::Menu, text: text.clone() }
            },
            MenuEntry::Separator => rsx! {
                div { class: "ds-menu-separator", role: "separator" }
            },
            MenuEntry::Item {
                title,
                detail,
                tile,
                trail,
                check,
                ..
            } => {
                let at = index;
                index += 1;
                item(
                    ItemView {
                        title,
                        detail: detail.as_deref(),
                        tile: tile.as_ref(),
                        trail,
                        check: *check,
                        marks: &line.marks,
                        selection: if at == selected {
                            Selection::Selected
                        } else {
                            Selection::Unselected
                        },
                    },
                    row,
                    EventHandler::new(move |()| onpick.call(at)),
                    EventHandler::new(move |()| onpoint.call(at)),
                )
            }
        })
        .collect();
    rsx! {
        for (key , line) in drawn.into_iter().enumerate() {
            Fragment { key: "{key}", {line} }
        }
    }
}

/// A floating list of choices.
#[component]
pub fn Menu<T: Clone + PartialEq + 'static>(
    kind: MenuKind,
    anchor: Anchor,
    entries: Vec<MenuEntry<T>>,
    #[props(default)] filter: Filter,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
) -> Element {
    let float = use_float(ZLayer::Menu, Stacking::Layer(Dismiss::EscAndOutside));
    let presence = use_entrance(kind.entrance());
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);
    let typed = query();
    let shown = lines(&entries, &typed);
    let picks = values(&shown);
    let count = picks.len();
    let current = selected().min(count.saturating_sub(1));
    let label = entries.iter().find_map(|entry| match entry {
        MenuEntry::Header(text) => Some(text.clone()),
        MenuEntry::Item { .. } | MenuEntry::Separator => None,
    });
    let at = float
        .anchor_rect(&anchor)
        .map(|rect| kind.placement(rect))
        .map_or(Point::default(), |(rect, want, gap)| {
            float.origin(Some(rect), want, gap)
        });
    let pick = {
        let picks = picks.clone();
        move |index: usize| {
            if let Some(value) = picks.get(index) {
                onclose.call(());
                onpick.call(value.clone());
            }
        }
    };
    let onkey = {
        let pick = pick.clone();
        move |event: KeyboardEvent| {
            let Some(action) = menu_key(&event.key(), event.modifiers(), filter) else {
                return;
            };
            if action == MenuKey::Close {
                escape_closes(float, &event, onclose);
                return;
            }
            event.prevent_default();
            event.stop_propagation();
            match action {
                MenuKey::Move(step) => selected.set(moved(Nav::Wrap, current, count, step)),
                MenuKey::Pick => pick(current),
                MenuKey::Type(text) => {
                    query.with_mut(|query| query.push_str(&text));
                    selected.set(0);
                }
                MenuKey::Erase => {
                    query.with_mut(|query| {
                        query.pop();
                    });
                    selected.set(0);
                }
                MenuKey::Close => {}
            }
        }
    };
    let pointed = move |index: usize| {
        if kind.row() == Row::Tiled && *selected.peek() != index {
            selected.set(index);
        }
    };
    let body = if shown.is_empty() {
        rsx! {
            div { class: "ds-menu-empty", "Nothing matches \"{typed}\"." }
        }
    } else {
        render_lines(
            &shown,
            kind.row(),
            current,
            EventHandler::new(pick),
            EventHandler::new(pointed),
        )
    };
    let probe = float.surface();
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-menu",
                "data-elevation": "pop",
                "data-layer": "menu",
                "data-kind": kind.slug(),
                role: "listbox",
                "aria-label": label,
                "data-presence": presence.slug(),
                tabindex: "-1",
                style: position_style(at),
                onmounted: move |event| {
                    let element = event.data();
                    probe.on_mounted(event);
                    spawn(async move {
                        let _ = element.set_focus(true).await;
                    });
                },
                onkeydown: onkey,
                {body}
            }
        },
        onclose,
    );
    rsx! {}
}

#[cfg(test)]
mod tests {
    use super::{Filter, MenuKey, Nav, Step, menu_key, moved};
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
    fn menu_keys_follow_section_2_4() {
        let none = Modifiers::empty();
        let a = Key::Character("a".to_string());
        #[rustfmt::skip]
        let cases: Vec<(Key, Modifiers, Filter, Option<MenuKey>)> = vec![
            (Key::ArrowDown, none, Filter::None, Some(MenuKey::Move(Step::Down))),
            (Key::ArrowUp, none, Filter::None, Some(MenuKey::Move(Step::Up))),
            (Key::Enter, none, Filter::None, Some(MenuKey::Pick)),
            (Key::Tab, none, Filter::None, Some(MenuKey::Pick)),
            (Key::Escape, none, Filter::None, Some(MenuKey::Close)),
            (a.clone(), none, Filter::Typing, Some(MenuKey::Type("a".to_string()))),
            (a.clone(), Modifiers::SHIFT, Filter::Typing, Some(MenuKey::Type("a".to_string()))),
            (a.clone(), Modifiers::CONTROL, Filter::Typing, None),
            (a, none, Filter::None, None),
            (Key::Backspace, none, Filter::Typing, Some(MenuKey::Erase)),
            (Key::Backspace, none, Filter::None, None),
            (Key::ArrowLeft, none, Filter::Typing, None),
        ];
        for (key, modifiers, filter, want) in cases {
            assert_eq!(
                menu_key(&key, modifiers, filter),
                want,
                "{key:?} {modifiers:?}"
            );
        }
    }
}
