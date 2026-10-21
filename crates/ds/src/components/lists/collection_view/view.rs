//! CollectionView: Finder's icon view, a grid of tiles that never builds them all. A tile is a
//! thumbnail over a caption; the caller lists `items` and the view draws the rows near the
//! viewport, as `VirtualTable` does for its rows: the rows between are two spacers' height inside
//! a `Scroller`, so a folder of a hundred thousand files costs a window of tiles.
//!
//! The grid is as many tiles wide as the view's measured width holds at the size's tile width
//! ([`Flow::Fit`]), or a fixed number ([`Flow::Columns`]). A tile's identity is its `key`; the
//! cursor, the selection and every report are tile indexes, as `VirtualTable`'s are row indexes.
//! The cursor is the tile the keys rest on; the owner moves it (a controlled component) and it is
//! kept in view. Which tiles are selected is the cursor's alone unless the owner gives a
//! `selection`; the owner who wants several listens to `onreach`, hears a `Pick` and folds it
//! into its `RowSelection` with `RowSelection::picked`: a click replaces, Shift extends, the
//! primary modifier toggles, and Shift with an arrow extends. Select all is the keymap's.
//!
//! Keys: Left and Right walk the tiles in reading order and wrap across rows, Up and Down move a
//! row in the same column (into a short last row, to its last tile), Home and End go to the ends,
//! letters jump to the next caption that starts with them, and Command-Down (the primary modifier
//! with Down) asks `onopen`, as does a double-click. Return and Space are left to the app's keymap
//! (Finder's Rename and Quick Look), and so is any key with a modifier besides Shift. Each move asks `onselect` (or `onreach`) for another index and stops at the
//! edges.
//!
//! A tile the owner stops listing while it is mounted plays a fade where it stood
//! (`ds_motion::presence::Exit::Fade`) and one it adds fades in, through the same roster a
//! `VirtualList` uses. The tiles after a removed one close the gap when its exit settles; they do
//! not slide, as a grid has no single axis to heal along. `Change::Replace` (a new folder) plays
//! no exit.
//!
//! A caption is one line, cut from the middle to a character budget (`CollectionSize::caption_chars`):
//! Finder wraps to two lines, which Blitz cannot clamp.
//!
//! Markup: `div.ds-cv[role=listbox]` (`--cv-thumb`, `--cv-tile` and `--cv-pitch` inline) of a
//! `Scroller` of a spacer, `div.ds-cv-grid` of `div.ds-cv-item[role=option]` (`aria-selected`,
//! `data-index`, `data-presence`, `data-exit` while leaving, `data-editing` while a field holds
//! the caption) of `span.ds-cv-thumb` and `span.ds-cv-caption`, and a spacer.
//! `data-focus=away` marks a view that does not hold the keyboard.

use crate::components::controls::scroller::handle::use_scroller;
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::collection_view::act::Act;
use crate::components::lists::collection_view::item::CollectionItem;
use crate::components::lists::collection_view::model::{
    CollectionKey, collection_key, target, tiles_in,
};
use crate::components::lists::collection_view::size::{CollectionSize, Flow};
use crate::components::lists::collection_view::window::{Memory, Shown, Stand, reconcile, shown};
use crate::components::lists::reach::Reach;
use crate::components::lists::virtual_list::layout::Layout;
use crate::components::lists::virtual_list::model::Change;
use crate::components::lists::virtual_list::view::use_cursor_reveal;
use crate::components::lists::virtual_table::model::key_how;
use crate::host::measure::use_rect;
use crate::keys::use_platform;
use crate::root::common::Common;
use crate::stack::typeahead::Typeahead;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::selection::{Pick, PickHow, RowSelection};
use ds_core::time::clock::now;
use ds_core::vocab::Selection;
use ds_core::word::Word;
use ds_motion::presence::Exit;
use ds_motion::roster::RowPitch;
use ds_motion::use_roster::{LeaveBy, RosterSpec, use_roster_from};
use std::hash::Hash;

