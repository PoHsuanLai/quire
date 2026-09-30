//! A Space's colours, derived from its dots.
//!
//! Pure: dots and a scheme in, hex and the occasional `rgba(...)` out. No file
//! and no clock. The arithmetic is the approved mockup's, including the steps
//! it takes when a colour would leave sRGB or fail its contrast floor
//! (design/03-COLOR.md section 4).

use crate::appearance::theme::Scheme;
use crate::tokens::accent_band::{
    band::{AccentPick, BandWeight, Hue},
    derive::accent_roles,
    picked::BAND,
    roles::AccentRoles,
};
use ds_core::colour::contrast::ratio;
use ds_core::colour::fit::{js_round, oklch_hex as hex};
use serde::{Deserialize, Serialize};

pub(crate) mod card;
pub mod readout;

/// Whether deriving a palette had to lower a stop's chroma to keep the frame's text legible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capping {
    /// Every stop passed at the chroma the person chose.
    Uncapped,
    /// At least one stop's chroma was lowered so the text on it stays legible.
    Capped,
}

/// One colour the person placed.
///
/// `chroma` is a fraction of the most the frame will show, not an OKLCH chroma
/// on its own. Lightness belongs to the theme; only these two are chosen.
///
/// `f32` has no `Eq`, so dots compare with [`PartialEq`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dot {
    /// Position on the OKLCH wheel, in degrees.
    pub hue: f32,
    /// How much of the hue to use, from none to the frame's maximum.
    pub chroma: f32,
}

impl Default for Dot {
    fn default() -> Self {
        NEUTRAL_DOT
    }
}

/// The neutral dot an empty Space is read as: hue 250, a trace of chroma.
pub const NEUTRAL_DOT: Dot = Dot {
    hue: 250.0,
    chroma: 0.06,
};

/// The colours a Space paints: the frame behind the window, the text on it,
/// and the accent the card borrows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    /// One frame colour per dot, left to right across the gradient.
    pub stops: Vec<String>,
    /// The swatch drawn on the dot itself, at the pick's fixed lightness.
    pub picked: Vec<String>,
    /// Sidebar text.
    pub ink: String,
    /// Secondary sidebar text.
    pub soft: String,
    /// The quietest sidebar text: counts, meta.
    pub faint: String,
    /// A row under the pointer. Dark is the literal `rgba(255,255,255,.06)`.
    pub hover: String,
    /// A selected pill. Dark is `rgba(255,255,255,.10)`; light is white at .72.
    pub pill: String,
    /// The accent inside the card: the settled band's fill at the Space's hue, the first dot's
    /// chroma as its weight (design/03-COLOR.md section 20).
    pub accent: String,
    /// The accent's translucent wash, behind a selected row: `rgba(...)`, never a hex.
    pub accent_soft: String,
    /// Text drawn on top of [`Self::accent`].
    pub accent_ink: String,
    /// The accent as text or a thin mark on the card.
    pub accent_text: String,
    /// The focus ring: [`Self::accent_text`] at an alpha, `rgba(...)`.
    pub accent_ring: String,
    /// The same accent as typed roles, for a caller that measures or composites.
    pub accent_roles: AccentRoles,
    /// Whether a stop's chroma was lowered so the text on it stays legible.
    pub capped: Capping,
}

/// Lightness, the step to the next stop, and the chroma a fully saturated dot reaches.
struct Frame {
    lightness: f64,
    step: f64,
    chroma: f64,
}

/// Calibrated against palettes Arc exposed as `--arc-palette-*`.
///
/// A light peach Space has background `#F4EBE5` (L 0.945, C 0.013), title
/// `#2E1000` (L 0.22, C 0.06) and pick `#EF8C62` (C 0.13). A mint gradient
/// runs `#D2F3E5`→`#D2EBF3` (L 0.93, C 0.03–0.04). Dark is `#001E15` (L 0.21).
///
/// A light frame therefore sits at L 0.936 with chroma 0.052, and a dark one
/// at L 0.215 with chroma 0.042. Fitting a colour into sRGB steps chroma down
/// by [`GAMUT_STEP`] (0.002). The contrast cap steps it by [`CAP_STEP`] (0.003).
/// Those two steps are the approved mockup's; the accent's own steps are the band's
/// (`tokens::accent_band::floors`).
const FRAME_LIGHT: Frame = Frame {
    lightness: 0.936,
    step: -0.012,
    chroma: 0.052,
};

/// The dark frame. See [`FRAME_LIGHT`] for where the numbers come from.
const FRAME_DARK: Frame = Frame {
    lightness: 0.215,
    step: 0.014,
    chroma: 0.042,
};

/// Lightness of the swatch on a dot. The peach pick `#EF8C62` was C 0.13;
/// a fully saturated dot is drawn at L 0.74, C 0.15. See [`FRAME_LIGHT`].
const PICK_L: f64 = 0.74;
/// Lightness of the field's swatches on a dark field, where 0.74 glares. See [`PICK_L`].
const PICK_L_DARK: f64 = 0.66;
/// Chroma of a fully saturated pick. See [`PICK_L`].
const PICK_C: f64 = 0.15;

/// How far the contrast cap drops chroma when text would fail. See [`FRAME_LIGHT`].
const CAP_STEP: f64 = 0.003;

