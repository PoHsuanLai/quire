//! Lists, mailo gaps 4: a strip whose press acts before any measurement, and sidebar places
//! that name themselves and hand over their pointer, lit as the drop target under it.

use super::Section;
use dioxus::prelude::*;
use ds::{
    ActionId, Anim, DropState, Emphasis, Here, HoverStrip, Icon, ItemKind, ListRow, PlaceId,
    Presence, PulseKey, Selection, Shown, SidebarItem, StaggerIndex, StripAction,
};

/// Archive and snooze, each doing nothing on its measured click: the press says what happened.
fn actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("snooze", Icon::Clock, "Snooze", "Snooze until…"),
    ]
    .into_iter()
    .map(|(id, icon, label, fly)| StripAction {
        id: ActionId(id.to_string()),
        icon,
        label: label.to_string(),
        fly: fly.to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    })
    .collect()
}

/// A row whose strip's press is heard at once.
#[component]
pub fn StripPress() -> Element {
    let mut pressed = use_signal(|| None::<ActionId>);
    let said = pressed().map_or("nothing yet".to_string(), |id| id.0);
    rsx! {
        Section {
            title: "A strip that acts before it measures",
            note: "on_press hears the button inside the click, before the rect read; the measured onclick follows only where layout answers.",
            ul { class: "g-list g-stage-pad",
                ListRow {
                    selection: Selection::Selected,
                    emphasis: Emphasis::Strong,
                    index: StaggerIndex::new(0),
                    presence: Presence::Present,
                    name: "Dana Okafor",
                    via: None,
                    subject: "Re: UIDL stability",
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    star_pulse: ds::PulseKey::rest(Anim::StarPop),
                    strip: rsx! {
                        HoverStrip {
                            actions: actions(),
                            shown: Shown::Visible,
                            on_press: move |id| pressed.set(Some(id)),
                        }
                    },
                    onclick: |_| {},
                }
            }
            p { class: "g-note", "Pressed: {said}" }
        }
    }
}

/// The places a thread can be dropped on.
const PLACES: [(&str, &str, Icon); 3] = [
    ("inbox", "Inbox", Icon::Inbox),
    ("archive", "Archive", Icon::Archive),
    ("label:7", "Invoices", Icon::Tag),
];

/// Places that light as the drop target under the pointer, from their own pointer hooks.
#[component]
pub fn DropPlaces() -> Element {
    // Posed with Archive lit, as a drag over it would leave it.
    let mut over = use_signal(|| Some("archive"));
    let mut dropped = use_signal(|| "nothing yet".to_string());
    rsx! {
        Section {
            title: "Places as drop targets",
            note: "place writes data-place; onpointerenter, onpointerleave, onpointermove and onpointerup hand the pointer to the caller, which sets drop: DropState::Target on the place under it (accent-soft, 1.045). Release over one to drop.",
            div { class: "g-col g-stage-pad", style: "width:220px",
                for (id , label , icon) in PLACES {
                    SidebarItem {
                        key: "{id}",
                        kind: ItemKind::Place { icon },
                        label,
                        here: Here::Elsewhere,
                        count: None,
                        presence: Presence::Present,
                        preview: None,
                        pulse: PulseKey::rest(Anim::Gulp),
                        onclick: |_| {},
                        onclose: None,
                        drop: if over() == Some(id) { DropState::Target } else { DropState::Idle },
                        place: PlaceId(id.to_string()),
                        onpointerenter: move |_| over.set(Some(id)),
                        onpointerleave: move |_| over.set(None),
                        onpointerup: move |_| dropped.set(id.to_string()),
                    }
                }
            }
            p { class: "g-note", "Dropped on: {dropped}" }
        }
    }
}
