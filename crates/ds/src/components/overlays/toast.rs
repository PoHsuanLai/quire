//! Toast: a short notice that slides in from the right, holds, and slides out (design/30 section
//! 2.9, design/04-COMPONENTS.md section 23, design/06-INTERACTIONS.md section 9). It is not a
//! macOS component; it moves like a notification banner.
//!
//! `Ds` renders `ToastHost` after the overlay host. It draws the hub's toast, one at a time: it
//! slides in from past the right edge over `--t-move --e-out` and, when the hub hides it, slides
//! out over `--t-quick --e-exit`; nothing is drawn while the hub is empty. The hold is
//! `ToastHold` (5 s), and the pointer over the toast pauses it. A toast pushed with an undo has
//! an action button (`Undo`, or the consumer's own: [`ToastHub::push_action`]); a swipe to the right dismisses it, from where the hand let go.
//!
//! A toast that appears under a still pointer does not take it: `data-armed=waiting` makes it
//! transparent to the pointer (so a press reaches the button below and nothing counts as a hover,
//! so the hold runs) until a move of the pointer arrives (`Ds` tells the hub), then `live`.
//!
//! While a sheet or a peek is up the toast sits at the top right, where the Mac's banners sit,
//! instead of the bottom right: a sheet's buttons are at its bottom, and a toast over them would
//! take the next press (its hold pauses under a still pointer).

use crate::components::overlays::swipe_glue::{SwipeOn, use_swipe_glue};
use crate::stack::host::use_overlays;
use crate::stack::toast_hub::{ToastAction, ToastHub, ToastState, use_toast_hub};
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_motion::swipe::SwipeMetrics;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::layer::ZLayer;

/// The enclosing `Ds`'s toast manager: `push(text, undo)`, one visible at a time.
pub fn use_toasts() -> ToastHub {
    use_toast_hub()
}

/// Renders the toast. `Ds` places it; consumers never do.
#[component]
pub fn ToastHost() -> Element {
    let hub = use_toast_hub();
    let overlays = use_overlays();
    // The toast keeps its last words while it slides away: a plain cell, written as the state
    // is read, so keeping them schedules no second render.
    let mut last = use_hook(|| CopyValue::new((String::new(), None::<ToastAction>)));
    let shown = match hub.state() {
        ToastState::Shown { text, action, .. } => {
            last.set((text, action));
            Shown::Visible
        }
        ToastState::Hidden => Shown::Hidden,
    };
    let Presented { presence, alias } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::PanelIn,
            exit: Exit::PanelOut,
        },
        None,
    );
    let swipe = use_swipe_glue(
        SwipeOn::Yes,
        SwipeMetrics::default(),
        EventHandler::new(move |()| hub.hide()),
    );
    let Some(drawn) = presence.drawn_slug() else {
        return rsx! {};
    };
    let (text, action) = last.peek().clone();
    let place = match overlays.shown_on(ZLayer::Peek) {
        Shown::Visible => "top",
        Shown::Hidden => "bottom",
    };
    rsx! {
        div { class: "ds-overlay", "data-layer": "toast",
            div {
                class: "ds-toast",
                role: "status",
                "data-presence": drawn,
                "data-pulse": alias.slug(),
                "data-swipe": swipe.look(),
                "data-place": place,
                "data-armed": hub.armed().slug(),
                style: swipe.style(),
                onmouseenter: move |_| hub.pause(),
                onmouseleave: move |_| {
                    swipe.left();
                    hub.resume();
                },
                onpointerdown: move |event| swipe.down(&event),
                onpointermove: move |event| swipe.moved(&event),
                onpointerup: move |event| swipe.released(&event),
                span { class: "ds-toast-body", "{text}" }
                if let Some(action) = action {
                    button {
                        r#type: "button",
                        class: "ds-toast-action",
                        onclick: move |_| {
                            if swipe.click_passes() {
                                hub.press_action();
                            }
                        },
                        if let Some(icon) = action.icon {
                            Glyph { icon, size: IconSize::Small }
                        }
                        "{action.label}"
                    }
                }
            }
        }
    }
}
