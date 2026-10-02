//! The plan list component.

use super::model::{PlanOut, PlanView};
use crate::root::common::Common;
use dioxus::prelude::*;

/// A plan as a list of steps grouped by effect, each with its state and, in a draft, a switch.
/// `on_out` hears what the plan asks of its owner (run, edit, stop, resume, undo all).
#[component]
pub fn PlanList(
    view: PlanView,
    on_out: EventHandler<PlanOut>,
    #[props(default)] common: Common,
) -> Element {
    let _ = on_out;
    let class = common.class("ds-plan-list");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-groups": view.groups.len(),
            onmounted: move |event| common.mounted(event),
            ..data,
        }
    }
}
