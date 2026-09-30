//! Sidebar: the pane of a `SplitView` that lists places (`NSSplitViewItem` sidebar, design/30
//! section 2.7): an optional header (a search field, a title), then a `List` in the `SourceList`
//! style at the sidebar's size, on the sidebar ground.
//!
//! Markup: `nav.ds-sidebar[data-size]` of `div.ds-sidebar-header` and `div.ds-sidebar-body`
//! around the list.

use crate::components::lists::list::list::List;
use crate::components::lists::list::model::{ListItem, ListStyle};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::control_size::SidebarSize;
use std::hash::Hash;

/// A sidebar of `items`. `cursor` is the place the keys rest on and the caller draws selected;
/// `onselect` hears where a click, an arrow or a typed letter reached. `size` is the rows'
/// height, `appearance.sidebar_size`'s value.
#[component]
pub fn Sidebar<K: Clone + PartialEq + Hash + 'static>(
    #[props(into)] label: String,
    items: Vec<ListItem<K>>,
    #[props(default)] size: SidebarSize,
    #[props(default)] cursor: Option<K>,
    onselect: EventHandler<K>,
    #[props(default)] header: Option<Element>,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        nav {
            id: common.id.clone(),
            class: common.class("ds-sidebar"),
            "aria-label": common.aria_label.clone().unwrap_or_else(|| label.clone()),
            "data-size": size.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(header) = header {
                div { class: "ds-sidebar-header", {header} }
            }
            div { class: "ds-sidebar-body",
                List::<K> {
                    label: label.clone(),
                    items,
                    style: ListStyle::SourceList,
                    sidebar: size,
                    cursor,
                    onselect: move |key| onselect.call(key),
                }
            }
        }
    }
}
