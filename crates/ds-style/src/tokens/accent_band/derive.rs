//! The one function every accent comes from.

use super::band::{AccentBand, AccentPick, InkRule, SchemeBand};
use super::floors;
use super::grounds::{card_grounds, contrast, least};
use super::roles::AccentRoles;
use super::text_grounds::{TextGround, TextOn, text_grounds};
use crate::appearance::theme::Scheme;
use crate::tokens::hex::{Alpha, Hex};
use ds_core::colour::{
    fit::oklch_bytes,
    oklab::{Oklab, Oklch},
    srgb::Srgb,
};

/// The hue and chroma one accent is drawn in; only lightness moves from here.
#[derive(Debug, Clone, Copy)]
struct Tone {
    chroma: f64,
    hue: f64,
}

impl Tone {
    fn at(self, lightness: f64) -> Hex {
        Hex(oklch_bytes(lightness, self.chroma, self.hue))
    }
}

/// The roles `pick` paints in `scheme` inside `band`.
///
/// The fill starts at the band's lightness and steps (0.01) toward its ink until the ink reads
/// on it; the wash's alpha rises (0.01) only until it shows on every card ground; each text
/// accent starts at the band's text lightness and steps away from the card until it reads on
/// every ground in its [`text_grounds`] (the card's text on the card and the wash over it, the
/// material's on those and on the text-carrying materials over black and white and the wash
/// over them); the ring's alpha rises (0.05) until the card's text at it stands off every card
/// ground. Chroma is the band's at `pick.weight`, fitted to sRGB at each lightness.
pub fn accent_roles(band: &AccentBand, pick: AccentPick, scheme: Scheme) -> AccentRoles {
    let bounds = band.scheme(scheme);
    let tone = Tone {
        chroma: bounds.chroma.at(pick.weight),
        hue: pick.hue.degrees(),
    };
    let (fill, ink) = solid_fill(bounds, tone);
    roles_around(bounds, tone, scheme, (fill, ink), None)
}

/// The roles of the Mac Look's accent: `fill` is the system colour itself, its ink white when
/// white reads on it at [`floors::INK_ON_SYSTEM_FILL`] (Apple's own blue button is 4.0:1) and the
/// deep ink of the hue otherwise (macOS's rule for yellow); the text accent, wash and ring are
/// derived around it. The text accents lead from `text_lead` (the hue's increased-contrast
/// value) and are it exactly wherever it reads on every ground; where it does not, they step
/// from its lightness, in its own hue and chroma, until they do.
pub fn system_roles(band: &AccentBand, scheme: Scheme, fill: Hex, text_lead: Hex) -> AccentRoles {
    let bounds = band.scheme(scheme);
    let own = Oklch::from(Oklab::from(Srgb::from(fill)));
    let tone = Tone {
        chroma: own.c,
        hue: own.h.to_degrees().rem_euclid(360.0),
    };
    let white = Hex([0xFF, 0xFF, 0xFF]);
    let ink = if contrast(white, fill) >= floors::INK_ON_SYSTEM_FILL {
        white
    } else {
        tone_ink(tone)
    };
    roles_around(bounds, tone, scheme, (fill, ink), Some(text_lead))
}

/// The text accent, wash and ring around a chosen fill and ink, the text leading from `lead`
/// when there is one.
fn roles_around(
    bounds: &SchemeBand,
    tone: Tone,
    scheme: Scheme,
    (fill, ink): (Hex, Hex),
    lead: Option<Hex>,
) -> AccentRoles {
    let grounds = card_grounds(scheme);
    let wash = wash_alpha(bounds.wash, fill, &grounds);
    let text_on = |on| {
        let grounds = text_grounds(on, scheme, fill, wash);
        match lead {
            Some(lead) => text_from(lead, scheme, &grounds),
            None => text_accent(bounds, tone, scheme, &grounds),
        }
    };
    let text = text_on(TextOn::Card);
    AccentRoles {
        fill,
        ink,
        text,
        text_material: text_on(TextOn::Material),
        wash,
        ring: ring_alpha(bounds.ring, text, &grounds),
    }
}

