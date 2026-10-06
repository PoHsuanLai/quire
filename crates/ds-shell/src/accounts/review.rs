//! ReviewServices: what the sign-in found, before anything is stored (design/31 section 5.6): each
//! service with a switch, or why the provider has none, and the last button. When an app's chooser
//! sent the person here the button reads "Add, and allow Mail to use it": one grant, not a second
//! prompt (design/31 section 4.5). The switches are the host's.

use super::adapter::{Action, Intent, Landing};
use super::frame::StepFrame;
use super::model::{ServiceKey, ServiceLine, ServiceOffer, StepTitle};
use super::wording::{confirm_label, limitation};
use dioxus::prelude::*;
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::prelude::ListItem;
use ds::prelude::{Accessory, List, Row, TextLine};
use ds_core::vocab::Check;

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
    #[props(default)] title: StepTitle,
) -> Element {
    let done = confirm_label(allow.as_deref());
    let items: Vec<ListItem<String>> = services
        .iter()
        .map(|line| {
            let key = line.key.clone();
            let name = line.name.clone();
            let (accessory, detail) = match line.offer {
                ServiceOffer::Offered(value) => (
                    Accessory::Toggle {
                        value,
                        on_toggle: EventHandler::new(move |next| {
                            on_toggle.call((key.clone(), next))
                        }),
                    },
                    line.limit.map(limitation),
                ),
                ServiceOffer::Absent(reason) => (Accessory::None, Some(limitation(reason))),
            };
            ListItem::row(
                line.key.0.clone(),
                name.clone(),
                rsx! {
                    Row {
                        title: name.clone(),
                        detail: detail.map(TextLine::from),
                        size: RowSize::Settings,
                        accessory,
                    }
                },
            )
        })
        .collect();
    rsx! {
        StepFrame {
            shown: title,
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
