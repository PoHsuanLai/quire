//! The glyph that follows the level (the user's brief of 2026-09-25). It is drawn as stacked
//! layers, one `svg` per part, so each part can come and go by opacity over `--t-quick`, a
//! cross-fade Blitz paints (an SVG's own CSS does not animate there, spike S6): the speaker's
//! body with three waves shown by thirds and a slash when muted, and the sun's core with its
//! rays, which grow with the brightness (scaled by `--f` in `level.css`).
//!
//! Every part is a filled shape, no stroke (design/08-ICONS.md section 1.2): the speaker body is
//! Lucide `volume`'s path, the sun's core a disc and its eight rays capsules, the waves three
//! annular arcs on one centre (10, 12) at radii 5, 8.25 and 11.5 with a gap between them, and the
//! slash a solid bar. A muted speaker's body has the gap the slash leaves cut out of it.

use super::solid::{CORE, KEYBOARD_BODY, KEYBOARD_CORE, KEYBOARD_RAYS, RAYS};
use super::vocab::LevelGlyph;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::vocab::Muting;
use ds_core::word::Word;
use ds_style::icon::posed::Thousandths;
use ds_style::icon::render::IconSize;
use ds_style::icon::shape::Shape;
use ds_style::icon::slash::{Cut, SLASH_SHAPE};
use ds_style::icon::solid_fan::{SPEAKER, WAVE_1, WAVE_2, WAVE_3};

const WAVES: [&[Shape]; 3] = [&[WAVE_1], &[WAVE_2], &[WAVE_3]];
const BODY: &[Shape] = &[SPEAKER];
const SLASH: &[Shape] = &[SLASH_SHAPE];

/// One layer of a level glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum Part {
    Body,
    #[word(slug = "wave-1")]
    Wave1,
    #[word(slug = "wave-2")]
    Wave2,
    #[word(slug = "wave-3")]
    Wave3,
    Slash,
    Core,
    Rays,
    KeyBody,
    KeyCore,
    KeyRays,
}

impl Part {
    fn shapes(self) -> &'static [Shape] {
        match self {
            Part::Body => BODY,
            Part::Wave1 => WAVES[0],
            Part::Wave2 => WAVES[1],
            Part::Wave3 => WAVES[2],
            Part::Slash => SLASH,
            Part::Core => CORE,
            Part::Rays => RAYS,
            Part::KeyBody => KEYBOARD_BODY,
            Part::KeyCore => KEYBOARD_CORE,
            Part::KeyRays => KEYBOARD_RAYS,
        }
    }
}

/// Whether a part is showing: `data-on`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum Showing {
    On,
    Off,
}

/// The parts `glyph` draws, in paint order.
pub(crate) fn parts(glyph: LevelGlyph) -> &'static [Part] {
    match glyph {
        LevelGlyph::Volume(_) => &[
            Part::Body,
            Part::Wave1,
            Part::Wave2,
            Part::Wave3,
            Part::Slash,
        ],
        LevelGlyph::Brightness => &[Part::Core, Part::Rays],
        LevelGlyph::KeyboardBrightness => &[Part::KeyBody, Part::KeyCore, Part::KeyRays],
    }
}

/// How many waves a heard speaker shows at `value`: none at 0, then one, two and three by
/// thirds of the range.
pub(crate) fn waves(value: Fraction) -> u8 {
    match value.clamped().0 {
        0 => 0,
        1..=333 => 1,
        334..=666 => 2,
        _ => 3,
    }
}

/// Whether `part` of `glyph` shows at `value`.
pub(crate) fn showing(part: Part, glyph: LevelGlyph, value: Fraction) -> Showing {
    let on = match (part, glyph) {
        (
            Part::Body | Part::Core | Part::Rays | Part::KeyBody | Part::KeyCore | Part::KeyRays,
            _,
        ) => true,
        (Part::Slash, LevelGlyph::Volume(muting)) => muting == Muting::Muted,
        (Part::Wave1 | Part::Wave2 | Part::Wave3, LevelGlyph::Volume(Muting::Muted)) => false,
        (Part::Wave1, LevelGlyph::Volume(Muting::Audible)) => waves(value) >= 1,
        (Part::Wave2, LevelGlyph::Volume(Muting::Audible)) => waves(value) >= 2,
        (Part::Wave3, LevelGlyph::Volume(Muting::Audible)) => waves(value) >= 3,
        (
            Part::Slash | Part::Wave1 | Part::Wave2 | Part::Wave3,
            LevelGlyph::Brightness | LevelGlyph::KeyboardBrightness,
        ) => false,
    };
    if on { Showing::On } else { Showing::Off }
}

