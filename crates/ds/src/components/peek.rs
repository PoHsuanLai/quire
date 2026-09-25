//! Peek: a thread's reader in a panel over the card (design/04-COMPONENTS.md section 24,
//! design/06-INTERACTIONS.md section 15).
//!
//! Through the overlay host with its own scrim: the scrim, the close tool, or Escape (when the
//! peek is the topmost layer) closes it, at once. Center plays `peek-in` over `--t-big
//! --e-spring`; Full covers the card and plays it over `--t-move --e-out` (`Anim::PeekFullIn`).

use crate::appearance::PeekMode;
use crate::components::icon_button::{IconButton, IconButtonVariant};
use crate::components::popover::{Dismiss, Stacking, escape_closes, use_float};
use crate::components::scrim::scrim_button;
use crate::icon::Icon;
use crate::motion::anim::Anim;
use crate::motion::entrance::use_entrance;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// The `data-mode` word.
fn mode_slug(mode: PeekMode) -> &'static str {
    match mode {
        PeekMode::Center => "center",
        PeekMode::Full => "full",
    }
}

/// The entrance each mode plays (design/05-MOTION.md section 5 rows 6 and 64).
fn entrance(mode: PeekMode) -> Anim {
    match mode {
        PeekMode::Center => Anim::PeekIn,
        PeekMode::Full => Anim::PeekFullIn,
    }
}

/// A reader floating over the card.
#[component]
pub fn Peek(
    mode: PeekMode,
    label: String,
    onclose: EventHandler<()>,
    children: Element,
) -> Element {
    let float = use_float(ZLayer::Peek, Stacking::Layer(Dismiss::EscOnly));
    let presence = use_entrance(entrance(mode));
    // Both close controls say "Close peek" (`S:1579`); the dialog is labelled by its thread.
    let close = "Close peek".to_string();
    float.show(
        rsx! {
            {scrim_button(&close, move || float.is_top(), onclose)}
            div {
                class: "ds-peek",
                "data-mode": mode_slug(mode),
                "data-presence": presence.slug(),
                role: "dialog",
                "aria-label": "{label}",
                onkeydown: move |event| escape_closes(float, &event, onclose),
                div { class: "ds-peek-tools",
                    IconButton {
                        variant: IconButtonVariant::Tool,
                        icon: Icon::X,
                        label: close.clone(),
                        onclick: move |_| onclose.call(()),
                    }
                }
                {children}
            }
        },
        onclose,
    );
    rsx! {}
}
