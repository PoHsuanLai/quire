//! KeyEquivalent: a shortcut as it is written beside a menu item or on a key cap (design/30
//! section 2.1): the symbols in the Mac's order, Control, Option, Shift, Command, then the key.
//! `Text` is the plain run of glyphs after a menu item's words; `Cap` draws each key as a key
//! cap. A "key plus label" is a `KeyEquivalent` beside a `Label`, not a component. It is static:
//! no hover, no press. Markup: `span.ds-key-equivalent[data-style][data-size]`, in `Cap` holding
//! one `kbd.ds-key-equivalent-key` per key.

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{GlyphKind, Shortcut};
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

/// How the shortcut is drawn, `data-style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum KeyStyle {
    /// The glyphs in a row in the secondary ink at the size of the words beside them, no key
    /// caps: a menu item's trailing key equivalent, Spotlight's hint.
    #[default]
    Text,
    /// Each key on its own cap, in the code face: a keyboard legend, a hover card's key.
    Cap,
}

/// `shortcut` in `style`. `size` sets a cap's type (Mini for a hint row, Small, Regular); a text
/// run takes its surroundings' size. An empty shortcut draws nothing.
#[component]
pub fn KeyEquivalent(
    shortcut: Shortcut,
    #[props(default)] style: KeyStyle,
    #[props(default)] size: ControlSize,
    #[props(default)] common: Common,
) -> Element {
    let keys = shortcut.keys();
    if keys.is_empty() {
        return rsx! {};
    }
    let class = common.class("ds-key-equivalent");
    let data = common.data_attributes();
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-style": style.slug(),
            "data-size": size.slug(),
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            match style {
                KeyStyle::Text => rsx! { "{shortcut.glyphs()}" },
                KeyStyle::Cap => rsx! {
                    for key in keys {
                        kbd {
                            class: "ds-key-equivalent-key",
                            "data-glyph": key.glyph_kind().map(GlyphKind::slug),
                            "{key.glyph()}"
                        }
                    }
                },
            }
        }
    }
}
