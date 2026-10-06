//! PageHeader: the head of a settings page inside a pane stack. Below the root it holds the back
//! button, a chevron and the title of the page it returns to (Sequoia's drill-in), then the
//! page's own title; at the root it holds the title alone.
//!
//! Markup: `header.ds-page-header` of an optional back `Button` and `h2.ds-page-title`.

use crate::components::controls::button::Button;
use crate::components::controls::button_model::Bezel;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// A page head: `title` in the pane title type; `back` the title of the parent page, which makes
/// the back button ("Back to <parent>" to a screen reader) that calls `on_back`. `back_id` is the
/// back button's element id, for the stack to move the focus to.
#[component]
pub fn PageHeader(
    title: String,
    #[props(default)] back: Option<String>,
    #[props(default)] on_back: EventHandler<()>,
    #[props(default)] back_id: Option<String>,
    #[props(default)] common: Common,
) -> Element {
    let data = common.data_attributes();
    rsx! {
        header {
            id: common.id.clone(),
            class: common.class("ds-page-header"),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(parent) = back {
                Button {
                    label: parent.clone(),
                    icon: Some(Icon::ChevronLeft),
                    bezel: Bezel::Inline,
                    size: ControlSize::Small,
                    onclick: move |_| on_back.call(()),
                    common: Common {
                        id: back_id,
                        aria_label: Some(format!("Back to {parent}")),
                        ..Common::default()
                    },
                }
            }
            h2 { class: "ds-page-title", "{title}" }
        }
    }
}
