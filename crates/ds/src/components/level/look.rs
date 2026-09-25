//! Drawing the three looks (`vocab.rs`, `LevelLook`). Each is the level as `--f` on the root and
//! plain boxes the stylesheet sizes: the capsule's well and fill with the glyph in both (the
//! fill's copy clipped by the fill, which is how the glyph is knocked out without a blend mode),
//! the knob riding the fill's end, or sixteen squares.

use super::glyph::LevelGlyphView;
use super::vocab::{LevelGlyph, LevelLook};
use crate::components::vocab::Fraction;
use crate::icon::render::IconSize;
use dioxus::prelude::*;

/// How many squares the segmented look draws: a volume key's sixteen steps.
pub(crate) const SEGMENTS: u16 = 16;

/// What one render draws.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Drawn {
    pub look: LevelLook,
    pub value: Fraction,
    /// The level the previous render showed: where the segments' stagger starts.
    pub before: Fraction,
    pub glyph: LevelGlyph,
    /// The tick's pulse class and alias, when it ticks.
    pub tick: Option<(String, &'static str)>,
}

/// The rail's contents for `drawn.look`.
pub(crate) fn body(drawn: Drawn) -> Element {
    match drawn.look {
        LevelLook::Capsule => capsule(&drawn, Inside::Glyph),
        LevelLook::CapsuleKnob => rsx! {
            {capsule(&drawn, Inside::Nothing)}
            div { class: "ds-level-knob" }
        },
        LevelLook::Segments => segments(drawn.value, drawn.before),
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
                LevelGlyphView { glyph: drawn.glyph, value: drawn.value, size: IconSize::Base }
            }
        },
        Inside::Nothing => rsx! {},
    };
    let edge = drawn.tick.clone().map(|(class, alias)| {
        rsx! { div { class: "ds-level-edge {class}", "data-pulse": alias } }
    });
    rsx! {
        div { class: "ds-level-track",
            {glyph()}
            div { class: "ds-level-fill",
                {glyph()}
                {edge}
            }
        }
    }
}

/// How many squares `value` fills.
pub(crate) fn filled(value: Fraction) -> u16 {
    (value.clamped().0 * SEGMENTS + 500) / 1000
}

/// The squares, each `data-on` when filled; the ones that change fill in (or empty) one
/// `--stagger` after another, counted from where the level was.
fn segments(value: Fraction, before: Fraction) -> Element {
    let (now, was) = (filled(value), filled(before));
    let order = move |index: u16| match now.cmp(&was) {
        std::cmp::Ordering::Greater if (was..now).contains(&index) => index - was,
        std::cmp::Ordering::Less if (now..was).contains(&index) => was - 1 - index,
        _ => 0,
    };
    rsx! {
        div { class: "ds-level-segments",
            for index in 0..SEGMENTS {
                div {
                    key: "{index}",
                    class: "ds-level-seg",
                    "data-on": if index < now { "on" } else { "off" },
                    style: "--i:{order(index)}",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filled;
    use crate::components::vocab::Fraction;

    #[test]
    fn a_level_fills_the_nearest_sixteenth() {
        assert_eq!(
            [0, 30, 32, 400, 1000].map(|v| filled(Fraction(v))),
            [0, 0, 1, 6, 16]
        );
    }
}
