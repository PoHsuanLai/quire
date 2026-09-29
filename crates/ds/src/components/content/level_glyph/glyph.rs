//! The glyph that follows the level (the user's brief of 2026-09-25). It is drawn as stacked
//! layers, one `svg` per part, so each part can come and go by opacity over `--t-quick`, a
//! cross-fade Blitz paints (an SVG's own CSS does not animate there, spike S6): the speaker's
//! body with three waves shown by thirds and a slash when muted, and the sun's core with its
//! rays, which grow with the brightness (scaled by `--f` in `level.css`).
//!
//! The geometry is Lucide's: the speaker body is `volume`'s, the sun `sun`'s core and eight rays,
//! the slash `volume-off`'s. Lucide draws at most two waves; the three here are arcs on one
//! centre (10, 12) at radii 5, 8.25 and 11.5, spaced so a 2-unit stroke leaves a gap between them.

use super::vocab::{LevelGlyph, Muting};
use crate::core::vocab::Fraction;
use crate::core::word::Word;
use crate::style::icon::Icon;
use crate::style::icon::render::IconSize;
use crate::style::icon::shape::Shape;
use crate::style::icon::stroke::stroke_width;
use crate::style::scale::use_scale;
use dioxus::prelude::*;

const WAVE_1: &[Shape] = &[Shape::Path("M13.83 8.79a5 5 0 0 1 0 6.42")];
const WAVE_2: &[Shape] = &[Shape::Path("M16.32 6.7a8.25 8.25 0 0 1 0 10.6")];
const WAVE_3: &[Shape] = &[Shape::Path("M18.81 4.61a11.5 11.5 0 0 1 0 14.78")];
const SLASH: &[Shape] = &[Shape::Path("M2 2l20 20")];

// The keyboard-brightness glyph (design/26), on Lucide's grid: Lucide `keyboard`'s body
// lowered and shortened to the grid's bottom half (y 12 to 22), a row of keys and the space bar,
// and above it a sun rising from behind (a half disc of radius 3 on (12, 9)) with five rays at
// 4.5 to 6 units out, which grow with the level as the display sun's do.
const KEYBOARD_BODY: &[Shape] = &[
    Shape::Rect {
        x: "2",
        y: "12",
        width: "20",
        height: "10",
        rx: "2",
    },
    Shape::Path("M6 16h.01"),
    Shape::Path("M10 16h.01"),
    Shape::Path("M14 16h.01"),
    Shape::Path("M18 16h.01"),
    Shape::Path("M8 19h8"),
];
const KEYBOARD_CORE: &[Shape] = &[Shape::Path("M9 9a3 3 0 0 1 6 0")];
const KEYBOARD_RAYS: &[Shape] = &[
    Shape::Path("M12 4.5V3"),
    Shape::Path("m15.18 5.82 1.06-1.06"),
    Shape::Path("m8.82 5.82-1.06-1.06"),
    Shape::Path("M16.5 9H18"),
    Shape::Path("M7.5 9H6"),
];

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
        let sun = Icon::Sun.shapes();
        match self {
            Part::Body => Icon::Volume.shapes(),
            Part::Wave1 => WAVE_1,
            Part::Wave2 => WAVE_2,
            Part::Wave3 => WAVE_3,
            Part::Slash => SLASH,
            Part::Core => &sun[..1],
            Part::Rays => &sun[1..],
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

/// `glyph` at `value`, `size`: a `span.ds-level-glyph` of stacked parts in `currentColor`.
#[component]
pub(crate) fn LevelGlyphView(glyph: LevelGlyph, value: Fraction, size: IconSize) -> Element {
    let px = size.px();
    let stroke = stroke_width(size, use_scale());
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
                    "stroke": "currentColor",
                    "stroke-width": stroke.clone(),
                    "stroke-linecap": "round",
                    "stroke-linejoin": "round",
                    "fill": "none",
                    for shape in part.shapes() {
                        {shape_child(shape)}
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
    use super::{Part, Showing, showing, waves};
    use crate::components::content::level_glyph::vocab::{LevelGlyph, Muting};
    use crate::core::vocab::Fraction;

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
    fn the_sun_is_lucides_core_and_eight_rays() {
        assert_eq!(Part::Core.shapes().len(), 1);
        assert_eq!(Part::Rays.shapes().len(), 8);
        assert_eq!(Part::Body.shapes().len(), 1);
    }
}
