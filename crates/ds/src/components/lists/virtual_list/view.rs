//! VirtualList: a list of thousands of rows, of one height or one per key, that mounts only the
//! rows near the viewport. Its scroll state is a `ScrollerRef`: the rows to mount come from `Scroll::rows`
//! over it, a cursor the owner moves is kept in view by revealing its row, and the end of the list
//! coming near is reported for paging. A row the owner removes while it is mounted plays the
//! list's exit where it stood and the rows below heal, as in `List`: the mounted rows are a
//! `use_roster` roster (`Roster::leave_from_render` for a removal, the window's keys as its list),
//! so a row that only scrolled out of the window is unmounted at once and nothing heals for it.

use crate::components::controls::scroller::handle::{ScrollerRef, use_scroller};
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::list::entry::SetPlace;
use crate::components::lists::list::keys::{ListKey, list_key};
use crate::components::lists::list::list::node_key;
use crate::components::lists::virtual_list::layout::Layout;
use crate::components::lists::virtual_list::model::{Change, RowHeight, departed, roved};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::scroll::{Scroll, ScrollSpan};
use ds_core::geometry::units::Px;
use ds_core::word::Word;
use ds_motion::presence::{Exit, Presence};
use ds_motion::roster::{RosterEntry, RowPitch};
use ds_motion::use_roster::{LeaveBy, RosterSpec, use_roster_from};
use std::hash::Hash;

/// How many rows around the viewport are mounted besides the visible ones.
const OVERSCAN_ROWS: usize = 4;
/// How many rows from the end `near_end` starts to be reported.
const PAGE_ROWS: usize = 10;

/// Whether the end of the list is near.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Near {
    Yes,
    No,
}

/// Whether the scroller has been measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Measured {
    Yes,
    No,
}

/// A row that left while mounted: what it drew and how tall it was, for as long as it plays out.
struct Exiting<K> {
    key: K,
    content: Element,
    height: Px,
}

/// What the list remembers between renders.
struct Memory<K> {
    /// The keys as of the last render.
    keys: Vec<K>,
    /// The cursor's index in `keys`.
    cursor: Option<usize>,
    /// The index in `keys` of the first mounted row.
    start: usize,
    /// The mounted rows, in order, as they were drawn.
    shown: Vec<(K, Element)>,
    /// Rows that left while mounted and are still playing their exit.
    exiting: Vec<Exiting<K>>,
    /// Where the rows lay as of the last render.
    layout: Layout,
}

/// `keys`, each drawn by `row` only while it is within the viewport (and `overscan` rows around
/// it). `height` is every row's height, one for all or one per key. `cursor` is the key the owner's keys rest on: it is kept
/// in view, and the arrow keys, Home and End ask `onselect` to move it, Enter and Space ask
/// `onpick`. `near_end` is called when fewer than `page_rows` rows lie below the viewport, and
/// again when the list grows and it is still so. A row's content must fit its height.
///
/// A key the owner stops listing while its row is mounted plays `exit` where it stood and the
/// rows below heal, unless `change` says the keys were replaced: then the row is gone at once. A
/// key listed again before its exit settles is an ordinary row again.
#[component]
pub fn VirtualList<K: Clone + Eq + Hash + 'static>(
    label: String,
    keys: Vec<K>,
    row: Callback<K, Element>,
    height: RowHeight<K>,
    #[props(default = OVERSCAN_ROWS)] overscan: usize,
    #[props(default)] cursor: Option<K>,
    #[props(default)] onselect: Option<EventHandler<K>>,
    #[props(default)] onpick: Option<EventHandler<K>>,
    #[props(default)] near_end: Option<EventHandler<()>>,
    #[props(default = PAGE_ROWS)] page_rows: usize,
    #[props(default = Exit::Row)] exit: Exit,
    #[props(default)] change: Change,
    #[props(default)] common: Common,
) -> Element {
    let len = keys.len();
    let scroller = use_scroller();
    let memory = use_hook(|| {
        CopyValue::new(Memory {
            keys: keys.clone(),
            cursor: None,
            start: 0,
            shown: Vec::new(),
            exiting: Vec::new(),
            layout: Layout::Even {
                pitch: Px(0.0),
                len: 0,
            },
        })
    });
    let layout = height.layout(&keys, &memory.peek().layout);
    let window = use_memo(use_reactive(
        (&layout, &overscan),
        move |(layout, overscan)| layout.window(*scroller.scroll().read(), overscan),
    ));
    use_paging(scroller, &layout, page_rows, near_end);
    let end = window().end.min(len);
    let start = window().start.min(end);
    let roster = use_roster_from(
        &keys[start..end],
        RosterSpec {
            leave: LeaveBy::Action,
            exit,
            pitch: RowPitch(Px(0.0)),
            on_settled: None,
        },
    );
    let mut mem = memory;
    let mut state = mem.write();
    if state.keys != keys {
        let before = state.start..state.start + state.shown.len();
        let gone = match change {
            Change::Edit => departed(&state.keys, before, &keys, &state.layout),
            Change::Replace => Vec::new(),
        };
        for gone in &gone {
            let content = state
                .shown
                .iter()
                .find(|(key, _)| *key == gone.key)
                .map_or_else(|| rsx! {}, |(_, content)| content.clone());
            state.exiting.push(Exiting {
                key: gone.key.clone(),
                content,
                height: gone.height,
            });
            roster
                .pitches()
                .set(gone.key.clone(), RowPitch(gone.height));
        }
        roster.leave_from_render(gone.into_iter().map(|gone| gone.key).collect());
        state.keys = keys.clone();
    }
    roster.list_from_render(keys[start..end].to_vec());
    let entries = roster.entries();
    state.exiting.retain(|exiting| {
        entries
            .iter()
            .any(|entry| entry.key == exiting.key && matches!(entry.presence, Presence::Leaving(_)))
    });
    state.layout = layout.clone();
    state.cursor = cursor
        .as_ref()
        .and_then(|cursor| keys.iter().position(|key| key == cursor));
    let cursor_at = state.cursor;
    let items = items(&mut state, &entries, start, len, &layout, row);
    drop(state);
    use_cursor_reveal(scroller, cursor_at.map(|at| layout.span(at)));
    let onkeydown = move |event: KeyboardEvent| {
        let Some(act) = list_key(&event.key(), event.modifiers()) else {
            return;
        };
        let held = memory.peek();
        let at = held.cursor;
        let keys = &held.keys;
        match act {
            ListKey::Move(rove) => {
                if let (Some(to), Some(onselect)) = (roved(at, keys.len(), rove), onselect) {
                    event.prevent_default();
                    onselect.call(keys[to].clone());
                }
            }
            ListKey::Pick => {
                if let (Some(at), Some(onpick)) = (at, onpick) {
                    event.prevent_default();
                    onpick.call(keys[at].clone());
                }
            }
            ListKey::Jump(_) => {}
        }
    };
    let data = common.data_attributes();
    let (above, below) = (layout.top(start).0, layout.below(end).0);
    rsx! {
        div {
            class: common.class("ds-virtual-list"),
            id: common.id.clone(),
            role: "listbox",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            onkeydown,
            ..data,
            Scroller { scroller,
                div { class: "ds-virtual-spacer", style: "height:{above}px" }
                for item in items {
                    div {
                        key: "{item.node}",
                        class: "ds-list-item",
                        role: "none",
                        "aria-posinset": item.place.map(SetPlace::posinset),
                        "aria-setsize": item.place.map(SetPlace::setsize),
                        "data-presence": item.presence,
                        "data-exit": item.exit,
                        style: item.style,
                        {item.content}
                    }
                }
                div { class: "ds-virtual-spacer", style: "height:{below}px" }
            }
        }
    }
}

