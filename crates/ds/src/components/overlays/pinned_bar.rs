//! PinnedBar: an offer bar that stays at the top or bottom edge of a scroll container over the
//! content ("Suggested: ...", "3 selected"). Blitz has no `position: sticky`, so the bar is not
//! inside the scroller: the container is a positioned box, `children` (the scroller) fill it and
//! the bar is laid over its edge. The bar covers the first or last rows at the scroller's rest;
//! the caller pads the scroller by the bar's height when that matters.
//!
//! Markup: `div.ds-pinned[data-edge=bottom]` of `children` and `div.ds-pinned-bar[role=region]`
//! (the `bar`).

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;

/// Which edge of the container the bar is pinned to, `data-edge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
#[non_exhaustive]
pub enum PinEdge {
    /// The top edge.
    #[default]
    Top,
    /// The bottom edge.
    Bottom,
}

/// `children` (the scroll container) with `bar` pinned over its `edge`; `label` names the bar.
#[component]
pub fn PinnedBar(
    #[props(into)] label: String,
    bar: Element,
    children: Element,
    #[props(default)] edge: PinEdge,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    let bar_label = common.aria_label.clone().unwrap_or(label);
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-pinned"),
            "data-edge": (edge != PinEdge::Top).then(|| edge.slug()),
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
            div {
                class: "ds-pinned-bar",
                role: "region",
                "aria-label": bar_label,
                {bar}
            }
        }
    }
}
