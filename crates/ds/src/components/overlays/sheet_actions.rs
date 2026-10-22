//! SheetActions: the row of buttons at the foot of a sheet (design/30 section 2.5), side by side
//! as a Mac sheet draws them.
//!
//! Markup: `div.ds-sheet-actions[role=group]` of an optional `span.ds-sheet-actions-leading` and
//! the buttons. The buttons end at the trailing edge in the order given, so the default button
//! goes last (`Cancel`, `Go`), with `--s-8` between them; the row brings no padding, as a
//! sheet's content does.

use crate::root::common::Common;
use dioxus::prelude::*;

/// A sheet's buttons in one row. `leading` is a control kept at the start of the row, apart from
/// the rest (a help button, or a Cancel that stands away from the default button). `children`
/// are the buttons, the default one last.
#[component]
pub fn SheetActions(
    #[props(default)] leading: Option<Element>,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-sheet-actions"),
            role: "group",
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(leading) = leading {
                span { class: "ds-sheet-actions-leading", {leading} }
            }
            {children}
        }
    }
}