/// How many rows around the viewport are mounted besides the visible ones.
const OVERSCAN_ROWS: usize = 2;

/// Whether the view holds the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Yes,
    No,
}

/// `items` as a grid of tiles at `size`. `label` names the grid for a screen reader. `cursor` is
/// the tile the keys rest on, kept in view; `onselect` hears a click or a key asking for another,
/// `onreach` hears the same with how it was reached (in place of `onselect`), `selection` says
/// which tiles are selected (the cursor's alone otherwise), and `onopen` hears Command-Down or a
/// double-click. `overscan` is how many rows are mounted beyond the visible ones on each side
/// (2 by default). `change` says whether a change of `items` is an edit (removed tiles fade out)
/// or a replacement (they are gone at once).
///
/// `items` is read on every render and its captions are cloned for type-to-select only when
/// letters are typed; the keys are collected on every render, so an owner of a million tiles keeps
/// its `items` in a memo.
#[component]
pub fn CollectionView<K: Clone + Eq + Hash + 'static>(
    label: String,
    items: Vec<CollectionItem<K>>,
    #[props(default)] size: CollectionSize,
    #[props(default)] flow: Flow,
    #[props(default)] cursor: Option<usize>,
    #[props(default)] selection: Option<RowSelection>,
    #[props(default)] onselect: Option<EventHandler<usize>>,
    #[props(default)] onreach: Option<EventHandler<Pick>>,
    #[props(default)] onopen: Option<EventHandler<usize>>,
    #[props(default = OVERSCAN_ROWS)] overscan: usize,
    #[props(default)] change: Change,
    #[props(default)] common: Common,
) -> Element {
    let len = items.len();
    let probe = use_rect();
    let columns = flow.columns(probe.rect().map(|rect| rect.size.width), size.tile());
    let scroller = use_scroller();
    let layout = Layout::Even {
        pitch: size.pitch(),
        len: len.div_ceil(columns),
    };
    let rows = use_memo(use_reactive(
        (&layout, &overscan),
        move |(layout, overscan)| layout.window(*scroller.scroll().read(), overscan),
    ));
    let window = tiles_in(rows(), columns, len);
    let keys: Vec<K> = items.iter().map(|item| item.key.clone()).collect();
    let roster = use_roster_from(
        &keys[window.clone()],
        RosterSpec {
            leave: LeaveBy::Action,
            exit: Exit::Fade,
            pitch: RowPitch(Px(0.0)),
            on_settled: None,
        },
    );
    let mut memory = use_hook(|| CopyValue::new(Memory::new(keys.clone())));
    let drawn = {
        let mut state = memory.write();
        reconcile(&mut state, &keys, change, &layout, roster);
        roster.list_from_render(keys[window.clone()].to_vec());
        shown(&mut state, &roster.entries(), &items, window, size)
    };
    use_cursor_reveal(
        scroller,
        cursor
            .filter(|at| *at < len)
            .map(|at| layout.span(at / columns)),
    );
    let mut held = use_signal(|| Held::No);
    let typeahead = use_hook(|| CopyValue::new(Typeahead::default()));
    let reach = Reach {
        platform: use_platform(),
        onreach,
        onselect,
        onopen,
    };
    let selected = |at: usize| {
        let on = match &selection {
            Some(chosen) => chosen.contains(at),
            None => cursor == Some(at),
        };
        match on {
            true => Selection::Selected,
            false => Selection::Unselected,
        }
    };
    let tiles: Vec<Tile> = drawn
        .into_iter()
        .map(|tile| Tile::of(tile, &items, &selected, reach))
        .collect();
    let onkeydown = move |event: KeyboardEvent| {
        let Some(act) = collection_key(&event.key(), event.modifiers(), reach.platform) else {
            return;
        };
        match act {
            CollectionKey::Open => {
                if let (Some(at), Some(_)) = (cursor.filter(|at| *at < len), onopen) {
                    event.prevent_default();
                    event.stop_propagation();
                    reach.opened(at);
                }
            }
            CollectionKey::Jump(text) => {
                let labels: Vec<&str> = items.iter().map(|item| item.caption.as_str()).collect();
                let from = cursor.unwrap_or_else(|| labels.len().saturating_sub(1));
                let mut buffer = typeahead;
                let (next, found) = buffer.peek().clone().typed(&text, now(), &labels, from);
                buffer.set(next);
                if let Some(row) = found {
                    reach.tell(Pick {
                        row,
                        how: PickHow::Replace,
                    });
                }
            }
            CollectionKey::Arrow(_) | CollectionKey::Edge(_) => {
                if let (Some(row), true) = (target(cursor, len, &act, columns), reach.heard()) {
                    event.prevent_default();
                    reach.tell(Pick {
                        row,
                        how: key_how(event.modifiers()),
                    });
                }
            }
        }
    };
    let data = common.data_attributes();
    let mounted = common.clone();
    let away = (held() == Held::No).then_some("away");
    let (above, below) = (layout.top(rows().start).0, layout.below(rows().end).0);
    let grid = format!("grid-template-columns:repeat({columns},1fr)");
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-cv"),
            role: "listbox",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-size": size.slug(),
            "data-focus": away,
            style: size.style(),
            onmounted: move |event| {
                probe.on_mounted(event.clone());
                mounted.mounted(event);
            },
            onfocus: move |_| held.set(Held::Yes),
            onblur: move |_| held.set(Held::No),
            onkeydown,
            ..data,
            Scroller { scroller,
                div { class: "ds-cv-spacer", style: "height:{above}px" }
                div { class: "ds-cv-grid", style: grid,
                    for tile in tiles {
                        div {
                            key: "{tile.node}",
                            class: "ds-cv-item",
                            role: "option",
                            "aria-label": tile.label,
                            "aria-selected": tile.selection.aria(),
                            "aria-posinset": tile.place.map(|at| (at + 1).to_string()),
                            "aria-setsize": tile.place.map(|_| len.to_string()),
                            "aria-hidden": tile.leaving,
                            "data-index": tile.place.map(|at| at.to_string()),
                            "data-presence": tile.presence,
                            "data-exit": tile.exit,
                            "data-editing": tile.editing,
                            onclick: move |event| {
                                if let Some(act) = tile.act {
                                    act.click(&event);
                                }
                            },
                            ondoubleclick: move |_| {
                                if let Some(act) = tile.act {
                                    act.open();
                                }
                            },
                            {tile.content}
                        }
                    }
                }
                div { class: "ds-cv-spacer", style: "height:{below}px" }
            }
        }
    }
}

