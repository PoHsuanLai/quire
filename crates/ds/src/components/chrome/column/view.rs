//! Column: `children` stacked top to bottom, `gap` apart, lined up by `align`. It draws nothing
//! of its own and may be nested.
//!
//! Markup: `div.ds-column[data-align=start|center|end][data-gap=none][data-extent=fill|fixed]
//! [data-size]` of `children`; a stepped gap enters as the custom property `--column-gap`.

use crate::components::chrome::column::model::{ColumnAlign, ColumnExtent};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::spacing::SpacingToken;

/// A vertical stack: `gap` between children (a spacing step, 8 px unless given; `None`
/// for none), `align` across the width (stretched unless given), `extent` along its height (as
/// tall as the children unless given). `oncontextmenu` hears a right click anywhere in the column
/// and keeps it from reaching the menu of whatever holds the column.
#[component]
pub fn Column(
    children: Element,
    #[props(default = Some(SpacingToken::S8))] gap: Option<SpacingToken>,
    #[props(default)] align: ColumnAlign,
    #[props(default)] extent: ColumnExtent,
    #[props(default)] oncontextmenu: Option<EventHandler<MouseEvent>>,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    let (extent_kind, extent_size) = extent.attributes();
    let gap_style = gap.map(|step| format!("--column-gap:var({})", step.var().as_str()));
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-column"),
            style: gap_style,
            "aria-label": common.aria_label.clone(),
            "data-gap": gap.is_none().then_some("none"),
            "data-extent": extent_kind,
            "data-size": extent_size,
            "data-align": (align != ColumnAlign::Stretch).then(|| align.slug()),
            oncontextmenu: move |event: MouseEvent| {
                if let Some(hear) = oncontextmenu {
                    event.stop_propagation();
                    hear.call(event);
                }
            },
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
        }
    }
}
