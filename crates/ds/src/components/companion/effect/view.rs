//! The effect tag component.

use super::look::{effect_glyph, effect_words};
use dioxus::prelude::*;
use ds_core::vocab::EffectMark;
use ds_core::word::Word;
use ds_style::icon::render::{Glyph, IconSize};

/// An effect as a tag: its glyph and its words. The words are always written, so the mark never
/// rests on colour or shape alone.
#[component]
pub fn EffectTag(effect: EffectMark) -> Element {
    let words = effect_words(effect);
    rsx! {
        span { class: "ds-effect-tag", "data-effect": effect.slug(),
            Glyph { icon: effect_glyph(effect), size: IconSize::Compact }
            span { class: "ds-effect-tag-words", "{words}" }
        }
    }
}
