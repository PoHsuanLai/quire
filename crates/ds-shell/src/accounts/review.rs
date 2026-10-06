//! ReviewServices: what the sign-in found, before anything is stored (design/31 section 5.6): each
//! service with a switch, or why the provider has none, and the last button. When an app's chooser
//! sent the person here the button reads "Add, and allow Mail to use it": one grant, not a second
//! prompt (design/31 section 4.5). The switches are the host's.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::{ServiceKey, ServiceLine, ServiceOffer};
use super::wording::{confirm_label, limitation};
use dioxus::prelude::*;
use ds::components::content::label::{Label, LabelRole, LabelStyle};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::row::Row;
use ds::prelude::List;
use ds::prelude::ListItem;
use ds::prelude::Toggle;
use ds_core::vocab::Check;
use ds_style::tokens::control_size::ControlSize;

/// The review of `account`'s `services`. `allow` names the app to allow in the same step.
/// `on_toggle` hears a switch flipped; `on_done` the last button or Return.
#[component]
pub fn ReviewServices(
    #[props(into)] account: String,
    services: Vec<ServiceLine>,
    #[props(default)] allow: Option<String>,
    on_toggle: EventHandler<(ServiceKey, Check)>,
    on_done: EventHandler<()>,
    on_back: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let done = confirm_label(allow.as_deref());
    let items: Vec<ListItem<String>> = services
        .iter()
        .map(|line| {
            let key = line.key.clone();
            let name = line.name.clone();
            let (toggle, detail) = match line.offer {
                ServiceOffer::Offered(value) => (
                    Some((
                        value,
                        EventHandler::new(move |next| on_toggle.call((key.clone(), next))),
                    )),
                    line.limit.map(limitation),
                ),
                ServiceOffer::Absent(reason) => (None, Some(limitation(reason))),
            };
            ListItem::row(
                line.key.0.clone(),
                name.clone(),
                rsx! {
                    Row {
                        title: name.clone(),
                        content: Some(service(name, detail, toggle)),
                    }
                },
            )
        })
        .collect();
    rsx! {
        StepFrame {
            step: "review",
            title: "Choose what to use",
            onenter: EventHandler::new(move |()| on_done.call(())),
            oncancel: on_cancel,
            body: rsx! {
                div { class: "ds-acc-caption", "{account}" }
                List { label: "Services", items, style: ListStyle::Inset }
            },
            actions: rsx! {
                Action { label: "Back", onclick: move |()| on_back.call(()) }
                Action { label: "Cancel", intent: Intent::Cancel, onclick: move |()| on_cancel.call(()) }
                Action { label: done, intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_done.call(()) }
            },
        }
    }
}

/// One service's line: its name over the reason it is limited, and its switch at the end. The
/// switch stands in the row's own content rather than its accessory, because an accessory fences
/// every key it hears and Return must reach the sheet's default button wherever the keyboard is.
fn service(
    name: String,
    detail: Option<&'static str>,
    toggle: Option<(Check, EventHandler<Check>)>,
) -> Element {
    rsx! {
        span { class: "ds-acc-service",
            span { class: "ds-acc-service-text",
                Label { text: name.clone(), style: LabelStyle::Body }
                if let Some(detail) = detail {
                    Label { text: detail, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
            if let Some((value, onchange)) = toggle {
                Toggle { label: name, value, size: ControlSize::Mini, onchange }
            }
        }
    }
}
