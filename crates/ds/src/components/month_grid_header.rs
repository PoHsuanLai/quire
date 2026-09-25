//! A MonthGrid's header: the month's title and, with `onstep`, the previous and next buttons
//! (design/04-COMPONENTS.md section 39). The regular grid uses `IconButton { Tool }` (28 x 26);
//! that alone is taller than the compact grid can spare in a small widget's 132 px (sill Q190),
//! so the compact grid draws plain 14 px glyph buttons (`ds-month-step`, an 11 px glyph).

use crate::components::icon_button::{IconButton, IconButtonVariant};
use crate::components::month_grid_data::Step;
use crate::components::month_grid_density::Drawn;
use crate::components::text_runs::{Text, text};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// The header row at `density`.
pub(crate) fn header(title: &Text, density: Drawn, onstep: Option<EventHandler<Step>>) -> Element {
    rsx! {
        div { class: "ds-month-header",
            span { class: "ds-month-title", {text(title)} }
            if let Some(onstep) = onstep {
                {step_button(Step::Previous, Icon::ChevronLeft, density, onstep)}
                {step_button(Step::Next, Icon::ChevronRight, density, onstep)}
            }
        }
    }
}

/// One of the header's step buttons, sized for `density`.
fn step_button(step: Step, icon: Icon, density: Drawn, onstep: EventHandler<Step>) -> Element {
    match density {
        Drawn::Regular => rsx! {
            IconButton {
                variant: IconButtonVariant::Tool,
                icon,
                label: step.label(),
                tooltip: step.label().to_owned(),
                onclick: move |_| onstep.call(step),
            }
        },
        Drawn::Compact => rsx! {
            button {
                r#type: "button",
                class: "ds-month-step",
                "aria-label": step.label(),
                title: step.label(),
                onclick: move |_| onstep.call(step),
                Glyph { icon, size: IconSize::Micro }
            }
        },
    }
}