/// How much of the slash's gap is cut out of `part`: all of it from a muted speaker's body.
fn cut(part: Part, glyph: LevelGlyph) -> Thousandths {
    match (part, glyph) {
        (Part::Body, LevelGlyph::Volume(Muting::Muted)) => Thousandths(1000),
        _ => Thousandths::default(),
    }
}

/// `glyph` at `value`, `size`: a `span.ds-level-glyph` of stacked parts in `currentColor`.
#[component]
pub(crate) fn LevelGlyphView(glyph: LevelGlyph, value: Fraction, size: IconSize) -> Element {
    let px = size.px();
    let mask = use_hook(|| format!("ds-lcut-{}", current_scope_id().0));
    rsx! {
        span { class: "ds-level-glyph", "data-size": "{px}", "aria-hidden": "true",
            for part in parts(glyph).iter().copied() {
                svg {
                    key: "{part.slug()}",
                    class: "ds-ic ds-level-part",
                    "data-part": part.slug(),
                    "data-on": showing(part, glyph, value).slug(),
                    width: "{px}",
                    height: "{px}",
                    view_box: "0 0 24 24",
                    "stroke": "none",
                    "fill": "currentColor",
                    "fill-rule": "evenodd",
                    Cut { id: mask.clone(), drawn: cut(part, glyph),
                        for shape in part.shapes() {
                            {shape_child(shape)}
                        }
                    }
                }
            }
        }
    }
}

fn shape_child(shape: &Shape) -> Element {
    match shape {
        Shape::Path(d) => rsx! { path { d: "{d}" } },
        Shape::Solid(d) => rsx! { path { d: "{d}", fill: "currentColor" } },
        Shape::Circle { cx, cy, r } => rsx! { circle { cx: "{cx}", cy: "{cy}", r: "{r}" } },
        Shape::Rect {
            x,
            y,
            width,
            height,
            rx,
        } => rsx! {
            rect { x: "{x}", y: "{y}", width: "{width}", height: "{height}", rx: "{rx}" }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{Part, Shape, Showing, showing, waves};
    use crate::components::content::level_glyph::vocab::LevelGlyph;
    use ds_core::vocab::Fraction;
    use ds_core::vocab::Muting;

    #[test]
    fn the_waves_follow_the_level_and_mute_brings_the_slash() {
        let heard = LevelGlyph::Volume(Muting::Audible);
        let muted = LevelGlyph::Volume(Muting::Muted);
        assert_eq!(
            [0, 1, 333, 334, 667, 1000].map(|v| waves(Fraction(v))),
            [0, 1, 1, 2, 3, 3]
        );
        assert_eq!(showing(Part::Wave3, heard, Fraction(400)), Showing::Off);
        assert_eq!(showing(Part::Wave2, heard, Fraction(400)), Showing::On);
        assert_eq!(showing(Part::Slash, heard, Fraction(400)), Showing::Off);
        assert_eq!(showing(Part::Slash, muted, Fraction(400)), Showing::On);
        assert_eq!(showing(Part::Wave1, muted, Fraction(1000)), Showing::Off);
        assert_eq!(
            showing(Part::Rays, LevelGlyph::Brightness, Fraction(0)),
            Showing::On
        );
    }

    #[test]
    fn the_keyboard_light_is_a_keyboard_under_a_rising_sun_whose_rays_follow() {
        let glyph = LevelGlyph::KeyboardBrightness;
        assert_eq!(
            super::parts(glyph),
            &[Part::KeyBody, Part::KeyCore, Part::KeyRays][..]
        );
        assert_eq!(Part::KeyRays.shapes().len(), 5);
        for part in super::parts(glyph) {
            assert_eq!(showing(*part, glyph, Fraction(0)), Showing::On, "{part:?}");
        }
    }

    #[test]
    fn the_sun_is_a_disc_and_eight_rays_and_every_part_is_filled_geometry() {
        assert_eq!(Part::Core.shapes().len(), 1);
        assert_eq!(Part::Rays.shapes().len(), 8);
        assert_eq!(Part::Body.shapes().len(), 1);
        for part in [
            Part::Body,
            Part::Wave1,
            Part::Wave2,
            Part::Wave3,
            Part::Slash,
            Part::Rays,
        ] {
            assert!(
                part.shapes()
                    .iter()
                    .all(|shape| matches!(shape, Shape::Solid(_))),
                "{part:?}"
            );
        }
    }
}
