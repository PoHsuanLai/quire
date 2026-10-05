//! VirtualList: a list of thousands of rows, of one height or one per key, that mounts only the
//! rows near the viewport. Its scroll state is a `ScrollerRef`: the rows to mount come from `Scroll::rows`
//! over it, a cursor the owner moves is kept in view by revealing its row, and the end of the list
//! coming near is reported for paging. A row the owner removes while it is mounted plays the
//! list's exit where it stood and the rows below heal, as in `List`; a row that only scrolled out
//! of the window is unmounted at once.

use crate::components::controls::scroller::handle::{ScrollerRef, use_scroller};
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::list::entry::SetPlace;
use crate::components::lists::list::keys::{ListKey, list_key};
use crate::components::lists::list::list::node_key;
use crate::components::lists::virtual_list::layout::Layout;
use crate::components::lists::virtual_list::model::{
    Change, Leaving, RowHeight, exit_anim, heal_dy, leaving, positions, roved, slot_in,
};
use crate::root::common::Common;
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::geometry::scroll::{Scroll, ScrollSpan};
use ds_core::geometry::units::Px;
use ds_core::time::clock::sleep;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::Exit;
use ds_motion::settle::settle;
use ds_style::scope::{Scope, use_scope_signal};
use ds_style::task::{spawn_in, try_get};
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

/// The exits that started together: they settle and drop together.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Batch(u32);

/// A row that left while mounted: where it stood, what it drew and the batch it plays out in.
struct Exiting<K> {
    gone: Leaving<K>,
    content: Element,
    batch: Batch,
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
    leaving: Vec<Exiting<K>>,
    /// The batch the next exits belong to.
    next_batch: Batch,
    /// The slots and heights of rows just dropped, whose rows below are healing.
    healing: Vec<(usize, Px)>,
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
    let tick = use_signal(|| 0u32);
    let _ = tick();
    let memory = use_hook(|| {
        CopyValue::new(Memory {
            keys: keys.clone(),
            cursor: None,
            start: 0,
            shown: Vec::new(),
            leaving: Vec::new(),
            next_batch: Batch(0),
            healing: Vec::new(),
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
    let scope = use_scope_signal();
    let task_scope = use_hook(dioxus::core::current_scope_id);
    let mut mem = memory;
    let mut state = mem.write();
    if state.keys != keys {
        let state = &mut *state;
        let index = positions(&keys);
        // A key listed again stops leaving: it is an ordinary row once more, and its batch no
        // longer drops it or heals below it. The rows still leaving keep their place among the
        // new keys.
        state
            .leaving
            .retain(|exiting| !index.contains_key(&exiting.gone.key));
        for exiting in &mut state.leaving {
            exiting.gone.slot = slot_in(&state.keys, exiting.gone.slot, &index, keys.len());
        }
        let before = state.start..state.start + state.shown.len();
        let gone = match change {
            Change::Edit => leaving(&state.keys, before, &keys, &state.layout),
            Change::Replace => Vec::new(),
        };
        let batch = state.next_batch;
        state.next_batch = Batch(batch.0 + 1);
        let started = !gone.is_empty();
        for gone in gone {
            let content = state
                .shown
                .iter()
                .find(|(key, _)| *key == gone.key)
                .map_or_else(|| rsx! {}, |(_, content)| content.clone());
            state.leaving.push(Exiting {
                gone,
                content,
                batch,
            });
        }
        state.keys = keys.clone();
        if started {
            queue_effect(move || drop_after_exit(memory, tick, scope, task_scope, batch, exit));
        }
    }
    state.layout = layout.clone();
    state.cursor = cursor
        .as_ref()
        .and_then(|cursor| keys.iter().position(|key| key == cursor));
    let cursor_at = state.cursor;
    let end = window().end.min(len);
    let start = window().start.min(end);
    let items = items(&mut state, &keys, start..end, &layout, row, exit);
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

/// The rows of `window` in order, each leaving row before the row it stood above, and what was
/// drawn recorded for the next render.
fn items<K: Clone + Eq + Hash + 'static>(
    state: &mut Memory<K>,
    keys: &[K],
    window: std::ops::Range<usize>,
    layout: &Layout,
    row: Callback<K, Element>,
    exit: Exit,
) -> Vec<Item> {
    let dropped = state.healing.clone();
    let mut items = Vec::new();
    let mut shown = Vec::with_capacity(window.len());
    for at in window.start..=window.end {
        for exiting in state
            .leaving
            .iter()
            .filter(|exiting| exiting.gone.slot == at)
        {
            items.push(Item {
                node: node_key(&exiting.gone.key),
                place: None,
                presence: Some("leaving"),
                exit: Some(exit.slug()),
                style: row_style(exiting.gone.height, None),
                content: exiting.content.clone(),
            });
        }
        let Some(key) = keys.get(at).filter(|_| at < window.end) else {
            continue;
        };
        let content = row.call(key.clone());
        let heal = heal_dy(&dropped, at);
        items.push(Item {
            node: node_key(key),
            place: Some(SetPlace {
                index: at,
                len: keys.len(),
            }),
            presence: heal.map(|_| "healing"),
            exit: None,
            style: row_style(layout.height(at), heal),
            content: content.clone(),
        });
        shown.push((key.clone(), content));
    }
    state.start = window.start;
    state.shown = shown;
    items
}

/// Drop the rows of `batch` once their exit has played, heal the rows below, and redraw.
fn drop_after_exit<K: Clone + Eq + 'static>(
    memory: CopyValue<Memory<K>>,
    mut tick: Signal<u32>,
    scope: Signal<Scope>,
    task_scope: ScopeId,
    batch: Batch,
    exit: Exit,
) {
    spawn_in(task_scope, async move {
        let Ok(motion) = try_get(scope).map(|scope| scope.resolved.motion) else {
            return;
        };
        sleep(settle(exit_anim(exit), motion)).await;
        let mut memory = memory;
        let Ok(mut state) = memory.try_write() else {
            return;
        };
        let dropped: Vec<(usize, Px)> = state
            .leaving
            .iter()
            .filter(|exiting| exiting.batch == batch)
            .map(|exiting| (exiting.gone.slot, exiting.gone.height))
            .collect();
        state.leaving.retain(|exiting| exiting.batch != batch);
        state.healing = dropped;
        drop(state);
        tick += 1;
        sleep(settle(Anim::Heal, motion)).await;
        if let Ok(mut state) = memory.try_write() {
            state.healing.clear();
        }
        tick += 1;
    });
}

/// Scroll the cursor's row, at `span`, into view when it moves, and once the scroller has been
/// measured.
fn use_cursor_reveal(scroller: ScrollerRef, span: Option<ScrollSpan>) {
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
