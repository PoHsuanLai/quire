//! TabView: `NSTabView` (design/30 section 2.7): a strip of at most six tabs, a `SegmentedControl`
//! in `SelectOne`, over the body of the tab that is selected. The caller draws that body; the
//! view frames it.
//!
//! Markup: `div.ds-tab-view[data-size]` of `div.ds-tab-view-strip` around the segmented control
//! and `div.ds-tab-view-body[role=tabpanel]`.

use crate::components::controls::choice::Choice;
use crate::components::controls::segmented::{SegmentedControl, Tracking};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

/// How many tabs a tab view holds: a strip of more is a sidebar's job.
pub const TAB_LIMIT: usize = 6;

/// A tab view of `tabs` (the first six), `value` selected, `children` the body of that tab.
/// `onchange` hears the tab a press or an arrow key selects.
#[component]
pub fn TabView<T: Clone + PartialEq + 'static>(
    #[props(into)] label: String,
    tabs: Vec<Choice<T>>,
    value: T,
    onchange: EventHandler<T>,
    #[props(default)] size: ControlSize,
    children: Element,
    #[props(default)] common: Common,
) -> Element {
    let tabs: Vec<Choice<T>> = tabs.into_iter().take(TAB_LIMIT).collect();
    let name = tabs
        .iter()
        .find(|tab| tab.value == value)
        .map(|tab| tab.label.plain_text())
        .unwrap_or_default();
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-tab-view"),
            "data-size": size.slug(),
            onmounted: move |event| common.mounted(event),
            ..data,
            div { class: "ds-tab-view-strip",
                SegmentedControl::<T> {
                    label: label.clone(),
                    choices: tabs,
                    tracking: Tracking::SelectOne(value),
                    size,
                    onchange: move |next| onchange.call(next),
                }
            }
            div { class: "ds-tab-view-body", role: "tabpanel", "aria-label": name, {children} }
        }
    }
}
