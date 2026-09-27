//! Chord: a shortcut as plain text, the way Spotlight hints at an action's keys ("Reveal in
//! Files ⌘R"): the glyphs in a row, no key caps, no border, in the secondary ink at the size of
//! the words beside it. [`Kbd`](crate::Kbd) stays the boxed key caps for a keyboard legend.

use crate::components::vocab::Shortcut;
use dioxus::prelude::*;

/// `shortcut` as one run of glyphs (`⌘⇧R`, modifiers in the Mac's order, O-2), in `--ink-soft`
/// and the surrounding face and size so a modifier glyph draws as large as a letter. An empty
/// shortcut draws nothing.
#[component]
pub fn Chord(shortcut: Shortcut) -> Element {
    let glyphs = shortcut.glyphs();
    if glyphs.is_empty() {
        return rsx! {};
    }
    rsx! {
        span { class: "ds-chord", "{glyphs}" }
    }
}
