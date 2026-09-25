//! SidebarItem: a navigable place on the frame, with the seal, gulp and destination preview
//! (design/04-COMPONENTS.md section 19).

use crate::components::avatar::{AvatarFace, face};
use crate::components::count::{Count, CountPlace};
use crate::components::row_hooks::relay;
use crate::components::vocab::{DropState, Here, PulseKey};
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

/// What a scheduled Today row carries after its label: when it leaves, and a button that
/// cancels it (the send goes back to being a draft).
#[derive(Debug, Clone, PartialEq)]
pub struct TodayTrailing {
    /// When it leaves, as the caller words it ("Mon 9:00").
    pub time: String,
    /// The cancel button's accessible name, in the caller's words ("Cancel sending Q3 notes").
    pub cancel: String,
    /// The person cancelled it. The row itself does not also open.
    pub on_cancel: EventHandler<()>,
}

/// Which place an item is, by the consumer's own name (`inbox`, `label:7`), written as
/// `data-place` so a drag in progress can tell which place is under the pointer (mailo gaps 4:
/// design/06-INTERACTIONS.md section 6.1 reads the drop target off the element).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlaceId(pub String);

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

/// The item's classes: its own and the drop place's, whose rules draw `drop` (shared with
/// `TreeItem`, mailo gaps 6).
const CLASS: &str = "ds-sidebar-item ds-drop-place";

/// The class list and `data-pulse` for an item playing `pulse` (the gulp).
fn pulse_attrs(pulse: PulseKey) -> (String, Option<&'static str>) {
    match pulse.attrs() {
        Some((anim, alias)) => (format!("{CLASS} {anim}"), Some(alias)),
        None => (CLASS.to_string(), None),
    }
}

/// A place in the sidebar.
///
/// Place and Pinned are buttons; Today is a `div[role=button]` because it holds its own close
/// button (a button cannot contain a button). A current Place wears the seal, a real span
/// rather than `::before` (O-22's fallback). `presence` animates a Today entry in (`tab-in`) and
/// out (`Presence::Leaving(Exit::TabOut)` plays `tab-out`; the consumer drops it at
/// `settle(Anim::TabOut)`, which is what `RosterState::leave` returns for that exit); the other
/// kinds do not move.
/// `pulse` is a `use_pulse(Anim::Gulp)` key, fired when the place receives something. `drop` is
/// the item's part in a drag: `Target` while a dragged thread is over a place that accepts it,
/// `Source` while the item itself is dragged. A Today item's close button is named "Close
/// {label}", so each row's close says whose it is; `trailing` puts a scheduled row's time and
/// its cancel button after the label (Today only; other kinds ignore it).
///
/// A drag over the sidebar (mailo gaps 4, design/06 section 6.1) needs each place to say which
/// it is and to hear the pointer: `place` is written as `data-place`, and `onpointerenter`,
/// `onpointerleave`, `onpointermove` and `onpointerup` hand the item's pointer events to the
/// caller, who sets `drop: DropState::Target` on the place under a dragged thread (lit with
/// `--accent-soft` and grown to 1.045) and applies the drop on the release. The listeners are
/// always attached and call nothing without a handler, so a server render's markup is unchanged.
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
    #[props(default)] drop: DropState,
    #[props(default)] trailing: Option<TodayTrailing>,
    #[props(default)] place: Option<PlaceId>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointermove: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerup: Option<EventHandler<PointerEvent>>,
) -> Element {
    let (class, alias) = pulse_attrs(pulse);
    let place = place.map(|PlaceId(name)| name);
    let slug = kind.slug();
    let current = here.aria_current();
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
                "data-drop": drop.drop_attr(),
                "data-drag": drop.drag_attr(),
                "data-place": place,
                onpointerenter: relay(onpointerenter),
                onpointerleave: relay(onpointerleave),
                onpointermove: relay(onpointermove),
                onpointerup: relay(onpointerup),
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
                "data-drop": drop.drop_attr(),
                "data-drag": drop.drag_attr(),
                "data-place": place,
                onpointerenter: relay(onpointerenter),
                onpointerleave: relay(onpointerleave),
                onpointermove: relay(onpointermove),
                onpointerup: relay(onpointerup),
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
                "data-drop": drop.drop_attr(),
                "data-drag": drop.drag_attr(),
                "data-place": place,
                onpointerenter: relay(onpointerenter),
                onpointerleave: relay(onpointerleave),
                onpointermove: relay(onpointermove),
                onpointerup: relay(onpointerup),
                onclick: move |_| onclick.call(()),
                onkeydown: move |event| {
                    if matches!(event.key(), Key::Enter) || event.key() == Key::Character(" ".into()) {
                        onclick.call(());
                    }
                },
                {face(avatar)}
                span { class: "ds-sidebar-item-text ds-truncate", "data-emphasis": "plain", "{label}" }
                {count}
                if let Some(trailing) = trailing {
                    {today_trailing(trailing)}
                }
                if let Some(onclose) = onclose {
                    button {
                        r#type: "button",
                        class: "ds-sidebar-item-close",
                        "aria-label": "Close {label}",
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

/// A scheduled row's time and cancel button.
fn today_trailing(trailing: TodayTrailing) -> Element {
    let TodayTrailing {
        time,
        cancel,
        on_cancel,
    } = trailing;
    rsx! {
        span { class: "ds-sidebar-item-time", "{time}" }
        button {
            r#type: "button",
            class: "ds-sidebar-item-cancel",
            "aria-label": "{cancel}",
            title: "{cancel}",
            onclick: move |event| {
                // Cancelling is not opening: the entry itself must not also navigate.
                event.stop_propagation();
                on_cancel.call(());
            },
            Glyph { icon: Icon::X, size: IconSize::Small }
        }
    }
}
