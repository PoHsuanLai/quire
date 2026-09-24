//! Drawing a panel's lines: headers, rules and choice rows, each row wired to report a click,
//! a pointer move and its mount by its choice number (`menu_lines`).

use crate::components::menu_entry::{Branch, ItemView, MenuEntry, Row, Trail, item};
use crate::components::menu_lines::Line;
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::vocab::{Selection, Switch};
use crate::geometry::Point;
use dioxus::prelude::*;

/// What a panel draws around its choices: the selection, the choice whose submenu is open,
/// and what a click, a pointer move and a mount on choice `i` report.
#[derive(Clone, Copy)]
pub(crate) struct Drawn {
    /// The selected choice.
    pub selected: usize,
    /// The choice whose submenu is open.
    pub open: Option<usize>,
    /// A click on a live choice.
    pub onpick: EventHandler<usize>,
    /// The pointer moved over a choice, at this client point.
    pub onpoint: EventHandler<(usize, Point)>,
    /// A choice's element mounted.
    pub onmounted: EventHandler<(usize, MountedEvent)>,
}

/// Draw `lines` as `row`s.
pub(crate) fn render_lines<T>(lines: &[Line<'_, T>], row: Row, drawn: Drawn) -> Element {
    let mut index = 0usize;
    let rendered: Vec<Element> = lines
        .iter()
        .map(|line| {
            let (view, at) = match line.entry {
                MenuEntry::Header(text) => {
                    return rsx! {
                        SectionHeader { kind: HeaderKind::Menu, text: text.clone() }
                    };
                }
                MenuEntry::Separator => {
                    return rsx! {
                        div { class: "ds-menu-separator", role: "separator" }
                    };
                }
                MenuEntry::Item {
                    title,
                    detail,
                    tile,
                    trail,
                    check,
                    availability,
                    ..
                } => (
                    ItemView {
                        title,
                        detail: detail.as_deref(),
                        tile: tile.as_ref(),
                        trail,
                        check: *check,
                        marks: &line.marks,
                        selection: Selection::of(&index, &drawn.selected),
                        availability: *availability,
                        branch: Branch::Leaf,
                    },
                    index,
                ),
                MenuEntry::Submenu {
                    title,
                    tile,
                    availability,
                    ..
                } => (
                    ItemView {
                        title,
                        detail: None,
                        tile: tile.as_ref(),
                        trail: &Trail::None,
                        check: None,
                        marks: &line.marks,
                        selection: Selection::of(&index, &drawn.selected),
                        availability: *availability,
                        branch: Branch::Parent(if drawn.open == Some(index) {
                            Switch::On
                        } else {
                            Switch::Off
                        }),
                    },
                    index,
                ),
            };
            index += 1;
            item(
                view,
                row,
                EventHandler::new(move |()| drawn.onpick.call(at)),
                EventHandler::new(move |point| drawn.onpoint.call((at, point))),
                EventHandler::new(move |event| drawn.onmounted.call((at, event))),
            )
        })
        .collect();
    rsx! {
        for (key , line) in rendered.into_iter().enumerate() {
            Fragment { key: "{key}", {line} }
        }
    }
}
