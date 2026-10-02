//! Sidebar: the pane of a `SplitView` that lists places (`NSSplitViewItem` sidebar, design/30
//! section 2.7): an optional header (a search field, a command pill), then sections that scroll
//! (source-list `List`s at the sidebar's size, and whatever else the caller puts between them:
//! pinned tiles, Today tabs), then an optional foot that stays at the bottom. The ground is paper
//! or leaves the window's own colour showing (a Space's tint), by `fill`.
//!
//! Markup: `nav.ds-sidebar[data-size][data-fill]` of `div.ds-sidebar-header`,
//! `div.ds-sidebar-body` (one `div.ds-sidebar-section` per section) and `div.ds-sidebar-foot`.

use crate::components::chrome::sidebar_model::{SidebarFill, SidebarSection};
use crate::components::lists::list::list::List;
use crate::components::lists::list::model::ListStyle;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::control_size::SidebarSize;
use std::hash::Hash;

/// A sidebar of `sections`, with `header` above them and `foot` below. `cursor` is the place the
/// keys rest on and the caller draws selected, across every list; `onselect` hears where a click,
/// an arrow or a typed letter reached. `size` is the rows' height, `appearance.sidebar_size`'s
/// value. `fill` is what it paints behind its rows: its own ground (the default) or nothing
/// (`SidebarFill::Clear`), so the window's colour shows through.
#[component]
pub fn Sidebar<K: Clone + PartialEq + Hash + 'static>(
    #[props(into)] label: String,
    sections: Vec<SidebarSection<K>>,
    #[props(default)] size: SidebarSize,
    #[props(default)] cursor: Option<K>,
    onselect: EventHandler<K>,
    #[props(default)] header: Option<Element>,
    #[props(default)] foot: Option<Element>,
    #[props(default)] fill: SidebarFill,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        nav {
            id: common.id.clone(),
            class: common.class("ds-sidebar"),
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "data-size": size.slug(),
            "data-fill": (fill == SidebarFill::Clear).then(|| fill.slug()),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(header) = header {
                div { class: "ds-sidebar-header", {header} }
            }
            div { class: "ds-sidebar-body",
                for (at , section) in sections.into_iter().enumerate() {
                    div { key: "{at}", class: "ds-sidebar-section",
                        match section {
                            SidebarSection::List(items) => rsx! {
                                List::<K> {
                                    label: label.clone(),
                                    items,
                                    style: ListStyle::SourceList,
                                    sidebar: size,
                                    cursor: cursor.clone(),
                                    onselect: move |key| onselect.call(key),
                                }
                            },
                            SidebarSection::Custom(content) => content,
                        }
                    }
                }
            }
            if let Some(foot) = foot {
                div { class: "ds-sidebar-foot", {foot} }
            }
        }
    }
}