/// One tile as the markup writes it.
struct Tile {
    node: String,
    place: Option<usize>,
    label: Option<String>,
    selection: Selection,
    presence: &'static str,
    exit: Option<&'static str>,
    leaving: Option<&'static str>,
    editing: Option<&'static str>,
    act: Option<Act>,
    content: Element,
}

impl Tile {
    /// `drawn` with its row's selection and what a press on it does: a listed tile answers
    /// presses unless a field holds its caption; a leaving one answers none.
    fn of<K>(
        drawn: Shown,
        items: &[CollectionItem<K>],
        selected: &impl Fn(usize) -> Selection,
        reach: Reach,
    ) -> Tile {
        let Shown {
            node,
            stand,
            presence,
            content,
        } = drawn;
        let (place, exit) = match stand {
            Stand::Live(at) => (Some(at), None),
            Stand::Leaving(exit) => (None, Some(exit)),
        };
        let item = place.map(|at| &items[at]);
        let editing = item.is_some_and(|item| item.edit.is_some());
        Tile {
            node,
            place,
            label: item.map(|item| item.caption.clone()),
            selection: place.map_or(Selection::Unselected, selected),
            presence,
            exit,
            leaving: exit.map(|_| "true"),
            editing: editing.then_some("true"),
            act: place.filter(|_| !editing).map(|at| Act { at, reach }),
            content,
        }
    }
}
