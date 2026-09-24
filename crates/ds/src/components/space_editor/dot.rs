//! One Space's dot in the sidebar foot, the switch between Spaces (design/04-COMPONENTS.md
//! section 32, `S:147-150`).

use crate::components::vocab::{Here, Shortcut, Switch};
use crate::space::FrameVars;
use dioxus::prelude::*;

/// One Space's dot in the sidebar foot.
#[component]
pub fn SpaceDot(
    name: String,
    frame: FrameVars,
    here: Here,
    shortcut: Shortcut,
    onclick: EventHandler<()>,
) -> Element {
    let pressed = match here {
        Here::Current => Switch::On,
        Here::Elsewhere => Switch::Off,
    };
    let keys = shortcut.glyphs();
    rsx! {
        button {
            r#type: "button",
            class: "ds-space-dot",
            "aria-pressed": pressed.aria(),
            "aria-label": "{name} Space",
            title: "{name} ({keys})",
            style: "background:{frame.gradient}",
            onclick: move |_| onclick.call(()),
        }
    }
}
