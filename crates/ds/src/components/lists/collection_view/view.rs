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
//! the caption, `data-drop` and `data-drag` as a drag puts them) of `span.ds-cv-thumb`,
//! `span.ds-cv-caption` and, when the item has one, `span.ds-cv-badge`, and a spacer.
//! `data-focus=away` marks a view that does not hold the keyboard; `data-aspect=landscape` and
//! `data-extent=content` mark the non-default shapes (`--cv-thumb-h` is then the thumbnail's height).
//! While a rubber band is dragged a `div.ds-marquee-layer` holding a `div.ds-marquee` is the
//! scroller's first child, placed in the content's coordinates.

use crate::components::controls::scroller::handle::use_scroller;
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::collection_view::act::Act;
use crate::components::lists::collection_view::item::CollectionItem;
use crate::components::lists::collection_view::landing::{Drops, Landing};
use crate::components::lists::collection_view::model::{
    CollectionKey, collection_key, target, tiles_in,
};
use crate::components::lists::collection_view::size::{
    CollectionExtent, CollectionSize, Flow, GRID_INSET, ThumbAspect,
};
use crate::components::lists::collection_view::window::{Memory, Shown, Stand, reconcile, shown};
use crate::components::lists::marquee::band::MarqueeBand;
use crate::components::lists::marquee::hook::use_marquee;
use crate::components::lists::marquee::model::{GridShape, tiles_in as band_tiles};
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
/// `oncontextmenu` hears a right click on a tile with its index and the event, after the tile is
/// selected through `onreach` or `onselect` unless it already is selected; a right click on
/// empty space is not reported, so the page's own menu gets it. A tile whose `CollectionItem::drop`
/// is `Accepts` or `Target` is a place that takes a drop: `ondropenter` and `ondropleave` hear the
/// pointer come and go, `ondrop` hears it released there, each with the tile's index, and the
/// owner marks the tile under the pointer `Target` and the thing being dragged `Source`, as a
/// `Row`'s `RowState::drop` is driven.
///
/// `aspect` is the shape of each tile's thumbnail box: the square by default, or the landscape
/// 16:10 of a screen (`ThumbAspect::Landscape`; an image then covers its box, and the row pitch
/// shortens to match). `extent` is `Fill` by default, a view that scrolls inside the box its
/// owner gives it; `Content` makes it as tall as all its rows (`ceil(len / columns)` of them)
/// and mounts every tile, for a page that scrolls instead.
///
/// A drag that begins on empty space (not on a tile) draws a rubber band in the accent tint and
/// selects the tiles it touches; `onmarquee` hears the whole selection as the band changes it
/// (a `RowSelection` of tile indexes, to store as the owner's `selection`). With no modifier the
/// band's tiles are the selection and a click on empty space clears it; Shift or the primary
/// modifier adds them to the selection the drag began with (`selection`, or the cursor's tile).
/// The band scrolls the grid when the pointer moves near the top or bottom edge. Without
/// `onmarquee` a drag on empty space does nothing.
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
    #[props(default)] aspect: ThumbAspect,
    #[props(default)] extent: CollectionExtent,
    #[props(default)] cursor: Option<usize>,
    #[props(default)] selection: Option<RowSelection>,
    #[props(default)] onselect: Option<EventHandler<usize>>,
    #[props(default)] onreach: Option<EventHandler<Pick>>,
    #[props(default)] onopen: Option<EventHandler<usize>>,
    #[props(default)] oncontextmenu: Option<EventHandler<(usize, MouseEvent)>>,
    #[props(default)] ondropenter: Option<EventHandler<usize>>,
    #[props(default)] ondropleave: Option<EventHandler<usize>>,
    #[props(default)] ondrop: Option<EventHandler<usize>>,
    #[props(default)] onmarquee: Option<EventHandler<RowSelection>>,
    #[props(default = OVERSCAN_ROWS)] overscan: usize,
    #[props(default)] change: Change,
    #[props(default)] common: Common,
) -> Element {
    let len = items.len();
    let probe = use_rect();
    let columns = flow.columns(probe.rect().map(|rect| rect.size.width), size.tile());
    let scroller = use_scroller();
    let layout = Layout::Even {
        pitch: size.pitch_for(aspect),
        len: len.div_ceil(columns),
    };
    let rows = use_memo(use_reactive(
        (&layout, &overscan),
        move |(layout, overscan)| layout.window(*scroller.scroll().read(), overscan),
    ));
    let in_view = match extent {
        CollectionExtent::Fill => rows(),
        CollectionExtent::Content => 0..layout.len(),
    };
    let window = tiles_in(in_view.clone(), columns, len);
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
        shown(&mut state, &roster.entries(), &items, window, size, aspect)
    };
    use_cursor_reveal(
        scroller,
        cursor
            .filter(|at| *at < len)
            .map(|at| layout.span(at / columns)),
    );
    let marquee = use_marquee(scroller);
    let mut held = use_signal(|| Held::No);
    let typeahead = use_hook(|| CopyValue::new(Typeahead::default()));
    let reach = Reach {
        platform: use_platform(),
        onreach,
        onselect,
        onopen,
    };
    let drops = Drops {
        enter: ondropenter,
        leave: ondropleave,
        drop: ondrop,
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
        .map(|tile| Tile::of(tile, &items, &selected, reach, oncontextmenu, drops))
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
    let began_with = selection
        .clone()
        .unwrap_or_else(|| cursor.map_or_else(RowSelection::none, RowSelection::single));
    let data = common.data_attributes();
    let mounted = common.clone();
    let away = (held() == Held::No).then_some("away");
    let (above, below) = (layout.top(in_view.start).0, layout.below(in_view.end).0);
    let style = match extent {
        CollectionExtent::Fill => size.style_for(aspect),
        CollectionExtent::Content => {
            format!("{};height:{}px", size.style_for(aspect), layout.total().0)
        }
    };
    let grid = format!("grid-template-columns:repeat({columns},1fr)");
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-cv"),
            role: "listbox",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-size": size.slug(),
            "data-aspect": (aspect != ThumbAspect::Square).then(|| aspect.slug()),
            "data-extent": (extent != CollectionExtent::Fill).then(|| extent.slug()),
            "data-focus": away,
            style,
            onmounted: move |event| {
                probe.on_mounted(event.clone());
                mounted.mounted(event);
            },
            onfocus: move |_| held.set(Held::Yes),
            onblur: move |_| held.set(Held::No),
            onkeydown,
            onpointerdown: move |event: PointerEvent| {
                marquee.pressed(&event, began_with.clone(), onmarquee);
            },
            onpointermove: move |event: PointerEvent| {
                let width = probe.rect().map_or(Px(0.0), |rect| rect.size.width);
                let shape = GridShape {
                    columns,
                    width,
                    inset: GRID_INSET,
                    pitch: size.pitch_for(aspect),
                    tile: size.tile(),
                };
                marquee.moved(&event, &|band| band_tiles(band, shape, len), onmarquee);
            },
            onpointerup: move |_| marquee.end(),
            onpointerleave: move |_| marquee.end(),
            ..data,
            Scroller { scroller, common: marquee.common(),
                MarqueeBand { marquee }
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
                            "data-drop": tile.drop,
                            "data-drag": tile.drag,
                            onpointerdown: move |_| marquee.item_pressed(),
                            onclick: move |event| {
                                if tile.landing.is_some_and(Landing::lit) {
                                    return;
                                }
                                if let Some(act) = tile.act {
                                    act.click(&event);
                                }
                            },
                            oncontextmenu: move |event| {
                                if let Some(act) = tile.act {
                                    act.context_menu(&event);
                                }
                            },
                            onpointerenter: move |_| {
                                if let Some(landing) = tile.landing {
                                    landing.enter();
                                }
                            },
                            onpointerleave: move |_| {
                                if let Some(landing) = tile.landing {
                                    landing.leave();
                                }
                            },
                            onpointerup: move |_| {
                                if let Some(landing) = tile.landing {
                                    landing.release();
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
    drop: Option<&'static str>,
    drag: Option<&'static str>,
    act: Option<Act>,
    landing: Option<Landing>,
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
        menu: Option<EventHandler<(usize, MouseEvent)>>,
        drops: Drops,
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
        let state = item.map(|item| item.drop).unwrap_or_default();
        let selection = place.map_or(Selection::Unselected, selected);
        Tile {
            node,
            place,
            label: item.map(|item| item.caption.clone()),
            selection,
            presence,
            exit,
            leaving: exit.map(|_| "true"),
            editing: editing.then_some("true"),
            drop: state.drop_attr(),
            drag: state.drag_attr(),
            act: place.filter(|_| !editing).map(|at| Act {
                at,
                reach,
                menu,
                selection,
            }),
            landing: place.and_then(|at| Landing::of(at, state, drops)),
            content,
        }
    }
}
