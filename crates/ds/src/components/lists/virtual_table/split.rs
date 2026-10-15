//! A `VirtualTable` that holds its leading columns in place while the others scroll sideways
//! under them, drawn in two panes.
//!
//! `position: sticky` is not available (Blitz does not lay it out, and ds-lint bans it), so the
//! table is two boxes side by side instead of one box with sticky cells. Both panes sit inside
//! the table's one `Scroller`, so the rows of the pinned pane and of the scrolling pane are in
//! the same vertical flow: a wheel or a drag over either moves both by the same pixels, and a row
//! is the same height and the same distance from the top in both because both are built from
//! the same window of rows, with the same spacers above and below. The pinned pane is
//! `flex:none`. The scrolling pane is a box that scrolls sideways (`overflow-x:auto`) and holds
//! the other columns.
//!
//! The header is the one part that must not scroll vertically yet must scroll sideways with the
//! scrolling pane, so it stays above the `Scroller` and is two headers too: the pinned header
//! stands still, and the scrolling header sits in a clip and is moved by the scrolling pane's
//! own offset (`translateX(-offset)`), which the pane reports on every scroll. The header follows
//! the rows by a frame at most.
//!
//! A row is two elements, one in each pane, that must look like one. Selection and the stripe are
//! the row's index, so both draw them; the pointer being over either is shared through a
//! signal, so both take the tint. A selected row is one rounded block across the two panes: the
//! pinned pane's rows are rounded on the left and the scrolling pane's on the right
//! (virtual_table.css). Once the columns are scrolled a faint line marks the edge between the
//! panes (`data-across=yes` on the table, as `data-scrolled` is for the header). Only the
//! scrolling pane is the list for assistive technology; the pinned pane is `aria-hidden`.

use crate::components::controls::scroller::handle::ScrollerRef;
use crate::components::controls::scroller::view::Scroller;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Selection;

/// The id of the box that scrolls sideways in a table whose root has `id`, once it holds columns
/// in place: the root itself scrolls sideways otherwise. A caller that scrolls the table
/// sideways (to show a find hit) names this box.
pub fn columns_id(id: &str) -> String {
    format!("{id}-columns")
}

/// Whether a pane's rows are announced to assistive technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place {
    /// They are the list, of this many rows.
    Listed(usize),
    /// They repeat the other pane's rows.
    Hidden,
}

/// One row of a pane: its index, whether it is selected, whether it is striped, and what it
/// holds.
pub(super) struct Line {
    pub(super) row: usize,
    pub(super) selection: Selection,
    pub(super) stripe: Option<&'static str>,
    pub(super) content: Element,
}

/// One pane: the header over its columns, the rows' cells, and how its columns are laid out.
pub(super) struct Pane {
    pub(super) head: Element,
    pub(super) lines: Vec<Line>,
    /// `grid-template-columns` of the pane's columns.
    pub(super) template: String,
    /// The pane's columns together, in logical pixels.
    pub(super) width: Px,
}

/// What the two panes are drawn from.
pub(super) struct Panes {
    pub(super) held: Pane,
    pub(super) scrolling: Pane,
    pub(super) scroller: ScrollerRef,
    pub(super) label: String,
    /// The root's id, which names the sideways box (`columns_id`).
    pub(super) id: Option<String>,
    pub(super) rows: usize,
    /// The height of the rows above the window, of the rows below it, and of one row.
    pub(super) above: f32,
    pub(super) below: f32,
    pub(super) height: f32,
    /// How far the scrolling pane is scrolled sideways, and where to hear it change.
    pub(super) across: Signal<Px>,
    /// The row the pointer is over.
    pub(super) hovered: Signal<Option<usize>>,
}

