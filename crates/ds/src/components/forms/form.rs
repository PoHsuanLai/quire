//! Form: the page of grouped sections (System Settings' right-hand pane, a sheet's body). It
//! owns the page margin and the gap from one section to the next; the sections own everything
//! inside them.
//!
//! Markup: `div.ds-form[role=form]` of `section.ds-form-section` children.

use crate::root::common::Common;
use dioxus::prelude::*;

/// A form: `children` are its [`FormSection`](crate::components::forms::form_section::FormSection)s,
/// stacked 20 apart inside a 20 margin.
#[component]
pub fn Form(children: Element, #[props(default)] common: Common) -> Element {
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class: common.class("ds-form"),
            role: "form",
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
        }
    }
}
