//! SidebarItem: a navigable place on the frame, with the seal, gulp and destination preview
//! (design/04-COMPONENTS.md section 19).

use crate::components::content::avatar::{AvatarFace, face};
use crate::components::controls::count::{Count, CountPlace};
use crate::components::lists::row_hooks::relay;
use crate::focus::click::kept_click;
use dioxus::prelude::*;
use ds_core::vocab::{RowState, Selection};
use ds_core::word::Word;
use ds_motion::presence::Presence;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

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
/// `data-place` so a drag in progress can tell which place is under the pointer
/// (design/06-INTERACTIONS.md section 6.1 reads the drop target off the element).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlaceId(pub String);

/// A preview the item is showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
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

/// The item's classes: its own and the drop place's, whose rules draw `drop` (shared with
/// `TreeItem`).
const CLASS: &str = "ds-sidebar-item ds-drop-place";

/// A place in the sidebar.
///
/// Place and Pinned are buttons; Today is a `div[role=button]` because it holds its own close
/// button (a button cannot contain a button). A current Place wears the seal, a real span
/// rather than `::before` (O-22's fallback). `presence` animates a Today entry in (`row-in`) and
/// out (`Presence::Leaving(Exit::Row)` plays `row-out`, the roster's exit; the consumer drops it
/// at `settle(Anim::RowOut)`); the other kinds do not move.
/// `state` is the item's [`RowState`]: it reads `selection` (`aria-current`) and `drop`, the item's part in a drag: `Target` while a dragged thread is over a place that accepts it,
/// `Source` while the item itself is dragged. A Today item's close button is named "Close
/// {label}", so each row's close says whose it is; `trailing` puts a scheduled row's time and
/// its cancel button after the label (Today only; other kinds ignore it).
///
/// A drag over the sidebar (design/06 section 6.1) needs each place to say which
/// it is and to hear the pointer: `place` is written as `data-place`, and `onpointerenter`,
/// `onpointerleave`, `onpointermove` and `onpointerup` hand the item's pointer events to the
/// caller, who sets `state.drop` to `DropState::Target` on the place under a dragged thread (lit with
/// `--accent-soft` and grown to 1.045) and applies the drop on the release. The listeners are
/// always attached and call nothing without a handler, so a server render's markup is unchanged.
#[component]
pub fn SidebarItem(
    kind: ItemKind,
    label: String,
    #[props(default)] state: RowState,
    count: Option<u32>,
    presence: Presence,
    preview: Option<Preview>,
    onclick: EventHandler<()>,
    onclose: Option<EventHandler<()>>,
    #[props(default)] trailing: Option<TodayTrailing>,
    #[props(default)] place: Option<PlaceId>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointermove: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerup: Option<EventHandler<PointerEvent>>,
) -> Element {
    let RowState {
        selection: here,
        drop,
        ..
    } = state;
    let place = place.map(|PlaceId(name)| name);
    let slug = kind.slug();
    let current = here.aria_current();
    let preview = preview.map(Preview::slug);
    let count = count.map(|value| rsx! { Count { value, place: CountPlace::Item } });
    match kind {
        ItemKind::Place { icon } => rsx! {
            button {
                r#type: "button",
                class: CLASS,
                "data-kind": slug,
                "aria-current": current,
                "data-preview": preview,
                "data-drop": drop.drop_attr(),
                "data-drag": drop.drag_attr(),
                "data-place": place,
                onpointerenter: relay(onpointerenter),
                onpointerleave: relay(onpointerleave),
                onpointermove: relay(onpointermove),
                onpointerup: relay(onpointerup),
                onclick: move |_| onclick.call(()),
                if here == Selection::Selected {
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
                class: CLASS,
                "data-kind": slug,
                "aria-current": current,
                "data-preview": preview,
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
                class: CLASS,
                "data-kind": slug,
                role: "button",
                tabindex: "0",
                "aria-current": current,
                "data-presence": presence.slug(),
                "data-preview": preview,
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
                            kept_click(&event);
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
                kept_click(&event);
            },
            Glyph { icon: Icon::X, size: IconSize::Small }
        }
    }
}
