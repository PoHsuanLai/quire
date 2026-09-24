//! WorkspacePills: the bar's workspace indicator as one segmented group on the frame ground (the
//! macOS polish pass, 2026-09-24): a `--f-pill-hover` track with the pills inside it, the current
//! workspace a raised `--f-pill` segment with the current-item shadow, the rest quiet text. It
//! replaces a row of separate `Button { Mini }`s. Each pill reports its presses with the button
//! (a click activates, a right-click opens the Space menu); a caller that drags pills to
//! reorder them wraps each `WorkspacePill` in its own element and listens there.

use crate::components::press::{Press, PressListeners};
use crate::components::vocab::Here;
use dioxus::prelude::*;

/// The group: `children` are its `WorkspacePill`s (or the caller's wrappers around them).
#[component]
pub fn WorkspacePills(label: String, children: Element) -> Element {
    rsx! {
        div { class: "ds-ws-pills", role: "group", "aria-label": "{label}", {children} }
    }
}

/// One workspace. `current` marks the workspace on screen (`aria-current`).
#[component]
pub fn WorkspacePill(
    label: String,
    #[props(default)] current: Here,
    onclick: EventHandler<Press>,
    #[props(default)] id: Option<String>,
) -> Element {
    let listen = PressListeners::new(onclick);
    rsx! {
        button {
            r#type: "button",
            class: "ds-ws-pill",
            id,
            "aria-current": match current {
                Here::Current => "true",
                Here::Elsewhere => "false",
            },
            onclick: move |event| listen.click(&event),
            oncontextmenu: move |event| listen.context_menu(&event),
            onmouseup: move |event| listen.mouse_up(&event),
            "{label}"
        }
    }
}
