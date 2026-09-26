//! The four legibility floors the materials page reads out live: the card's ink over each
//! material's tint, translucent (`data-blur=on`) and solid (`off`), composited over the two worst
//! backdrops, black and white (design/21-SPACES.md section 7, `tests/legibility.rs` in ds).
//! `Material::Widget` alone is held to [`WIDGET_FLOOR`] rather than [`FLOOR`] (the contrast-relax
//! pass, 2026-09-26; see [`floor_for`]).

use ds::tokens::Alpha;
use ds::{ColourToken, Material, Scheme, Verdict, ratio, recipe};

/// WCAG AA for body text: what every floor has to clear.
pub const FLOOR: f64 = 4.5;

/// WCAG AA for large or bold text: `Material::Widget`'s own floor (the user's decision,
/// 2026-09-26, "relax the contrast then": design/03-COLOR.md section 17,
/// design/23-WIDGETS.md section 4.3). The widget's own text is large or bold, so it is held to
/// the large-text guideline rather than the small-text one every other material keeps.
pub const WIDGET_FLOOR: f64 = 3.0;

/// The floor `material` has to clear: [`WIDGET_FLOOR`] for `Material::Widget`, [`FLOOR`] for
/// everything else.
pub fn floor_for(material: Material) -> f64 {
    if material == Material::Widget {
        WIDGET_FLOOR
    } else {
        FLOOR
    }
}

/// What a panel is composited over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backdrop {
    /// Pure black.
    Black,
    /// Pure white.
    White,
}

impl Backdrop {
    fn rgb(self) -> [u8; 3] {
        match self {
            Backdrop::Black => [0, 0, 0],
            Backdrop::White => [255, 255, 255],
        }
    }

    /// What the readout calls it.
    pub fn label(self) -> &'static str {
        match self {
            Backdrop::Black => "black",
            Backdrop::White => "white",
        }
    }
}

/// Which tint is painted: over blur, or the solid fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tint {
    /// `--m-tint`, `data-blur=on`.
    OverBlur,
    /// `--m-tint-solid`, `data-blur=off`.
    Solid,
}

impl Tint {
    /// What the readout calls it.
    pub fn label(self) -> &'static str {
        match self {
            Tint::OverBlur => "blur on",
            Tint::Solid => "blur off",
        }
    }
}

/// One measured floor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Floor {
    /// Which tint.
    pub tint: Tint,
    /// Over what.
    pub backdrop: Backdrop,
    /// The ink's contrast ratio on the composited tint.
    pub measured: f64,
    /// What it has to clear: [`floor_for`] the material it was measured on.
    pub floor: f64,
}

impl Floor {
    /// Whether it clears [`Floor::floor`].
    pub fn verdict(&self) -> Verdict {
        if self.measured >= self.floor {
            Verdict::Pass
        } else {
            Verdict::Fail
        }
    }
}

/// The four floors of `material` in `scheme` at `tint_alpha`, or none for the window, which
/// paints the Space gradient rather than a tint.
pub fn floors(material: Material, scheme: Scheme, tint_alpha: Alpha) -> Vec<Floor> {
    let painted = recipe(material, scheme, tint_alpha);
    let ink = ColourToken::Ink.value(scheme).css();
    let floor = floor_for(material);
    let tints = [
        (Tint::OverBlur, painted.tint),
        (Tint::Solid, painted.tint_solid),
    ];
    tints
        .iter()
        .filter_map(|(tint, css)| rgba(css).map(|colour| (*tint, colour)))
        .flat_map(|(tint, colour)| {
            [Backdrop::Black, Backdrop::White].map(|backdrop| (tint, colour, backdrop))
        })
        .map(|(tint, colour, backdrop)| Floor {
            tint,
            backdrop,
            measured: ratio(&ink, &over(colour, backdrop.rgb())).unwrap_or(0.0),
            floor,
        })
        .collect()
}

/// A colour as `ds` writes one: `rgba(r,g,b,a)` or `#rrggbb` (alpha 1).
fn rgba(css: &str) -> Option<([u8; 3], f64)> {
    if let Some(hex) = ds::Hex::parse(css) {
        return Some((hex.0, 1.0));
    }
    let inner = css.strip_prefix("rgba(")?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
    let [r, g, b, a] = parts.as_slice() else {
        return None;
    };
    let channel = |text: &str| text.parse::<u8>().ok();
    Some((
        [channel(r)?, channel(g)?, channel(b)?],
        a.parse::<f64>().ok()?,
    ))
}

/// `colour` at its alpha over an opaque `backdrop`, rounded to 8 bits, as a hex.
fn over((rgb, alpha): ([u8; 3], f64), backdrop: [u8; 3]) -> String {
    let mix = |fore: u8, back: u8| {
        (f64::from(fore) * alpha + f64::from(back) * (1.0 - alpha))
            .round()
            .clamp(0.0, 255.0) as u8
    };
    ds::Hex([
        mix(rgb[0], backdrop[0]),
        mix(rgb[1], backdrop[1]),
        mix(rgb[2], backdrop[2]),
    ])
    .css()
}

#[cfg(test)]
mod tests {
    use super::{Backdrop, Tint, floors, over, rgba};
    use ds::tokens::Alpha;
    use ds::{Material, Scheme, Verdict};

    /// A colour's CSS, and its channels and alpha when it is one.
    type Case = (&'static str, Option<([u8; 3], f64)>);

    #[test]
    fn colours_parse_as_ds_writes_them() {
        const CASES: &[Case] = &[
            ("rgba(248,249,246,.7)", Some(([248, 249, 246], 0.7))),
            ("#FFFFFF", Some(([255, 255, 255], 1.0))),
            ("var(--f-grad)", None),
            ("rgba(1,2,3)", None),
        ];
        for (css, want) in CASES {
            assert_eq!(rgba(css), *want, "{css}");
        }
    }

    #[test]
    fn half_white_over_black_is_mid_grey() {
        assert_eq!(
            over(([255, 255, 255], 0.5), [0, 0, 0]).to_lowercase(),
            "#808080"
        );
    }

    #[test]
    fn every_tinted_material_has_four_floors_and_the_window_none() {
        for scheme in Scheme::ALL {
            for material in Material::ALL {
                let got = floors(material, scheme, Alpha(800));
                let want = if material == Material::Window { 0 } else { 4 };
                assert_eq!(got.len(), want, "{material:?} {scheme:?}");
            }
        }
    }

    #[test]
    fn the_default_tints_hold_the_floor_and_a_thin_one_does_not() {
        for scheme in Scheme::ALL {
            for material in Material::ALL {
                for floor in floors(material, scheme, Alpha(800)) {
                    assert_eq!(
                        floor.verdict(),
                        Verdict::Pass,
                        "{material:?} {scheme:?} {floor:?}"
                    );
                }
            }
        }
        let thin = floors(Material::Dock, Scheme::Light, Alpha(100));
        let over_black = thin
            .iter()
            .find(|floor| floor.tint == Tint::OverBlur && floor.backdrop == Backdrop::Black);
        assert_eq!(over_black.map(|floor| floor.verdict()), Some(Verdict::Fail));
    }
}
