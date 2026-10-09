//! Column: `children` stacked top to bottom, `gap` apart, lined up by `align`. It draws nothing
//! of its own and may be nested.
//!
//! Markup: `div.ds-column[data-align=start|center|end]` of `children`; the gap enters as the
//! custom property `--column-gap`.

use crate::components::chrome::column::model::ColumnAlign;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::spacing::SpacingToken;
use ds_style::tokens::token::Token;

/// A vertical stack: `gap` between children (8 px unless given), `align` across the width
/// (stretched unless given).
#[component]
pub fn Column(
    children: Element,
    #[props(default = SpacingToken::S8)] gap: SpacingToken,
    #[props(default)] align: ColumnAlign,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-column"),
            style: "--column-gap:var({gap.var().as_str()})",
            "aria-label": common.aria_label.clone(),
            "data-align": (align != ColumnAlign::Stretch).then(|| align.slug()),
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
        }
    }
}
