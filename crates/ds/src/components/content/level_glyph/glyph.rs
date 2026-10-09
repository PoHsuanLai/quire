//! The glyph that follows the level (the user's brief of 2026-09-25). It is drawn as stacked
//! layers, one `svg` per part, so each part can come and go by opacity over `--t-quick`, a
//! cross-fade Blitz paints (an SVG's own CSS does not animate there, spike S6): the speaker's
//! body with three waves shown by thirds and a slash when muted, and the sun's core with its
//! rays, which grow with the brightness (scaled by `--f` in `level.css`).
//!
//! Every part but the slash is stroked (design/08-ICONS.md section 1.2), Lucide's 2 unit stroke
//! with round caps and joins as on the Bluetooth glyph: the speaker body is Lucide `volume`'s
//! path, the sun's core a circle and its eight rays lines, the waves three arcs on one centre
//! (11, 12) at radii 4.5, 7.5 and 10.5. The slash stays a solid bar. A muted speaker's body has
//! the gap the slash leaves cut out of it.

use super::stroked::{KEYBOARD_BODY, KEYBOARD_CORE, KEYBOARD_RAYS, WAVE_1, WAVE_2, WAVE_3};
use super::stroked::{body, core, rays};
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
use ds_style::icon::stroke::stroke_width;
use ds_style::scale::use_scale;

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
    /// Whether the part is drawn with a stroke: all but the solid slash bar.
    fn is_stroked(self) -> bool {
        self != Part::Slash
    }

    fn shapes(self) -> &'static [Shape] {
        match self {
            Part::Body => body(),
            Part::Wave1 => WAVE_1,
            Part::Wave2 => WAVE_2,
            Part::Wave3 => WAVE_3,
            Part::Slash => SLASH,
            Part::Core => core(),
            Part::Rays => rays(),
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
    let stroke = stroke_width(size, use_scale());
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
                    "stroke": if part.is_stroked() { "currentColor" } else { "none" },
                    "stroke-width": if part.is_stroked() { stroke.clone() } else { String::new() },
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    "fill": if part.is_stroked() { "none" } else { "currentColor" },
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
    fn the_sun_is_a_disc_and_eight_rays_and_every_part_but_the_slash_is_stroked() {
        assert_eq!(Part::Core.shapes().len(), 1);
        assert_eq!(Part::Rays.shapes().len(), 8);
        assert_eq!(Part::Body.shapes().len(), 1);
        for part in [
            Part::Body,
            Part::Wave1,
            Part::Wave2,
            Part::Wave3,
            Part::Rays,
        ] {
            assert!(part.is_stroked(), "{part:?}");
            assert!(
                part.shapes()
                    .iter()
                    .all(|shape| matches!(shape, Shape::Path(_))),
                "{part:?}"
            );
        }
        assert!(!Part::Slash.is_stroked());
        assert!(matches!(Part::Slash.shapes(), [Shape::Solid(_)]));
    }
}
