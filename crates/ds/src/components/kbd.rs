//! Kbd: key caps, one `kbd.ds-kbd` per key (design/04-COMPONENTS.md section 9).

use crate::components::vocab::{GlyphKind, Shortcut};
use dioxus::prelude::*;

/// A key cap's size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum KbdSize {
    /// Data 10.5.
    #[default]
    Regular,
    /// Data 9.5, inside hints.
    Small,
}

impl KbdSize {
    /// The `data-size` word.
    fn slug(self) -> &'static str {
        match self {
            KbdSize::Regular => "regular",
            KbdSize::Small => "small",
        }
    }
}

/// A shortcut as key caps, one per key and nothing between them (O-2: glyphs, no separator).
/// `data-glyph="arrow"` (sill Q111) marks Up, Down, Left and Right: at `KbdSize::Small` the
/// face rule draws them larger than the rest of the small face, so the arrow reads as an arrow
/// rather than a dash.
#[component]
pub fn Kbd(shortcut: Shortcut, #[props(default)] size: KbdSize) -> Element {
    rsx! {
        for key in shortcut.0 {
            kbd {
                class: "ds-kbd",
                "data-size": size.slug(),
                "data-glyph": key.glyph_kind().map(GlyphKind::slug),
                "{key.glyph()}"
            }
        }
    }
}
