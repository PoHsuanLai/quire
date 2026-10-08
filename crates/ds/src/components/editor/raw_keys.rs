//! RawKeySurface: a focusable block for an app that draws its own region (a terminal grid) and
//! wants every key as the keyboard sent it. Where `EditSurface` turns keys into text and
//! gestures, this hands over each key down, repeat and up with its physical code, its modifiers,
//! the text it types and, where the platform reports it, the key it is unmodified
//! ([`RawKey`]).
//!
//! The surface calls `prevent_default` on every key it hears, so Tab stays in the region and no
//! key moves the page; it asks the host for the extras of each press (`DocumentHost::keys`) and
//! otherwise works from the event alone. It draws nothing and shows no focus ring: the app draws
//! its cursor. The app moves the keyboard in with `focus_soon_told` on the element its
//! `common.mounted` gives it, and runs its IME with `ds::edit::composition`.

use crate::edit::pointer::EditFocus;
use crate::edit::raw_key::{KeyPhase, RawKey};
use crate::host::document::use_document_host;
use crate::host::keys::KeyExtras;
use crate::root::common::Common;
use dioxus::prelude::*;

/// A region that hears raw keys.
///
/// - `on_key`: each key down, repeat and up, in order.
/// - `on_focus`: the region took or lost the keyboard through the renderer's own focus events
///   (a press, Tab). A focus the app asks for with `focus_soon_told` sends none; `told` is that.
/// - `common`: the app's `id`, class, `data-*` and accessible name on the element; its `mounted`
///   hears the element once it is in the document.
#[component]
pub fn RawKeySurface(
    on_key: EventHandler<RawKey>,
    #[props(default)] on_focus: Option<EventHandler<EditFocus>>,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let host = use_hook(use_document_host);
    let read = move |event: &KeyboardEvent, phase: KeyPhase| {
        event.prevent_default();
        let extras = host
            .keys()
            .map(|keys| keys.extras(&event.data()))
            .unwrap_or_else(KeyExtras::default);
        let phase = match phase {
            KeyPhase::Press if event.is_auto_repeating() => KeyPhase::Repeat,
            other => other,
        };
        on_key.call(RawKey::new(
            phase,
            (event.key(), event.code()),
            (event.modifiers(), event.location()),
            extras,
        ));
    };
    let read_up = read.clone();
    let class = common.class("ds-raw-keys");
    let data = common.data_attributes();
    let label = common.aria_label.clone();
    let id = common.id.clone();
    rsx! {
        div {
            class,
            id,
            role: "application",
            "aria-label": label,
            tabindex: "0",
            "data-keys": "capture",
            onmounted: move |event: MountedEvent| common.mounted(event),
            onkeydown: move |event: KeyboardEvent| read(&event, KeyPhase::Press),
            onkeyup: move |event: KeyboardEvent| read_up(&event, KeyPhase::Release),
            onfocus: move |_: FocusEvent| {
                if let Some(told) = on_focus {
                    told.call(EditFocus::In);
                }
            },
            onblur: move |_: FocusEvent| {
                if let Some(told) = on_focus {
                    told.call(EditFocus::Out);
                }
            },
            ..data,
            {children}
        }
    }
}
