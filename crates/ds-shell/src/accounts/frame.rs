//! The column every step of the add-account sheet stands in: a title, the step's body and a row
//! of buttons, with the window keys. Return runs the step's default action, Escape its cancel;
//! a button that has the keyboard keeps its own Return.

use super::adapter::Disc;
use super::model::StepTitle;
use dioxus::prelude::*;
use ds::components::content::avatar::AvatarSize;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};

/// A step's column: bare, so a host puts it in a window of its own (or in `AccountSheet`). `shown`
/// says who draws the title: `StepTitle::Host` leaves it to the window's title bar. `mark` is the
/// provider the step is about, shown round in the header.
/// `onenter` is the default action (absent while it cannot run, as a form with a
/// required field empty), `oncancel` the way out. Both keys are taken here so the sheet around
/// the step hears each once.
#[component]
pub(crate) fn StepFrame(
    step: &'static str,
    #[props(into)] title: String,
    #[props(default)] shown: StepTitle,
    #[props(default)] mark: Option<MarkProvider>,
    #[props(default)] style: MarkStyle,
    #[props(default)] onenter: Option<EventHandler<()>>,
    oncancel: EventHandler<()>,
    body: Element,
    actions: Element,
) -> Element {
    rsx! {
        div {
            class: "ds-acc-step",
            "data-step": step,
            onkeydown: move |event| match event.key() {
                Key::Escape => {
                    event.stop_propagation();
                    event.prevent_default();
                    oncancel.call(());
                }
                Key::Enter => {
                    if let Some(enter) = onenter {
                        event.stop_propagation();
                        event.prevent_default();
                        enter.call(());
                    }
                }
                _ => {}
            },
            if mark.is_some() || shown == StepTitle::Own {
                div { class: "ds-acc-header",
                    if let Some(provider) = mark {
                        Disc { provider, size: AvatarSize::Size48, style: style.clone() }
                    }
                    if shown == StepTitle::Own {
                        div { class: "ds-acc-title", "{title}" }
                    }
                }
            }
            div { class: "ds-acc-body", {body} }
            div { class: "ds-acc-actions", {actions} }
        }
    }
}
