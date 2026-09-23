//! CommandPalette: "the same menu, just bigger and centred" (design/04-COMPONENTS.md section 25).
//!
//! A SearchField over an embedded Rich menu, on a scrim that closes on a pointer down outside
//! the panel. Keys come from the search field (design/06-INTERACTIONS.md section 2.3): Up and
//! Down move the selection CLAMPED, Enter closes and then runs the selection, Escape closes the
//! topmost layer only. Ranking and grouping are the consumer's (section 11); the palette marks
//! the query in each title with the same fuzzy matcher.

use crate::components::menu::{Line, MenuKind, Nav, Step, moved, render_lines, values};
use crate::components::menu_entry::{MenuEntry, fuzzy};
use crate::components::popover::{Dismiss, Stacking, use_entrance, use_float};
use crate::components::search_field::SearchField;
use crate::components::text_input::Focus;
use crate::motion::anim::Anim;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// What a key in the search field does to the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaletteKey {
    /// Move the selection, clamped.
    Move(Step),
    /// Close, then run the selection.
    Run,
    /// Close.
    Close,
}

/// The palette's reading of a key (design/06-INTERACTIONS.md section 2.3).
fn palette_key(key: &Key) -> Option<PaletteKey> {
    match key {
        Key::ArrowDown => Some(PaletteKey::Move(Step::Down)),
        Key::ArrowUp => Some(PaletteKey::Move(Step::Up)),
        Key::Enter => Some(PaletteKey::Run),
        Key::Escape => Some(PaletteKey::Close),
        _ => None,
    }
}

/// The groups as one list: each non-empty group's title as a header, then its items.
fn headed<T: Clone>(groups: &[(String, Vec<MenuEntry<T>>)]) -> Vec<MenuEntry<T>> {
    groups
        .iter()
        .filter(|(_, entries)| !entries.is_empty())
        .flat_map(|(title, entries)| {
            std::iter::once(MenuEntry::Header(title.clone())).chain(entries.iter().cloned())
        })
        .collect()
}

/// `entries` as menu lines, each item's title marked where `query` matches it.
fn marked<'a, T>(entries: &'a [MenuEntry<T>], query: &str) -> Vec<Line<'a, T>> {
    entries
        .iter()
        .map(|entry| Line {
            entry,
            marks: match entry {
                MenuEntry::Item { title, .. } => {
                    fuzzy(query, title).map(|hit| hit.marks).unwrap_or_default()
                }
                MenuEntry::Header(_) | MenuEntry::Separator => Vec::new(),
            },
        })
        .collect()
}

/// Search and commands over a scrim.
#[component]
pub fn CommandPalette<T: Clone + PartialEq + 'static>(
    label: String,
    placeholder: String,
    query: String,
    tokens: Vec<String>,
    groups: Vec<(String, Vec<MenuEntry<T>>)>,
    empty: String,
    oninput: EventHandler<String>,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
) -> Element {
    let float = use_float(ZLayer::Palette, Stacking::Layer(Dismiss::EscOnly));
    let presence = use_entrance(Anim::PeekIn);
    // The selection belongs to the query it was made under: every keystroke puts it back on
    // the first item (`S:1651`).
    let mut selected = use_signal(|| (query.clone(), 0usize));
    let entries = headed(&groups);
    let shown = marked(&entries, &query);
    let picks = values(&shown);
    let count = picks.len();
    let current = match &*selected.read() {
        (made, index) if *made == query => (*index).min(count.saturating_sub(1)),
        _ => 0,
    };
    let run = {
        let picks = picks.clone();
        move |index: usize| {
            if let Some(value) = picks.get(index) {
                onclose.call(());
                onpick.call(value.clone());
            }
        }
    };
    let onkey = {
        let run = run.clone();
        let event_query = query.clone();
        move |event: KeyboardData| match palette_key(&event.key()) {
            Some(PaletteKey::Move(step)) => {
                selected.set((event_query.clone(), moved(Nav::Clamp, current, count, step)));
            }
            Some(PaletteKey::Run) => run(current),
            Some(PaletteKey::Close) if float.takes_escape() => onclose.call(()),
            Some(PaletteKey::Close) | None => {}
        }
    };
    let body = if count == 0 {
        rsx! {
            div { class: "ds-menu-empty", "{empty}" }
        }
    } else {
        render_lines(
            &shown,
            MenuKind::Rich.row(),
            current,
            EventHandler::new(run),
            EventHandler::new({
                let query = query.clone();
                move |index: usize| {
                    if selected.peek().1 != index {
                        selected.set((query.clone(), index));
                    }
                }
            }),
        )
    };
    float.show(
        rsx! {
            div {
                class: "ds-palette-wrap",
                onpointerdown: move |_| {
                    if float.is_top() {
                        onclose.call(());
                    }
                },
                div {
                    class: "ds-palette",
                    role: "dialog",
                    "aria-label": "{label}",
                    "data-presence": presence.slug(),
                    onpointerdown: move |event| event.stop_propagation(),
                    SearchField {
                        label: label.clone(),
                        value: query.clone(),
                        placeholder,
                        tokens,
                        oninput,
                        onkey,
                        focus: Focus::OnMount,
                    }
                    div {
                        class: "ds-menu",
                        "data-kind": "rich",
                        "data-embed": "palette",
                        role: "listbox",
                        onmousedown: move |event| event.prevent_default(),
                        {body}
                    }
                }
            }
        },
        onclose,
    );
    rsx! {}
}

#[cfg(test)]
mod tests {
    use super::{PaletteKey, palette_key};
    use crate::components::menu::Step;
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
}
