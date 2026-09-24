//! Round four (design/08-ICONS.md 2.10): one muted palette and four dialects over one shared
//! skeleton. A dialect decides only the colours of the plate and of the symbol's three roles;
//! the plate shape, light, bevel, emboss depth and grain are the same for all four.

use std::str::FromStr;

use serde::{Deserialize, Deserializer};

use crate::{IconsError, Oklab, Srgb8, oklab};

/// The chroma cap: no colour in an app icon, plate or symbol, exceeds this OKLCh chroma. Round
/// three's plates ran to about 0.15-0.20; Klein's soft grounds sit near 0.06-0.09.
pub const CHROMA_CAP: f32 = 0.07;

/// A colour in OKLCh: lightness 0..=1, chroma, hue in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Lch {
    /// The OKLab form, with the chroma cap applied.
    pub fn lab(self) -> Oklab {
        let c = self.c.min(CHROMA_CAP);
        let h = self.h.to_radians();
        Oklab {
            l: self.l,
            a: c * h.cos(),
            b: c * h.sin(),
        }
    }

    /// The OKLCh form of an sRGB colour (chroma not capped: this is a reading, not a paint).
    pub fn of(c: Srgb8) -> Lch {
        let p = oklab(c.unit());
        Lch {
            l: p.l,
            c: p.a.hypot(p.b),
            h: p.b.atan2(p.a).to_degrees().rem_euclid(360.0),
        }
    }
}

/// The hue a dialect is tinted with, and how much of the cap it may use (0..=1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    pub hue: f32,
    pub strength: f32,
}

/// The muted palette: six named hues; every icon's colour comes from one of them.
pub const PALETTE: [(&str, f32); 6] = [
    ("slate", 255.0),
    ("teal", 200.0),
    ("sage", 150.0),
    ("ochre", 85.0),
    ("clay", 40.0),
    ("plum", 320.0),
];

impl FromStr for Tint {
    type Err = IconsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PALETTE
            .iter()
            .find(|(name, _)| *name == s)
            .map(|(_, hue)| Tint {
                hue: *hue,
                strength: 1.0,
            })
            .ok_or_else(|| IconsError::UnknownTint(s.to_owned()))
    }
}

impl<'de> Deserialize<'de> for Tint {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// The four dialects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Dialect {
    /// One hue: plate and symbol share it and differ only in lightness.
    Monochrome,
    /// A near-black neutral plate, a light symbol, no hue.
    Graphite,
    /// Post's paper plate, an ink symbol, at most one small spot of colour.
    Paper,
    /// One flat muted colour, a white symbol.
    Solid,
}

impl Dialect {
    pub const ALL: [Dialect; 4] = [
        Dialect::Monochrome,
        Dialect::Graphite,
        Dialect::Paper,
        Dialect::Solid,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Dialect::Monochrome => "monochrome",
            Dialect::Graphite => "graphite",
            Dialect::Paper => "paper",
            Dialect::Solid => "solid",
        }
    }
}

/// Whether the plate carries the almost invisible tonal shift (Solid is one flat colour).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
    Tonal,
    Flat,
}

/// The colours one dialect paints with: the plate, and the symbol's roles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Roles {
    pub plate: Oklab,
    pub shift: Shift,
    /// The main shapes.
    pub symbol: Oklab,
    /// A quieter second shape standing on the plate beside or behind the symbol.
    pub secondary: Oklab,
    /// Details pressed into the symbol.
    pub detail: Oklab,
    /// The one small accent a symbol may carry: a hue only in Paper, a tone elsewhere.
    pub spot: Oklab,
}

fn lch(l: f32, c: f32, h: f32) -> Oklab {
    Lch { l, c, h }.lab()
}

fn hex(c: u32) -> Oklab {
    oklab(Srgb8::hex(c).unit())
}

/// The roles of a dialect under a tint. Values are proposed (08 2.10).
pub fn roles(dialect: Dialect, tint: Tint) -> Roles {
    let (h, k) = (tint.hue, tint.strength.clamp(0.0, 1.0));
    let cap = CHROMA_CAP * k;
    match dialect {
        Dialect::Monochrome => Roles {
            plate: lch(0.58, 0.85 * cap, h),
            shift: Shift::Tonal,
            symbol: lch(0.85, 0.6 * cap, h),
            secondary: lch(0.72, 0.7 * cap, h),
            detail: lch(0.70, 0.75 * cap, h),
            spot: lch(0.72, 0.7 * cap, h),
        },
        Dialect::Graphite => Roles {
            plate: lch(0.30, 0.004, 250.0),
            shift: Shift::Tonal,
            symbol: lch(0.93, 0.0, 0.0),
            secondary: lch(0.58, 0.0, 0.0),
            detail: lch(0.62, 0.0, 0.0),
            spot: lch(0.62, 0.0, 0.0),
        },
        Dialect::Paper => Roles {
            plate: hex(0xF8F9F6),
            shift: Shift::Tonal,
            symbol: hex(0x1A1E1A),
            secondary: hex(0xA0A79B),
            detail: hex(0xE3E7DE),
            spot: lch(0.62, cap, h),
        },
        Dialect::Solid => Roles {
            plate: lch(0.62, cap, h),
            shift: Shift::Flat,
            symbol: hex(0xFBFBF8),
            secondary: lch(0.84, 0.3 * cap, h),
            detail: lch(0.72, 0.7 * cap, h),
            spot: lch(0.84, 0.3 * cap, h),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chroma(p: Oklab) -> f32 {
        p.a.hypot(p.b)
    }

    #[test]
    fn every_colour_respects_the_cap() {
        for d in Dialect::ALL {
            for (name, hue) in PALETTE {
                let r = roles(d, Tint { hue, strength: 1.0 });
                for (role, c) in [
                    ("plate", r.plate),
                    ("symbol", r.symbol),
                    ("secondary", r.secondary),
                    ("detail", r.detail),
                    ("spot", r.spot),
                ] {
                    assert!(chroma(c) <= CHROMA_CAP + 1e-4, "{d:?} {name} {role}");
                }
            }
        }
    }

    /// The symbol stands off its plate by lightness in every dialect (legible at 16 px).
    #[test]
    fn symbol_and_plate_differ_in_lightness() {
        for d in Dialect::ALL {
            let r = roles(d, "slate".parse().expect("slate"));
            assert!((r.symbol.l - r.plate.l).abs() >= 0.26, "{d:?}");
        }
    }

    #[test]
    fn monochrome_is_one_hue_and_graphite_none() {
        let r = roles(Dialect::Monochrome, "sage".parse().expect("sage"));
        let hue = |p: Oklab| p.b.atan2(p.a).to_degrees().rem_euclid(360.0);
        assert!((hue(r.plate) - 150.0).abs() < 0.5 && (hue(r.symbol) - 150.0).abs() < 0.5);
        let g = roles(Dialect::Graphite, "sage".parse().expect("sage"));
        assert!(chroma(g.plate) < 0.01 && chroma(g.symbol) < 1e-6);
    }

    #[test]
    fn lch_reads_back() {
        let l = Lch::of(Srgb8::hex(0x2B7CFF));
        assert!(l.c > 0.15 && (l.h - 260.0).abs() < 8.0, "{l:?}");
        assert!("teal".parse::<Tint>().is_ok() && "cyan".parse::<Tint>().is_err());
    }
}
