//! Toast: a short notice that slides in from the right, holds, and slides out (design/30 section
//! 2.9, design/04-COMPONENTS.md section 23, design/06-INTERACTIONS.md section 9). It is not a
//! macOS component; it moves like a notification banner.
//!
//! `Ds` renders `ToastHost` after the overlay host. It draws the hub's toast, one at a time: it
//! slides in from past the right edge over `--t-move --e-out` and, when the hub hides it, slides
//! out over `--t-quick --e-exit`; nothing is drawn while the hub is empty. The hold is
//! `ToastHold` (5 s), and the pointer over the toast pauses it. A toast pushed with an undo has
//! an action button (`Undo`); a swipe to the right dismisses it, from where the hand let go.

use crate::components::overlays::swipe_glue::{SwipeOn, use_swipe_glue};
use crate::stack::toast_hub::{ToastHub, ToastState, use_toast_hub};
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
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

/// The enclosing `Ds`'s toast manager: `push(text, undo)`, one visible at a time.
pub fn use_toasts() -> ToastHub {
    use_toast_hub()
}

/// Whether the toast offers an action: only an operation with an undo does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Undoable {
    Yes,
    No,
}

/// Renders the toast. `Ds` places it; consumers never do.
#[component]
pub fn ToastHost() -> Element {
    let hub = use_toast_hub();
    // The toast keeps its last words while it slides away: a plain cell, written as the state
    // is read, so keeping them schedules no second render.
    let mut last = use_hook(|| CopyValue::new((String::new(), Undoable::No)));
    let shown = match hub.state() {
        ToastState::Shown { text, undo } => {
            let undoable = if undo.is_some() {
                Undoable::Yes
            } else {
                Undoable::No
            };
            last.set((text, undoable));
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
    let (text, undoable) = last.peek().clone();
    rsx! {
        div { class: "ds-overlay", "data-layer": "toast",
            div {
                class: "ds-toast",
                role: "status",
                "data-presence": drawn,
                "data-pulse": alias.slug(),
                "data-swipe": swipe.look(),
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
                if undoable == Undoable::Yes {
                    button {
                        r#type: "button",
                        class: "ds-toast-action",
                        onclick: move |_| {
                            if swipe.click_passes() {
                                hub.undo();
                            }
                        },
                        Glyph { icon: Icon::Undo, size: IconSize::Small }
                        "Undo"
                    }
                }
            }
        }
    }
}