/// The body of a table that holds columns: the two headers, then the `Scroller` of the two
/// panes.
pub(super) fn body(panes: Panes) -> Element {
    let Panes {
        held,
        scrolling,
        scroller,
        label,
        id,
        rows,
        above,
        below,
        height,
        across,
        hovered,
    } = panes;
    let (held_width, held_template) = (held.width.0, held.template);
    let (side_width, side_template) = (scrolling.width.0, scrolling.template);
    let offset = across().0;
    let columns = id.as_deref().map(columns_id);
    let mut across = across;
    rsx! {
        div { class: "ds-vtable-body",
            div { class: "ds-vtable-heads",
                div {
                    class: "ds-vtable-held-head",
                    style: "width:calc({held_width}px + var(--pin-inset, 0px));--table-cols:{held_template}",
                    {held.head}
                }
                div { class: "ds-vtable-head-clip",
                    div {
                        class: "ds-vtable-head-track",
                        style: "min-width:calc({side_width}px + var(--pin-inset, 0px));--table-cols:{side_template};transform:translateX(-{offset}px)",
                        {scrolling.head}
                    }
                }
            }
            Scroller { scroller,
                div { class: "ds-vtable-split",
                    div {
                        class: "ds-vtable-held",
                        "aria-hidden": "true",
                        style: "width:calc({held_width}px + var(--pin-inset, 0px));--table-cols:{held_template}",
                        div { class: "ds-vtable-spacer", style: "height:{above}px" }
                        for line in held.lines {
                            HeldItem {
                                key: "{line.row}",
                                row: line.row,
                                hovered,
                                selection: line.selection,
                                stripe: line.stripe,
                                height,
                                place: Place::Hidden,
                                content: line.content,
                            }
                        }
                        div { class: "ds-vtable-spacer", style: "height:{below}px" }
                    }
                    div {
                        class: "ds-vtable-columns",
                        id: columns,
                        style: "--table-cols:{side_template}",
                        onscroll: move |event: ScrollEvent| {
                            let to = Px(event.scroll_left() as f32);
                            if *across.peek() != to {
                                across.set(to);
                            }
                        },
                        div {
                            class: "ds-vtable-track",
                            style: "min-width:calc({side_width}px + var(--pin-inset, 0px))",
                            div { class: "ds-vtable-spacer", style: "height:{above}px" }
                            div {
                                class: "ds-vtable-rows",
                                role: "listbox",
                                "aria-label": label,
                                for line in scrolling.lines {
                                    HeldItem {
                                        key: "{line.row}",
                                        row: line.row,
                                        hovered,
                                        selection: line.selection,
                                        stripe: line.stripe,
                                        height,
                                        place: Place::Listed(rows),
                                        content: line.content,
                                    }
                                }
                            }
                            div { class: "ds-vtable-spacer", style: "height:{below}px" }
                        }
                    }
                }
            }
        }
    }
}

/// One row of one pane. It reads the shared hover itself, so the pointer crossing rows redraws
/// these small items and not the whole table.
#[component]
fn HeldItem(
    row: usize,
    hovered: Signal<Option<usize>>,
    selection: Selection,
    stripe: Option<&'static str>,
    height: f32,
    place: Place,
    content: Element,
) -> Element {
    let mut hovered = hovered;
    let over = (hovered() == Some(row)).then_some("yes");
    let (hidden, posinset, setsize) = match place {
        Place::Listed(rows) => (None, Some((row + 1).to_string()), Some(rows.to_string())),
        Place::Hidden => (Some("true"), None, None),
    };
    rsx! {
        div {
            class: "ds-vtable-item",
            role: "none",
            "aria-hidden": hidden,
            "aria-posinset": posinset,
            "aria-setsize": setsize,
            "data-row": row.to_string(),
            "data-selected": (selection == Selection::Selected).then_some("yes"),
            "data-stripe": stripe,
            "data-hover": over,
            style: "height:{height}px",
            onpointerenter: move |_| hovered.set(Some(row)),
            onpointerleave: move |_| {
                if *hovered.peek() == Some(row) {
                    hovered.set(None);
                }
            },
            {content}
        }
    }
}