const HOVER_DARK: &str = "rgba(255,255,255,.06)";
const PILL_DARK: &str = "rgba(255,255,255,.10)";
const PILL_LIGHT: &str = "rgba(255,255,255,.72)";

/// Build a palette from a Space's dots.
///
/// An empty list is the neutral dot (hue 250, chroma 0.06), the same grey as
/// the last of the eight presets. `scheme` selects the light or the dark frame.
pub fn derive(dots: &[Dot], scheme: Scheme) -> Palette {
    let dark = scheme == Scheme::Dark;
    let neutral = [NEUTRAL_DOT];
    let dots = if dots.is_empty() { &neutral } else { dots };
    let frame = if dark { &FRAME_DARK } else { &FRAME_LIGHT };
    // Empty input was replaced with the neutral dot above.
    let first = dots.first().unwrap_or(&NEUTRAL_DOT);
    let hue0 = f64::from(first.hue);
    let k = f64::from(first.chroma);

    let ink = if dark {
        hex(0.93, 0.018 * k, hue0)
    } else {
        hex(0.22, 0.06 * k, hue0)
    };
    let soft = if dark {
        hex(0.80, 0.03 * k, hue0)
    } else {
        hex(0.40, 0.05 * k, hue0)
    };
    let faint = if dark {
        hex(0.66, 0.035 * k, hue0)
    } else {
        hex(0.52, 0.05 * k, hue0)
    };

    let mut capped = false;
    let stops = dots
        .iter()
        .enumerate()
        .map(|(index, dot)| {
            let lightness = frame.lightness + index as f64 * frame.step;
            let hue = f64::from(dot.hue);
            let mut chroma = f64::from(dot.chroma) * frame.chroma;
            let mut colour = hex(lightness, chroma, hue);
            while chroma > 0.0 && (below(&ink, &colour, 4.5) || below(&faint, &colour, 3.0)) {
                chroma -= CAP_STEP;
                capped = true;
                colour = hex(lightness, chroma, hue);
            }
            colour
        })
        .collect();

    let hover = if dark {
        HOVER_DARK.to_owned()
    } else {
        hex(0.885, 0.026 * k + 0.004, hue0)
    };
    let pill = if dark {
        PILL_DARK.to_owned()
    } else {
        PILL_LIGHT.to_owned()
    };

    let accent_roles = space_accent(hue0, k, scheme);
    let picked = dots
        .iter()
        .map(|dot| hex(PICK_L, f64::from(dot.chroma) * PICK_C, f64::from(dot.hue)))
        .collect();

    Palette {
        stops,
        picked,
        ink,
        soft,
        faint,
        hover,
        pill,
        accent: accent_roles.fill.css(),
        accent_soft: accent_roles.wash_colour().css(),
        accent_ink: accent_roles.ink.css(),
        accent_text: accent_roles.text.css(),
        accent_ring: accent_roles.ring_colour().css(),
        accent_roles,
        capped: if capped {
            Capping::Capped
        } else {
            Capping::Uncapped
        },
    }
}

/// The card accent a Space lends: the settled band at its first dot's hue, weighted by that
/// dot's chroma, so a grey Space lends a grey-blue (design/03-COLOR.md section 20). The one
/// derivation every accent shares; it replaced mailo's `oklch(aL, .045 + .035k, h0)`.
fn space_accent(hue: f64, chroma: f64, scheme: Scheme) -> AccentRoles {
    // A dot's hue is 0..360 and its chroma 0..1; both land in range after the clamp.
    let degrees = hue.rem_euclid(360.0).round() as u16 % 360;
    let weight = (chroma.clamp(0.0, 1.0) * 1000.0).round() as u16;
    let pick = AccentPick {
        hue: Hue(degrees),
        weight: BandWeight(weight),
    };
    accent_roles(&BAND, pick, scheme)
}

/// The colour one point of the editor's hue × chroma field is drawn in.
///
/// The pick's own lightness: L 0.74 on a light field and 0.66 on a dark one, at up to the
/// pick's chroma. The editor draws the field from this and computes no colour of its own.
pub fn swatch(dot: Dot, scheme: Scheme) -> String {
    let lightness = match scheme {
        Scheme::Light => PICK_L,
        Scheme::Dark => PICK_L_DARK,
    };
    hex(
        lightness,
        f64::from(dot.chroma) * PICK_C,
        f64::from(dot.hue),
    )
}

/// The frame's background, as a CSS `linear-gradient`.
///
/// One stop is repeated, so a single-colour Space is still a gradient. Several
/// stops are spread from 0% to 100%.
pub fn gradient(palette: &Palette) -> String {
    let stops = &palette.stops;
    if stops.len() <= 1 {
        let colour = stops.first().map(String::as_str).unwrap_or("");
        return format!("linear-gradient(135deg,{colour},{colour})");
    }
    let last = stops.len() - 1;
    let body = stops
        .iter()
        .enumerate()
        .map(|(index, colour)| {
            let position = js_round(index as f64 / last as f64 * 100.0);
            // A stop's position is a whole number of percent.
            let position = position as i64;
            format!("{colour} {position}%")
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("linear-gradient(135deg,{body})")
}

fn below(fore: &str, back: &str, need: f64) -> bool {
    match ratio(fore, back) {
        Some(measured) => measured < need,
        // Not a hex pair: treat it as failing, the same as a ratio under the floor.
        None => true,
    }
}

#[cfg(test)]
mod tests;
