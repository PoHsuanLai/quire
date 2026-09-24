//! Symbolic or image: design/08-ICONS.md section 1.5 step 2. A status item's pixmap (or a
//! non-symbolic themed icon) whose opaque pixels are all near-grey is monochrome and is
//! recoloured to the text colour like a symbolic icon; anything with colour in it states a
//! fact about its app and is shown as it is.
//!
//! Near-grey is OKLCH chroma below a threshold (design/08 section 1.5 proposes 0.04). The
//! settings key is `icons.symbolic_chroma_max` (design/22-SETTINGS.md section 3.3, default
//! 0.04, range 0.0..=0.2); `classify` uses [`ChromaLimit::default`] and [`classify_with`] takes
//! one built with [`ChromaLimit::try_from`]. `sill` still has to register the key in its own
//! settings crate and pass the parsed value through (FINDINGS "Tune wave").

use crate::error::DsError;

/// How an external icon should be drawn: the `IconSource` variant it belongs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconKind {
    /// Monochrome: recolour its alpha to the text colour (`IconSource::Symbolic`).
    Symbolic,
    /// Coloured: show it as it is (`IconSource::Image`).
    Image,
}

/// The chroma below which a pixel counts as grey, in thousandths of OKLCH chroma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChromaLimit(pub u16);

impl ChromaLimit {
    /// design/08 section 1.5's proposed threshold, 0.04 — the same value
    /// `icons.symbolic_chroma_max` (design/22-SETTINGS.md section 3.3) ships as its default.
    pub const PROPOSED: ChromaLimit = ChromaLimit(40);

    /// The range `icons.symbolic_chroma_max` allows, in plain OKLCH chroma (design/22-SETTINGS.md
    /// section 3.3: `0..0.2`).
    pub const RANGE: std::ops::RangeInclusive<f32> = 0.0..=0.2;
}

impl Default for ChromaLimit {
    /// `icons.symbolic_chroma_max`'s default, 0.04.
    fn default() -> Self {
        ChromaLimit::PROPOSED
    }
}

impl TryFrom<f32> for ChromaLimit {
    type Error = DsError;

    /// `chroma` as a settings key would carry it — plain OKLCH chroma, not thousandths — refusing
    /// anything outside `icons.symbolic_chroma_max`'s range (`ChromaLimit::RANGE`, `0.0..=0.2`).
    fn try_from(chroma: f32) -> Result<Self, DsError> {
        if !ChromaLimit::RANGE.contains(&chroma) {
            return Err(DsError::ChromaLimitRange {
                value: format!("{chroma}"),
            });
        }
        Ok(ChromaLimit((chroma * 1000.0).round() as u16))
    }
}

/// The alpha at which a pixel counts as opaque: half coverage. Fainter edge pixels carry
/// too little of their colour to say anything about it.
const OPAQUE_FROM: u8 = 128;

/// Whether the PNG `png` is a symbolic or an image icon, at the default threshold
/// (`icons.symbolic_chroma_max`'s default, 0.04).
pub fn classify(png: &[u8]) -> Result<IconKind, DsError> {
    classify_with(png, ChromaLimit::default())
}

/// Whether the PNG `png` is a symbolic or an image icon: symbolic when every pixel with at
/// least half coverage has OKLCH chroma below `limit` (a fully transparent icon is symbolic).
pub fn classify_with(png: &[u8], limit: ChromaLimit) -> Result<IconKind, DsError> {
    let decoded = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map_err(|error| DsError::IconDecode {
            reason: error.to_string(),
        })?
        .to_rgba8();
    let bound = f64::from(limit.0) / 1000.0;
    let coloured = decoded
        .pixels()
        .filter(|pixel| pixel.0[3] >= OPAQUE_FROM)
        .any(|pixel| chroma([pixel.0[0], pixel.0[1], pixel.0[2]]) >= bound);
    Ok(if coloured {
        IconKind::Image
    } else {
        IconKind::Symbolic
    })
}

/// The OKLCH chroma of an sRGB colour (Björn Ottosson's OKLab, 2020).
pub(crate) fn chroma([r, g, b]: [u8; 3]) -> f64 {
    let [r, g, b] = [r, g, b].map(linear);
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    let a = 1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s;
    let b = 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s;
    a.hypot(b)
}

