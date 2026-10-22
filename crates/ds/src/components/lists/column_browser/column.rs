//! One column of a `ColumnBrowser`: a scrolling list of rows, each a name cell and a chevron on
//! a row with something under it.
//!
//! Markup: `div.ds-colbrowser-column[role=listbox]` (`data-active=no` for a column the keys are
//! not in, `style` its width) of a `Scroller` of `div.ds-colbrowser-item` holding a `Row`, then
//! the `span.ds-colbrowser-divider` that resizes.

use crate::components::controls::edge_grab::EdgeGrab;
use crate::components::controls::press::{button_of, press_of};
use crate::components::controls::scroller::handle::use_scroller;
use crate::components::controls::scroller::view::Scroller;
use crate::components::lists::column_browser::item::{
    BrowserRow, BrowserRowKind, ColumnPick,
};
use crate::components::lists::column_browser::model::room;
use crate::components::lists::name_cell::NameCell;
use crate::components::lists::row::row::Row;
use crate::components::lists::row::accessory::Accessory;
use crate::components::lists::virtual_list::view::use_cursor_reveal;
use crate::components::lists::virtual_table::row_pitch;
use crate::focus::press_focus::on_click;
use crate::root::common::Common;
use chordkit::Platform;
use dioxus::prelude::*;
use ds_core::geometry::scroll::ScrollSpan;
use ds_core::geometry::units::Px;
use ds_core::press::PointerButton;
use ds_core::selection::{Pick, PickHow, RowSelection};
use ds_core::vocab::{RowState, Selection};

/// Whether the keys are in a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Activity {
    /// The column the keys are in.
    Active,
    /// A parent column: its selection is drawn quietly.
    Parent,
}

/// Where a press on a row is reported, and where a grab of the column's divider is kept.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Hear {
    /// The platform whose primary modifier toggles a row.
    pub(crate) platform: Platform,
    /// Hears a row picked.
    pub(crate) onpick: Option<EventHandler<ColumnPick>>,
    /// Hears a row opened.
    pub(crate) onopen: Option<EventHandler<ColumnPick>>,
    /// The divider being dragged, if one is.
    pub(crate) grab: Signal<Option<EdgeGrab>>,
}

/// Column `column` of the browser, `width` wide.
#[component]
pub(crate) fn BrowserColumnView<K: Clone + PartialEq + 'static>(
    column: usize,
    rows: Vec<BrowserRow<K>>,
    selection: RowSelection,
    activity: Activity,
    width: Px,
    hear: Hear,
) -> Element {
    let scroller = use_scroller();
    let pitch = row_pitch();
    use_cursor_reveal(
        scroller,
        selection
            .cursor()
            .filter(|at| *at < rows.len())
            .map(|at| ScrollSpan {
                start: at as f32 * pitch.0,
                length: pitch.0,
            }),
    );
    let inactive = (activity == Activity::Parent).then_some("no");
    rsx! {
        div {
            class: "ds-colbrowser-column",
            role: "listbox",
            "data-column": "{column}",
            "data-active": inactive,
            style: "width:{width.0}px",
            Scroller { scroller,
                for (at , row) in rows.into_iter().enumerate() {
                    div {
                        key: "{at}",
                        class: "ds-colbrowser-item",
                        role: "none",
                        "data-index": "{at}",
                        onclick: move |event: MouseEvent| {
                            on_click(&event);
                            if button_of(event.trigger_button()) == Some(PointerButton::Primary) {
                                let press = press_of(&event, PointerButton::Primary);
                                let how = PickHow::of_press(hear.platform, press.modifiers);
                                reach(hear.onpick, column, Pick { row: at, how });
                            }
                        },
                        ondoubleclick: move |_| {
                            reach(
                                hear.onopen,
                                column,
                                Pick {
                                    row: at,
                                    how: PickHow::Replace,
                                },
                            );
                        },
                        {line(&row, selection.contains(at), width)}
                    }
                }
            }
            span {
                class: "ds-colbrowser-divider",
                "aria-hidden": "true",
                onpointerdown: move |event: PointerEvent| {
                    event.stop_propagation();
                    let from = Px(event.client_coordinates().x as f32);
                    let mut grab = hear.grab;
                    grab.set(Some(EdgeGrab::new(from, width)));
                },
            }
        }
    }
}

/// Tell `handler`, if there is one, that `pick` was made in `column`.
fn reach(handler: Option<EventHandler<ColumnPick>>, column: usize, pick: Pick) {
    if let Some(handler) = handler {
        handler.call(ColumnPick { column, pick });
    }
}

/// One row: the name cell, and a chevron when something is under it.
fn line<K>(row: &BrowserRow<K>, selected: bool, width: Px) -> Element {
    let selection = match selected {
        true => Selection::Selected,
        false => Selection::Unselected,
    };
    let accessory = match row.kind {
        BrowserRowKind::Branch => Accessory::Chevron,
        BrowserRowKind::Leaf => Accessory::None,
    };
    let cell = rsx! {
        NameCell { icon: row.icon.clone(), name: row.name.clone(), room: Some(room(width)) }
    };
    rsx! {
        Row {
            title: row.name.clone(),
            content: Some(cell),
            accessory,
            state: RowState {
                selection,
                ..RowState::default()
            },
            common: Common {
                aria_label: Some(row.name.clone()),
                ..Common::default()
            },
        }
    }
}
