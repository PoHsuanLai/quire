//! The mailo gaps 4 list cases: a strip whose press is heard before any measurement (its markup
//! is the strip's own: the press is a listener, which a server render does not write), and
//! sidebar places that name themselves for a drag, one lit as its target.

use crate::cases::{Case, DANA};
use crate::rows::strip_actions;
use dioxus::prelude::*;
use ds::components::vocab::{DropState, Here, PulseKey};
use ds::{ActionId, Anim, HoverStrip, Icon, ItemKind, PlaceId, Presence, Shown, SidebarItem};

/// A place named `place`, in `drop` state, with every pointer hook attached.
fn place(kind: ItemKind, label: &str, place: &str, drop: DropState) -> Element {
    rsx! {
        SidebarItem {
            kind,
            label,
            here: Here::Elsewhere,
            count: None,
            presence: Presence::Present,
            preview: None,
            pulse: PulseKey::rest(Anim::Gulp),
            onclick: |_| {},
            onclose: None,
            drop,
            place: PlaceId(place.to_string()),
            onpointerenter: |_| {},
            onpointerleave: |_| {},
            onpointermove: |_| {},
            onpointerup: |_| {},
        }
    }
}

pub const MAILO4_CASES: &[Case] = &[
    Case {
        component: "hover_strip",
        state: "on-press",
        make: || rsx! { HoverStrip { actions: strip_actions(), shown: Shown::Visible, on_press: |_: ActionId| {} } },
    },
    Case {
        component: "sidebar_item",
        state: "place-named",
        make: || {
            place(
                ItemKind::Place {
                    icon: Icon::Archive,
                },
                "Archive",
                "archive",
                DropState::Idle,
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "place-drop-target",
        make: || {
            place(
                ItemKind::Place { icon: Icon::Tag },
                "Invoices",
                "label:7",
                DropState::Target,
            )
        },
    },
    Case {
        component: "sidebar_item",
        state: "pinned-named",
        make: || {
            place(
                ItemKind::Pinned { avatar: DANA },
                "Dana Okafor",
                "pin:dana",
                DropState::Idle,
            )
        },
    },
];
