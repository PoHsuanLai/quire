//! NameCell: the Name cell of a file list as Finder draws it, a glyph and then the name on one
//! line. A name that does not fit loses its middle, so the extension stays (`Truncate::Middle`).
//! Blitz cannot cut by measure, so the cut is made from a character budget: the caller gives the
//! room the cell has (`room`, the column's width less the cell's padding) and [`name_chars`] turns
//! it into the budget; with no room the whole name is written and the cell clips it at its end.
//!
//! Markup: `span.ds-name-cell` of `span.ds-name-icon` (the glyph, `aria-hidden`) and
//! `span.ds-name-text` (`aria-label` the whole name).
//!
//! [`name_chars`]: crate::components::lists::name_cell::model::name_chars

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::lists::name_cell::model::name_chars;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::text::clip::Truncate;
use ds_style::icon::render::IconSize;

/// `icon` and then `name`, cut from the middle to fit `room` when it is given. For a cell of a
/// `VirtualTable` or `Table`.
#[component]
pub fn NameCell(
    icon: IconSource,
    name: String,
    #[props(default)] room: Option<Px>,
    #[props(default)] common: Common,
) -> Element {
    let shown = match room {
        Some(room) => Truncate::Middle.clip(&name, name_chars(room)),
        None => name.clone(),
    };
    let data = common.data_attributes();
    let mounted = common.clone();
    rsx! {
        span {
            id: common.id.clone(),
            class: common.class("ds-name-cell"),
            onmounted: move |event| mounted.mounted(event),
            ..data,
            span { class: "ds-name-icon", "aria-hidden": "true",
                IconView { source: icon, size: IconSize::Base }
            }
            span { class: "ds-name-text", "aria-label": name, "{shown}" }
        }
    }
}
