//! Drawing a level (design/30 sections 2.1 and 2.9): the capsule, the capsule with a knob, or
//! sixteen squares, for the capsule `Slider` and the `LevelIndicator` alike. Each is the level as
//! `--f` on the root and plain boxes the stylesheet sizes: the capsule's well and fill with the
//! glyph in both (the fill's copy clipped by the fill, which is how the glyph is knocked out
//! without a blend mode), the knob riding the fill's end, or sixteen squares.

use crate::components::content::level_glyph::glyph::LevelGlyphView;
use crate::components::content::level_glyph::vocab::LevelGlyph;
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_style::icon::render::IconSize;

/// How many squares the discrete look draws: a volume key's sixteen steps.
pub(crate) const SEGMENTS: u16 = 16;

/// The class names of one component's parts, so the two share one drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Parts {
    pub track: &'static str,
    pub fill: &'static str,
    pub knob: &'static str,
    pub icon: &'static str,
    pub segments: &'static str,
    pub segment: &'static str,
}

/// The slider's parts.
pub(crate) const SLIDER: Parts = Parts {
    track: "ds-slider-track",
    fill: "ds-slider-fill",
    knob: "ds-slider-thumb",
    icon: "ds-slider-icon",
    segments: "ds-slider-segments",
    segment: "ds-slider-segment",
};

/// The level indicator's parts.
pub(crate) const INDICATOR: Parts = Parts {
    track: "ds-level-indicator-track",
    fill: "ds-level-indicator-fill",
    knob: "ds-level-indicator-thumb",
    icon: "ds-level-indicator-icon",
    segments: "ds-level-indicator-segments",
    segment: "ds-level-indicator-segment",
};

/// What is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shape {
    /// A capsule with the glyph inside at its left end.
    Capsule,
    /// The capsule with a round knob at the fill's end; the glyph is drawn before it.
    CapsuleKnob,
    /// Sixteen squares that fill in one by one; the glyph is drawn before them.
    Segments,
}

impl Shape {
    /// Whether the glyph stands before the rail rather than inside the capsule.
    pub(crate) fn glyph_leads(self) -> bool {
        match self {
            Shape::Capsule => false,
            Shape::CapsuleKnob | Shape::Segments => true,
        }
    }
}

/// The glyph a level carries and the level its parts follow: the value, or a volume state's band.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlyphAt {
    pub glyph: LevelGlyph,
    pub level: Fraction,
}

/// What one render draws.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Drawn {
    pub shape: Shape,
    pub parts: Parts,
    pub value: Fraction,
    pub glyph: Option<GlyphAt>,
}

/// The rail's contents for `drawn.shape`.
pub(crate) fn body(drawn: Drawn) -> Element {
    match drawn.shape {
        Shape::Capsule => capsule(&drawn, Inside::Glyph),
        Shape::CapsuleKnob => rsx! {
            {capsule(&drawn, Inside::Nothing)}
            div { class: drawn.parts.knob }
        },
        Shape::Segments => segments(&drawn),
    }
}

/// The glyph before the rail, drawn by the shapes that lead with it.
pub(crate) fn lead(drawn: &Drawn) -> Option<Element> {
    let at = drawn.glyph.filter(|_| drawn.shape.glyph_leads())?;
    Some(rsx! {
        span { class: "{drawn.parts.icon} {drawn.parts.icon}-lead",
            LevelGlyphView { glyph: at.glyph, value: at.level, size: IconSize::Bar }
        }
    })
}

/// Whether the capsule carries the glyph inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inside {
    Glyph,
    Nothing,
}

fn capsule(drawn: &Drawn, inside: Inside) -> Element {
    let glyph = || match (inside, drawn.glyph) {
        (Inside::Glyph, Some(at)) => rsx! {
            span { class: drawn.parts.icon,
                LevelGlyphView { glyph: at.glyph, value: at.level, size: IconSize::Base }
            }
        },
        (Inside::Glyph, None) | (Inside::Nothing, _) => rsx! {},
    };
    rsx! {
        div { class: drawn.parts.track,
            {glyph()}
            div { class: drawn.parts.fill, {glyph()} }
        }
    }
}

/// How many squares `value` fills.
pub(crate) fn filled(value: Fraction) -> u16 {
    (value.clamped().0 * SEGMENTS + 500) / 1000
}

/// The squares, each `data-on` when filled.
fn segments(drawn: &Drawn) -> Element {
    let now = filled(drawn.value);
    rsx! {
        div { class: drawn.parts.segments,
            for index in 0..SEGMENTS {
                div {
                    key: "{index}",
                    class: drawn.parts.segment,
                    "data-on": if index < now { "on" } else { "off" },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Shape, filled};
    use ds_core::vocab::Fraction;

    #[test]
    fn a_level_fills_the_nearest_sixteenth() {
        assert_eq!(
            [0, 30, 32, 400, 1000].map(|v| filled(Fraction(v))),
            [0, 0, 1, 6, 16]
        );
    }

    #[test]
    fn only_the_capsule_holds_its_glyph_inside() {
        assert!(!Shape::Capsule.glyph_leads());
        assert!(Shape::CapsuleKnob.glyph_leads());
        assert!(Shape::Segments.glyph_leads());
    }
}
