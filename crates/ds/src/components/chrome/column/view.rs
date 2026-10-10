//! Column: `children` stacked top to bottom, `gap` apart, lined up by `align`. It draws nothing
//! of its own and may be nested.
//!
//! Markup: `div.ds-column[data-align=start|center|end][data-gap=none][data-extent=fill|fixed]
//! [data-size]` of `children`; a stepped gap enters as the custom property `--column-gap`.

use crate::components::chrome::column::model::{ColumnAlign, ColumnExtent, ColumnGap};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;

/// A vertical stack: `gap` between children (a spacing step, 8 px unless given; `ColumnGap::None`
/// for none), `align` across the width (stretched unless given), `extent` along its height (as
/// tall as the children unless given).
#[component]
pub fn Column(
    children: Element,
    #[props(into, default)] gap: ColumnGap,
    #[props(default)] align: ColumnAlign,
    #[props(default)] extent: ColumnExtent,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    let (extent_kind, extent_size) = extent.attributes();
    let gap_style = match gap {
        ColumnGap::None => None,
        ColumnGap::Step(step) => Some(format!("--column-gap:var({})", step.var().as_str())),
    };
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-column"),
            style: gap_style,
            "aria-label": common.aria_label.clone(),
            "data-gap": (gap == ColumnGap::None).then_some("none"),
            "data-extent": extent_kind,
            "data-size": extent_size,
            "data-align": (align != ColumnAlign::Stretch).then(|| align.slug()),
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
        }
    }
}