/// An sRGB byte as linear light.
fn linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChromaLimit, IconKind, chroma, classify, classify_with};
    use image::{ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    /// A 16 x 16 PNG: a ring of `edge` around an 8 x 8 middle of `middle`.
    fn png(middle: [u8; 4], edge: [u8; 4]) -> Vec<u8> {
        let image = RgbaImage::from_fn(16, 16, |x, y| {
            let inside = (4..12).contains(&x) && (4..12).contains(&y);
            Rgba(if inside { middle } else { edge })
        });
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("a PNG encodes");
        bytes.into_inner()
    }

    const CLEAR: [u8; 4] = [0, 0, 0, 0];

    #[test]
    fn grey_is_symbolic_and_colour_is_an_image() {
        // (name, middle, edge, want)
        #[rustfmt::skip]
        const CASES: &[(&str, [u8; 4], [u8; 4], IconKind)] = &[
            ("black glyph on clear", [0, 0, 0, 255], CLEAR, IconKind::Symbolic),
            ("white glyph on clear", [255, 255, 255, 255], CLEAR, IconKind::Symbolic),
            ("mid grey, opaque ground", [128, 128, 128, 255], [40, 40, 40, 255], IconKind::Symbolic),
            ("a faint tint of blue", [120, 122, 128, 255], CLEAR, IconKind::Symbolic),
            ("red", [220, 30, 40, 255], CLEAR, IconKind::Image),
            ("app blue", [40, 110, 220, 255], CLEAR, IconKind::Image),
            ("grey glyph, green badge ring", [90, 90, 90, 255], [40, 180, 80, 255], IconKind::Image),
            ("colour only where nearly clear", [0, 0, 0, 255], [220, 30, 40, 100], IconKind::Symbolic),
            ("colour at half coverage", [0, 0, 0, 255], [220, 30, 40, 128], IconKind::Image),
            ("nothing opaque at all", CLEAR, CLEAR, IconKind::Symbolic),
        ];
        for &(name, middle, edge, want) in CASES {
            assert_eq!(classify(&png(middle, edge)), Ok(want), "{name}");
        }
    }

    #[test]
    fn the_threshold_is_the_callers_to_move() {
        let tinted = png([120, 122, 140, 255], CLEAR);
        let measured = chroma([120, 122, 140]);
        assert!((0.02..0.04).contains(&measured), "{measured}");
        assert_eq!(classify(&tinted), Ok(IconKind::Symbolic));
        assert_eq!(classify_with(&tinted, ChromaLimit(20)), Ok(IconKind::Image));
    }

    #[test]
    fn default_is_the_proposed_threshold() {
        assert_eq!(ChromaLimit::default(), ChromaLimit::PROPOSED);
        assert_eq!(ChromaLimit::default(), ChromaLimit(40));
    }

    #[test]
    fn try_from_f32_scales_to_thousandths_and_refuses_out_of_range() {
        assert_eq!(ChromaLimit::try_from(0.04), Ok(ChromaLimit(40)));
        assert_eq!(ChromaLimit::try_from(0.0), Ok(ChromaLimit(0)));
        assert_eq!(ChromaLimit::try_from(0.2), Ok(ChromaLimit(200)));
        assert!(ChromaLimit::try_from(-0.01).is_err());
        assert!(ChromaLimit::try_from(0.201).is_err());
    }

    #[test]
    fn grey_has_no_chroma_and_primaries_have_their_own() {
        #[rustfmt::skip]
        const CASES: &[([u8; 3], f64)] = &[
            ([0, 0, 0], 0.0),
            ([255, 255, 255], 0.0),
            ([119, 119, 119], 0.0),
            // Ottosson's published OKLCH for the sRGB primaries.
            ([255, 0, 0], 0.2577),
            ([0, 255, 0], 0.2948),
            ([0, 0, 255], 0.3132),
        ];
        for &(rgb, want) in CASES {
            let got = chroma(rgb);
            assert!((got - want).abs() < 0.001, "{rgb:?}: {got} vs {want}");
        }
    }

    #[test]
    fn bytes_that_are_not_a_png_are_refused() {
        assert!(classify(b"not a png").is_err());
    }
}
