//! ColumnBrowser: Finder's column view (Miller columns). A row of columns, each a list of rows (a
//! glyph, a name cut from the middle, a chevron on a row with something under it), scrolling
//! sideways when they outgrow the box. The data is the app's: it lists the `columns` and each
//! one's `selection`, and the browser reports what the person did. It never walks a filesystem.
//!
//! A pick in column N (`onpick`, a `ColumnPick`) is the app's to answer: it sets that column's
//! selection, drops the columns after N and, for a row with something under it, supplies column
//! N+1 with nothing selected. The report is made even for a row that is selected already, so the
//! app can always cut the columns after it. A selection is row indexes and a `Pick`, as
//! `VirtualTable`'s: the app that wants several rows selected in the last column folds the `Pick`
//! into its `RowSelection` with `RowSelection::picked`.
//!
//! The keys are in the last column with a selection (the first when none has one). Up and Down
//! move within it, Home and End go to its ends, Right enters the selected row's column (it picks
//! that column's first row), Left goes back to the parent (it re-picks the parent's selected row),
//! and letters jump to the next name that starts with them. Command-Down, as a double-click,
//! asks `onopen`. Return and Space are left to the app's keymap (Finder's Rename and Quick
//! Look), and so is any key with a modifier besides Shift.
//!
//! When the last column's selected row has nothing under it and the app gave a `preview`, it is
//! shown after the columns, as Finder shows a file's preview. The app gives it only for the
//! selection it describes. The divider after a column drags to resize: every column takes the
//! width, as Finder's do, from `column_width` to start with. All of a column's rows are mounted.
//!
//! Markup: `div.ds-colbrowser[role=group]` (`data-focus=away` when it does not hold the
//! keyboard) of `div.ds-colbrowser-strip` of the columns (`div.ds-colbrowser-column`, each ending
//! in its `span.ds-colbrowser-divider`) and then `div.ds-colbrowser-preview` when there is one.

use crate::components::controls::edge_grab::EdgeGrab;
use crate::components::lists::column_browser::column::{Activity, BrowserColumnView, Hear};
use crate::components::lists::column_browser::item::{BrowserColumn, ColumnPick};
use crate::components::lists::column_browser::model::{
    BrowserKey, active, browser_key, cursor, leaf_selected, target,
};
use crate::components::lists::virtual_table::model::key_how;
use crate::keys::use_platform;
use crate::root::common::Common;
use crate::stack::typeahead::Typeahead;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::selection::{Pick, PickHow};
use ds_core::time::clock::now;

/// A column's width before anyone drags it (Finder's default).
const COLUMN_WIDTH: Px = Px(220.0);
/// The least a drag leaves a column.
const LEAST_WIDTH: f32 = 120.0;

/// Whether the browser holds the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Held {
    Yes,
    No,
}

/// `columns` side by side. `label` names the browser for a screen reader. `onpick` hears a click
/// or a key asking for a row, `onopen` hears Command-Down or a double-click, and `preview` fills
/// the space after the last column while a row with nothing under it is selected there.
#[component]
pub fn ColumnBrowser<K: Clone + PartialEq + 'static>(
    label: String,
    columns: Vec<BrowserColumn<K>>,
    #[props(default)] preview: Option<Element>,
    #[props(default = COLUMN_WIDTH)] column_width: Px,
    #[props(default)] onpick: Option<EventHandler<ColumnPick>>,
    #[props(default)] onopen: Option<EventHandler<ColumnPick>>,
    #[props(default)] common: Common,
) -> Element {
    let platform = use_platform();
    let mut width = use_signal(|| column_width);
    let mut grab = use_signal(|| None::<EdgeGrab>);
    let mut held = use_signal(|| Held::No);
    let typeahead = use_hook(|| CopyValue::new(Typeahead::default()));
    let here = active(&columns);
    let hear = Hear {
        platform,
        onpick,
        onopen,
        grab,
    };
    let onkeydown = {
        let columns = columns.clone();
        move |event: KeyboardEvent| {
            let Some(key) = browser_key(&event.key(), event.modifiers(), platform) else {
                return;
            };
            let how = key_how(event.modifiers());
            let found = match &key {
                BrowserKey::Jump(text) => jumped(&columns, text, typeahead),
                _ => target(&columns, &key, how),
            };
            match (key, found) {
                (BrowserKey::Open, Some(pick)) => {
                    if let Some(onopen) = onopen {
                        event.prevent_default();
                        event.stop_propagation();
                        onopen.call(pick);
                    }
                }
                (BrowserKey::Open, None) => {}
                (_, Some(pick)) => {
                    if let Some(onpick) = onpick {
                        event.prevent_default();
                        onpick.call(pick);
                    }
                }
                (_, None) => {}
            }
        }
    };
    let preview = preview.filter(|_| leaf_selected(&columns));
    let away = (held() == Held::No).then_some("away");
    let data = common.data_attributes();
    let mounted = common.clone();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-colbrowser"),
            role: "group",
            tabindex: "0",
            "aria-label": common.aria_label.clone().unwrap_or(label),
            "data-focus": away,
            onmounted: move |event| mounted.mounted(event),
            onfocus: move |_| held.set(Held::Yes),
            onblur: move |_| held.set(Held::No),
            onkeydown,
            onpointermove: move |event: PointerEvent| {
                if let Some(edge) = grab() {
                    let next = edge.size_at(Px(event.client_coordinates().x as f32));
                    width.set(Px(next.0.max(LEAST_WIDTH)));
                }
            },
            onpointerup: move |_| grab.set(None),
            onpointerleave: move |_| grab.set(None),
            ..data,
            div { class: "ds-colbrowser-strip",
                for (at , column) in columns.into_iter().enumerate() {
                    BrowserColumnView::<K> {
                        key: "{at}",
                        column: at,
                        rows: column.rows,
                        selection: column.selection,
                        activity: if at == here { Activity::Active } else { Activity::Parent },
                        width: width(),
                        hear,
                    }
                }
                if let Some(preview) = preview {
                    div { class: "ds-colbrowser-preview", {preview} }
                }
            }
        }
    }
}

/// Where typing `text` takes the keys: the next name in the active column that starts with what
/// has been typed.
fn jumped<K>(
    columns: &[BrowserColumn<K>],
    text: &str,
    mut typeahead: CopyValue<Typeahead>,
) -> Option<ColumnPick> {
    let column = active(columns);
    let rows = &columns.get(column)?.rows;
    let names: Vec<&str> = rows.iter().map(|row| row.name.as_str()).collect();
    let from = cursor(columns).unwrap_or_else(|| names.len().saturating_sub(1));
    let (next, found) = typeahead.peek().clone().typed(text, now(), &names, from);
    typeahead.set(next);
    found.map(|row| ColumnPick {
        column,
        pick: Pick {
            row,
            how: PickHow::Replace,
        },
    })
}
