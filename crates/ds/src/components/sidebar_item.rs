//! SidebarItem: a navigable place on the frame, with the seal, gulp and destination preview
//! (design/04-COMPONENTS.md section 19).

use crate::components::avatar::{AvatarFace, face};
use crate::components::count::{Count, CountPlace};
use crate::components::vocab::{Here, PulseKey};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::motion::presence::Presence;
use dioxus::prelude::*;

/// What kind of place.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    /// Inbox, Starred, a label: carries the seal when current.
    Place {
        /// Its glyph.
        icon: Icon,
    },
    /// A pinned person or saved search, with a favicon.
    Pinned {
        /// The favicon.
        avatar: AvatarFace,
    },
    /// An opened thread or draft: slides in, has a close button.
    Today {
        /// The favicon.
        avatar: AvatarFace,
    },
}

/// A preview the item is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Preview {
    /// A strip button would send the thread here: the `dest` ring pulses.
    Destination,
}

impl ItemKind {
    /// The `data-kind` word.
    fn slug(&self) -> &'static str {
        match self {
            ItemKind::Place { .. } => "place",
            ItemKind::Pinned { .. } => "pinned",
            ItemKind::Today { .. } => "today",
        }
    }
}

impl Preview {
    /// The `data-preview` word.
    fn slug(self) -> &'static str {
        match self {
            Preview::Destination => "destination",
        }
    }
}

/// The `aria-current` word.
fn aria_current(here: Here) -> &'static str {
    match here {
        Here::Current => "true",
        Here::Elsewhere => "false",
    }
}

/// The class list and `data-pulse` for an item playing `pulse` (the gulp).
fn pulse_attrs(pulse: PulseKey) -> (String, Option<&'static str>) {
    match pulse.attrs() {
        Some((anim, alias)) => (format!("ds-sidebar-item {anim}"), Some(alias)),
        None => ("ds-sidebar-item".to_string(), None),
    }
}

/// A place in the sidebar.
///
/// Place and Pinned are buttons; Today is a `div[role=button]` because it holds its own close
/// button (a button cannot contain a button). A current Place wears the seal, a real span
/// rather than `::before` (O-22's fallback). `presence` animates a Today entry in (`tab-in`) and
/// out (`tab-out`; the consumer drops it at `settle(Anim::TabOut)`); the other kinds do not move.
/// `pulse` is a `use_pulse(Anim::Gulp)` key, fired when the place receives something.
#[component]
pub fn SidebarItem(
    kind: ItemKind,
    label: String,
    here: Here,
    count: Option<u32>,
    presence: Presence,
    preview: Option<Preview>,
    pulse: PulseKey,
    onclick: EventHandler<()>,
    onclose: Option<EventHandler<()>>,
) -> Element {
    let (class, alias) = pulse_attrs(pulse);
    let slug = kind.slug();
    let current = aria_current(here);
    let preview = preview.map(Preview::slug);
    let count = count.map(|value| rsx! { Count { value, place: CountPlace::Item } });
    match kind {
        ItemKind::Place { icon } => rsx! {
            button {
                r#type: "button",
                class,
                "data-kind": slug,
                "aria-current": current,
                "data-preview": preview,
                "data-pulse": alias,
                onclick: move |_| onclick.call(()),
                if here == Here::Current {
                    span { class: "ds-sidebar-item-seal" }
                }
                Glyph { icon, size: IconSize::Base }
                span { class: "ds-sidebar-item-text", "{label}" }
                {count}
            }
        },
        ItemKind::Pinned { avatar } => rsx! {
            button {
                r#type: "button",
                class,
                "data-kind": slug,
                "aria-current": current,
                "data-preview": preview,
                "data-pulse": alias,
                onclick: move |_| onclick.call(()),
                {face(avatar)}
                span { class: "ds-sidebar-item-text ds-truncate", "data-emphasis": "plain", "{label}" }
                {count}
            }
        },
        ItemKind::Today { avatar } => rsx! {
            div {
                class,
                "data-kind": slug,
                role: "button",
                tabindex: "0",
                "aria-current": current,
                "data-presence": presence.slug(),
                "data-preview": preview,
                "data-pulse": alias,
                onclick: move |_| onclick.call(()),
                onkeydown: move |event| {
                    if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                        onclick.call(());
                    }
                },
                {face(avatar)}
                span { class: "ds-sidebar-item-text ds-truncate", "data-emphasis": "plain", "{label}" }
                {count}
                if let Some(onclose) = onclose {
                    button {
                        r#type: "button",
                        class: "ds-sidebar-item-close",
                        "aria-label": "Close",
                        onclick: move |event| {
                            // Closing is not opening: the entry itself must not also navigate.
                            event.stop_propagation();
                            onclose.call(());
                        },
                        Glyph { icon: Icon::X, size: IconSize::Small }
                    }
                }
            }
        },
    }
}
