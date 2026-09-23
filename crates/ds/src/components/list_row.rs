//! ListRow: one item in a list, the thread row (design/04-COMPONENTS.md section 16).

use crate::components::vocab::{Emphasis, PulseKey, Selection, StaggerIndex, Switch};
use crate::motion::presence::Presence;
use dioxus::prelude::*;

/// One row: dot, name and via, subject, snippet, tail, star, and a hover-strip slot.
#[component]
pub fn ListRow(
    selection: Selection,
    emphasis: Emphasis,
    index: StaggerIndex,
    presence: Presence,
    name: String,
    via: Option<Element>,
    subject: String,
    snippet: Option<String>,
    time: String,
    tags: Element,
    star: Option<(Switch, EventHandler<Switch>)>,
    star_pulse: PulseKey,
    strip: Option<Element>,
    onclick: EventHandler<MouseData>,
) -> Element {
    todo!()
}