/// The fill and its ink.
fn solid_fill(bounds: &SchemeBand, tone: Tone) -> (Hex, Hex) {
    let (ink, step) = match bounds.ink {
        InkRule::Deep => (tone_ink(tone), floors::LIGHTNESS_STEP),
    };
    let lightness = walk(bounds.fill.fraction(), step, |lightness| {
        contrast(ink, tone.at(lightness)) >= floors::TEXT
    });
    (tone.at(lightness), ink)
}

/// The near-black of the accent's hue that [`InkRule::Deep`] writes.
fn tone_ink(tone: Tone) -> Hex {
    Hex(oklch_bytes(0.22, 0.03, tone.hue))
}

/// The text accent that leads from `lead`: `lead` itself where it reads on every one of
/// `grounds`, else its own hue and chroma stepped away from the card from its lightness.
fn text_from(lead: Hex, scheme: Scheme, grounds: &[TextGround]) -> Hex {
    let reads = |text: Hex| {
        grounds
            .iter()
            .all(|ground| contrast(text, ground.hex) >= floors::TEXT)
    };
    if reads(lead) {
        return lead;
    }
    let own = Oklch::from(Oklab::from(Srgb::from(lead)));
    let tone = Tone {
        chroma: own.c,
        hue: own.h.to_degrees().rem_euclid(360.0),
    };
    let step = match scheme {
        Scheme::Light => -floors::LIGHTNESS_STEP,
        Scheme::Dark => floors::LIGHTNESS_STEP,
    };
    tone.at(walk(own.l, step, |lightness| reads(tone.at(lightness))))
}

/// The text accent: darker than the band's start in light, lighter in dark, until it reads on
/// every one of `grounds`.
fn text_accent(bounds: &SchemeBand, tone: Tone, scheme: Scheme, grounds: &[TextGround]) -> Hex {
    let step = match scheme {
        Scheme::Light => -floors::LIGHTNESS_STEP,
        Scheme::Dark => floors::LIGHTNESS_STEP,
    };
    let lightness = walk(bounds.text.fraction(), step, |lightness| {
        let text = tone.at(lightness);
        grounds
            .iter()
            .all(|ground| contrast(text, ground.hex) >= floors::TEXT)
    });
    tone.at(lightness)
}

/// From `start`, `step` by step until `passes` or the band's lightness bounds.
fn walk(start: f64, step: f64, passes: impl Fn(f64) -> bool) -> f64 {
    let inside = |lightness: f64| (floors::DARKEST..=floors::LIGHTEST).contains(&lightness);
    let mut lightness = start;
    while !passes(lightness) && inside(lightness + step) {
        lightness += step;
    }
    lightness
}

/// The wash's alpha: the band's, raised until the wash shows on every ground.
fn wash_alpha(start: Alpha, fill: Hex, grounds: &[Hex]) -> Alpha {
    rise(start, floors::WASH_STEP, floors::WASH_MOST, |alpha| {
        least(|ground| fill.over(alpha, ground), grounds) >= floors::WASH_SHOWS
    })
}

/// The ring's alpha: the band's, raised until the ring stands off every ground.
fn ring_alpha(start: Alpha, text: Hex, grounds: &[Hex]) -> Alpha {
    rise(start, floors::RING_STEP, Alpha(1000), |alpha| {
        least(|ground| text.over(alpha, ground), grounds) >= floors::RING
    })
}

/// From `start`, `step` by step up to `most` until `passes`.
fn rise(start: Alpha, step: Alpha, most: Alpha, passes: impl Fn(Alpha) -> bool) -> Alpha {
    let mut alpha = start.min(most);
    while !passes(alpha) && alpha < most {
        alpha = Alpha((alpha.0 + step.0).min(most.0));
    }
    alpha
}
