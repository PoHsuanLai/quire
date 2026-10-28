//! Drawing a panel's lines: headers, status lines, rules and items, each item wired to report a
//! click, a pointer move and its mount by its choice number (`choices`).

use crate::components::content::tip_text::TipText;
use crate::components::menus::alive::Alive;
use crate::components::menus::item::item::{MenuItem, SwatchRow};
use crate::components::menus::item::swatches::{DotView, swatches};
use crate::components::menus::item::view::{
    Branch, Columns, ItemView, RowEvents, header, info, item,
};
use dioxus::prelude::*;
use ds_core::geometry::units::Point;
use ds_core::press::Press;
use ds_core::vocab::{Selection, Shown};

/// What a panel draws around its choices: the highlight, the choice whose submenu is open,
/// and what a click, a pointer move and a mount on choice `i` report.
#[derive(Clone, Copy)]
pub(crate) struct Drawn {
    /// The highlighted choice; none under a caller's cursor that names none, or while the
    /// highlight is blinking out.
    pub selected: Option<usize>,
    /// The choice whose submenu is open.
    pub open: Option<usize>,
    /// The window's keymap, which draws each key equivalent for the platform.
    pub keymap: crate::keys::Keys,
    /// A click on a live choice.
    pub onpick: EventHandler<usize>,
    /// The pointer moved over a choice, at this client point.
    pub onpoint: EventHandler<(usize, Point)>,
    /// A choice's element mounted.
    pub onmounted: EventHandler<(usize, MountedEvent)>,
    /// A button released over a choice, where the panel listens for it (a menu's root panel,
    /// for press-drag-release).
    pub onrelease: Option<EventHandler<(usize, Press)>>,
}

/// The leading columns `items` need: a column for the state mark when any item has one, and one
/// for the image likewise, so every title in the panel lines up.
pub(crate) fn columns<T>(items: &[MenuItem<T>]) -> Columns {
    let has = |wanted: fn(&MenuItem<T>) -> bool| items.iter().any(wanted);
    Columns {
        state: has(|item| matches!(item, MenuItem::Item { check: Some(_), .. })),
        image: has(|item| {
            matches!(
                item,
                MenuItem::Item { image: Some(_), .. } | MenuItem::Submenu { image: Some(_), .. }
            )
        }),
    }
}

/// Draw `items`.
pub(crate) fn render_lines<T>(items: &[MenuItem<T>], drawn: Drawn, alive: &Alive) -> Element {
    let columns = columns(items);
    let mut index = 0usize;
    let rendered: Vec<Element> = items
        .iter()
        .map(|entry| {
            let (view, at) = match entry {
                MenuItem::Header(text) => return header(text),
                MenuItem::Info { title, detail } => return info(title, detail.as_deref()),
                MenuItem::Separator => {
                    return rsx! {
                        div { class: "ds-menu-separator", role: "separator" }
                    };
                }
                MenuItem::Swatches(row) => {
                    let first = index;
                    index += row.swatches.len();
                    return swatch_row(row, first, drawn, alive);
                }
                MenuItem::Item {
                    title,
                    image,
                    key,
                    hint,
                    check,
                    availability,
                    text,
                    ..
                } => (
                    ItemView {
                        title,
                        text: Some(text),
                        tip: text
                            .tip
                            .as_ref()
                            .map(|name| TipText::new(name.clone()).with_shortcut_opt(key.clone())),
                        image: image.as_ref(),
                        key: key.as_ref().map(|shortcut| drawn.keymap.text_of(shortcut)),
                        hint: hint.as_deref(),
                        check: *check,
                        highlight: Selection::of(&Some(index), &drawn.selected),
                        availability: *availability,
                        branch: Branch::Leaf,
                        columns,
                    },
                    index,
                ),
                MenuItem::Submenu {
                    title,
                    image,
                    availability,
                    ..
                } => (
                    ItemView {
                        title,
                        text: None,
                        tip: None,
                        image: image.as_ref(),
                        key: None,
                        hint: None,
                        check: None,
                        highlight: Selection::of(&Some(index), &drawn.selected),
                        availability: *availability,
                        branch: Branch::Parent(if drawn.open == Some(index) {
                            Shown::Visible
                        } else {
                            Shown::Hidden
                        }),
                        columns,
                    },
                    index,
                ),
            };
            index += 1;
            item(view, events(drawn, at, alive))
        })
        .collect();
    rsx! {
        for (key , line) in rendered.into_iter().enumerate() {
            Fragment { key: "{key}", {line} }
        }
    }
}

/// What the choice `at` reports through `drawn`.
fn events(drawn: Drawn, at: usize, alive: &Alive) -> RowEvents {
    let release = drawn
        .onrelease
        .map(|onrelease| EventHandler::new(move |press| onrelease.call((at, press))));
    RowEvents {
        pick: EventHandler::new(move |()| drawn.onpick.call(at)),
        point: EventHandler::new(move |point| drawn.onpoint.call((at, point))),
        mounted: EventHandler::new(move |event| drawn.onmounted.call((at, event))),
        release,
        alive: alive.clone(),
    }
}

/// A row of swatches whose dots are the choices from `first` on.
fn swatch_row<T>(row: &SwatchRow<T>, first: usize, drawn: Drawn, alive: &Alive) -> Element {
    let dots = row.swatches.iter().enumerate().map(|(pos, dot)| {
        let at = first + pos;
        let view = DotView {
            name: &dot.name,
            colour: dot.colour,
            check: dot.check,
            highlight: Selection::of(&Some(at), &drawn.selected),
            availability: row.availability,
        };
        (at, view, events(drawn, at, alive))
    });
    swatches(&row.title, dots.collect())
}