/// One row to draw: its node, its motion state, its inline style and its content.
struct Item {
    node: String,
    /// Where it stands among the listed keys; a leaving row is no longer one of them.
    place: Option<SetPlace>,
    presence: Option<&'static str>,
    exit: Option<&'static str>,
    style: String,
    content: Element,
}

/// A row's inline style: its height, and the distance it heals from.
fn row_style(height: Px, heal: Option<Px>) -> String {
    match heal {
        Some(dy) => format!("height:{}px;--dy:{}px", height.0, dy.0),
        None => format!("height:{}px", height.0),
    }
}

/// The roster's rows in order, each leaving row where it stood, and what was drawn recorded for
/// the next render. `start` is the index among the `len` keys of the first row that is not leaving.
fn items<K: Clone + Eq + Hash + 'static>(
    state: &mut Memory<K>,
    entries: &[RosterEntry<K>],
    start: usize,
    len: usize,
    layout: &Layout,
    row: Callback<K, Element>,
) -> Vec<Item> {
    let mut items = Vec::with_capacity(entries.len());
    let mut shown = Vec::with_capacity(entries.len());
    let mut at = start;
    for entry in entries {
        if let Presence::Leaving(exit) = entry.presence {
            if let Some(gone) = state.exiting.iter().find(|gone| gone.key == entry.key) {
                items.push(Item {
                    node: node_key(&entry.key),
                    place: None,
                    presence: Some("leaving"),
                    exit: Some(exit.slug()),
                    style: row_style(gone.height, None),
                    content: gone.content.clone(),
                });
            }
            continue;
        }
        let content = row.call(entry.key.clone());
        let heal = entry.heal.map(|heal| heal.dy);
        items.push(Item {
            node: node_key(&entry.key),
            place: Some(SetPlace { index: at, len }),
            presence: heal.map(|_| "healing"),
            exit: None,
            style: row_style(layout.height(at), heal),
            content: content.clone(),
        });
        shown.push((entry.key.clone(), content));
        at += 1;
    }
    state.start = start;
    state.shown = shown;
    items
}

/// Scroll the cursor's row, at `span`, into view when it moves, and once the scroller has been
/// measured.
pub(crate) fn use_cursor_reveal(scroller: ScrollerRef, span: Option<ScrollSpan>) {
    let measured = use_memo(move || match scroller.scroll().read().viewport.0 > 0.0 {
        true => Measured::Yes,
        false => Measured::No,
    });
    use_effect(use_reactive((&span,), move |(span,)| {
        let _ = measured();
        if let Some(span) = span {
            scroller.reveal(span);
        }
    }));
}

/// Report the end of the list coming near, and again when the list grows and it still is.
fn use_paging(
    scroller: ScrollerRef,
    layout: &Layout,
    page_rows: usize,
    near_end: Option<EventHandler<()>>,
) {
    let near = use_memo(use_reactive(
        (layout, &page_rows),
        move |(layout, page_rows)| {
            let seen = *scroller.scroll().read();
            let whole = Scroll {
                content: layout.total(),
                ..seen
            };
            match whole.near_end(layout.page(page_rows)) {
                true => Near::Yes,
                false => Near::No,
            }
        },
    ));
    // The newest handler, without its being a reason to report again.
    let mut handler = use_hook(|| CopyValue::new(near_end));
    handler.set(near_end);
    use_effect(use_reactive((&layout.len(),), move |(_,)| {
        if let (Near::Yes, Some(near_end)) = (near(), *handler.peek()) {
            near_end.call(());
        }
    }));
}
