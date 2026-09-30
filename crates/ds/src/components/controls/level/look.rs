//! Drawing the three looks (`vocab.rs`, `LevelLook`). Each is the level as `--f` on the root and
//! plain boxes the stylesheet sizes: the capsule's well and fill with the glyph in both (the
//! fill's copy clipped by the fill, which is how the glyph is knocked out without a blend mode),
//! the knob riding the fill's end, or sixteen squares.

use crate::components::content::level_glyph::glyph::LevelGlyphView;
use crate::components::content::level_glyph::vocab::{LevelGlyph, LevelLook};
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_style::icon::render::IconSize;

/// How many squares the segmented look draws: a volume key's sixteen steps.
pub(crate) const SEGMENTS: u16 = 16;

/// What one render draws.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Drawn {
    pub look: LevelLook,
    pub value: Fraction,
    pub glyph: LevelGlyph,
    /// The level the glyph's parts follow: the value, or a volume state's band.
    pub glyph_level: Fraction,
}

/// The rail's contents for `drawn.look`.
pub(crate) fn body(drawn: Drawn) -> Element {
    match drawn.look {
        LevelLook::Capsule => capsule(&drawn, Inside::Glyph),
        LevelLook::CapsuleKnob => rsx! {
            {capsule(&drawn, Inside::Nothing)}
            div { class: "ds-level-knob" }
        },
        LevelLook::Segments => segments(drawn.value),
    }
}

/// Whether the capsule carries the glyph inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inside {
    Glyph,
    Nothing,
}

fn capsule(drawn: &Drawn, inside: Inside) -> Element {
    let glyph = || match inside {
        Inside::Glyph => rsx! {
            span { class: "ds-level-in",
                LevelGlyphView { glyph: drawn.glyph, value: drawn.glyph_level, size: IconSize::Base }
            }
        },
        Inside::Nothing => rsx! {},
    };
    rsx! {
        div { class: "ds-level-track",
            {glyph()}
            div { class: "ds-level-fill", {glyph()} }
        }
    }
}

/// How many squares `value` fills.
pub(crate) fn filled(value: Fraction) -> u16 {
    (value.clamped().0 * SEGMENTS + 500) / 1000
}

/// The squares, each `data-on` when filled.
fn segments(value: Fraction) -> Element {
    let now = filled(value);
    rsx! {
        div { class: "ds-level-segments",
            for index in 0..SEGMENTS {
                div {
                    key: "{index}",
                    class: "ds-level-seg",
                    "data-on": if index < now { "on" } else { "off" },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filled;
    use ds_core::vocab::Fraction;

    #[test]
    fn a_level_fills_the_nearest_sixteenth() {
        assert_eq!(
            [0, 30, 32, 400, 1000].map(|v| filled(Fraction(v))),
            [0, 0, 1, 6, 16]
        );
    }
}
