//! The pixel tokens: line widths that stay whole device pixels at any scale (design/01-LAYOUT.md
//! section 2.1, "Pixel snapping").
//!
//! A `1px` line at scale 1.5 is 1.5 device pixels, which paints as one full row and one half
//! row; at 1.25 and 1.75 it is worse. Every hairline, 1 px border, separator and focus ring in
//! quire reads one of these instead, and the root writes each one's value for its scale
//! (`Ds { scale }`), so the line is exactly one (or a whole number of) device pixels.
//!
//! Each is a [`Tuned`] token: `.ds` declares `--hair:var(--scale-hair,1px)` and the root writes
//! `--scale-hair` inline, so every nested `.ds` scope re-reads the root's value (the same reason
//! the shell's tuned tokens are written this way). At scale 1, and when no scale is given, every
//! token is its design value verbatim: the stylesheet and every picture at 1x are unchanged.

use super::name::VarName;
use super::tuned::Tuned;
use crate::geometry::Scale;

/// One pixel token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PixelToken {
    /// `--dpr`: the scale itself, unitless (`1.5`), for a `calc()` that needs it.
    Dpr,
    /// `--px`: exactly one device pixel, in logical pixels (`0.66667px` at 1.5).
    Px,
    /// `--hair`: a 1 px line (a border, a separator, the highlight): one logical pixel rounded
    /// down to whole device pixels, never below one. 1 px at 1x and 2x, one device pixel at
    /// 1.25, 1.5 and 1.75.
    Hair,
    /// `--hairline`: the material stack's half-pixel hairline (design/03-COLOR.md section 17.4),
    /// the same rounding from .5 px: .5 px at 1x (where no device pixel is thinner), one device
    /// pixel everywhere else.
    Hairline,
    /// `--ring`: a 3 px focus or flash ring's spread, rounded to whole device pixels.
    Ring,
    /// `--focus-ring`: the keyboard focus outline's width, 2.5 px rounded down to whole device
    /// pixels (Blitz's style engine floors an outline width the same way).
    FocusRing,
}

/// How a token's design length becomes whole device pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Snap {
    /// Exactly one device pixel.
    One,
    /// Round down, but never below one device pixel (the CSS rule for a border width).
    Floor(Thousandths),
    /// Round to the nearest device pixel, never below one.
    Nearest(Thousandths),
}

/// A length in thousandths of a logical pixel: 2500 is 2.5 px.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Thousandths(u32);

impl PixelToken {
    /// Every pixel token, in stylesheet order.
    pub const ALL: [PixelToken; 6] = [
        PixelToken::Dpr,
        PixelToken::Px,
        PixelToken::Hair,
        PixelToken::Hairline,
        PixelToken::Ring,
        PixelToken::FocusRing,
    ];

    /// The token components read and the input the root writes, with the 1x value behind it.
    pub fn tuned(self) -> Tuned {
        let (token, input, default) = match self {
            PixelToken::Dpr => ("--dpr", "--scale-dpr", "1"),
            PixelToken::Px => ("--px", "--scale-px", "1px"),
            PixelToken::Hair => ("--hair", "--scale-hair", "1px"),
            PixelToken::Hairline => ("--hairline", "--scale-hairline", ".5px"),
            PixelToken::Ring => ("--ring", "--scale-ring", "3px"),
            PixelToken::FocusRing => ("--focus-ring", "--scale-focus-ring", "2.5px"),
        };
        Tuned {
            token: VarName(token),
            input: VarName(input),
            default,
        }
    }

    /// The custom property components read: `--hair`.
    pub fn var(self) -> VarName {
        self.tuned().token
    }

    /// The value at `scale`, as CSS. At [`Scale::ONE`] it is the design value verbatim.
    pub fn css(self, scale: Scale) -> String {
        if scale == Scale::ONE {
            return self.tuned().default.to_owned();
        }
        match self.snap() {
            None => decimal(u64::from(scale.numerator()), u64::from(Scale::DENOMINATOR)),
            Some(snap) => format!("{}px", logical(device_pixels(snap, scale), scale)),
        }
    }

    /// How many device pixels the token covers at `scale`; `None` for `--dpr`, which is no
    /// length.
    pub fn device_pixels(self, scale: Scale) -> Option<u32> {
        self.snap().map(|snap| device_pixels(snap, scale))
    }

    fn snap(self) -> Option<Snap> {
        match self {
            PixelToken::Dpr => None,
            PixelToken::Px => Some(Snap::One),
            PixelToken::Hair => Some(Snap::Floor(Thousandths(1000))),
            PixelToken::Hairline => Some(Snap::Floor(Thousandths(500))),
            PixelToken::Ring => Some(Snap::Nearest(Thousandths(3000))),
            PixelToken::FocusRing => Some(Snap::Floor(Thousandths(2500))),
        }
    }

    /// The root's inline writes for `scale`: nothing at [`Scale::ONE`], where the stylesheet's
    /// defaults already hold, so a root without a scale renders the markup it always did.
    pub fn style_attr(scale: Scale) -> String {
        if scale == Scale::ONE {
            return String::new();
        }
        PixelToken::ALL
            .into_iter()
            .map(|token| token.tuned().write(&token.css(scale)))
            .collect()
    }
}

/// The whole device pixels `snap` covers at `scale`.
fn device_pixels(snap: Snap, scale: Scale) -> u32 {
    // A design length of `t` thousandths covers `t * n / 120 / 1000` device pixels.
    let per = u64::from(Scale::DENOMINATOR) * 1000;
    let exact = |Thousandths(t): Thousandths| u64::from(t) * u64::from(scale.numerator());
    let device = match snap {
        Snap::One => 1,
        Snap::Floor(length) => exact(length) / per,
        Snap::Nearest(length) => (exact(length) + per / 2) / per,
    };
    u32::try_from(device.max(1)).unwrap_or(u32::MAX)
}

/// `device` device pixels in logical pixels at `scale`, rounded up in the fifth decimal, so the
/// length a renderer multiplies back is never a hair short of the whole device pixel.
fn logical(device: u32, scale: Scale) -> String {
    decimal(
        u64::from(device) * u64::from(Scale::DENOMINATOR),
        u64::from(scale.numerator()),
    )
}

/// `numerator / denominator` to at most five decimals, rounded up, without trailing zeros:
/// `1.5`, `0.66667`, `1`.
fn decimal(numerator: u64, denominator: u64) -> String {
    const PLACES: u64 = 100_000;
    let scaled = (numerator * PLACES).div_ceil(denominator);
    let (whole, part) = (scaled / PLACES, scaled % PLACES);
    if part == 0 {
        return whole.to_string();
    }
    let digits = format!("{part:05}");
    format!("{whole}.{}", digits.trim_end_matches('0'))
}

#[cfg(test)]
mod tests {
    use super::PixelToken;
    use crate::geometry::Scale;

    /// Each token's CSS at 1.25, 1.5, 1.75 and 2.
    const TABLE: &[(PixelToken, [&str; 4])] = &[
        (PixelToken::Dpr, ["1.25", "1.5", "1.75", "2"]),
        (PixelToken::Px, ["0.8px", "0.66667px", "0.57143px", "0.5px"]),
        (PixelToken::Hair, ["0.8px", "0.66667px", "0.57143px", "1px"]),
        (
            PixelToken::Hairline,
            ["0.8px", "0.66667px", "0.57143px", "0.5px"],
        ),
        (PixelToken::Ring, ["3.2px", "3.33334px", "2.85715px", "3px"]),
        (
            PixelToken::FocusRing,
            ["2.4px", "2px", "2.28572px", "2.5px"],
        ),
    ];

    const SCALES: [Scale; 4] = [Scale(150), Scale(180), Scale(210), Scale(240)];

    #[test]
    fn each_token_has_its_value_at_each_scale() {
        for &(token, expected) in TABLE {
            for (scale, want) in SCALES.into_iter().zip(expected) {
                assert_eq!(token.css(scale), want, "{token:?} at {scale:?}");
            }
        }
    }

    #[test]
    fn every_length_is_a_whole_number_of_device_pixels() {
        for token in PixelToken::ALL {
            for scale in SCALES {
                let Some(device) = token.device_pixels(scale) else {
                    continue;
                };
                let css = token.css(scale);
                let logical: f64 = css.trim_end_matches("px").parse().expect("a number");
                let covered = logical * scale.as_f64();
                assert!(
                    covered >= f64::from(device) && covered - f64::from(device) < 1e-3,
                    "{token:?} at {scale:?}: {css} covers {covered} device px, not {device}"
                );
            }
        }
    }

    #[test]
    fn at_one_every_token_is_its_design_value_and_the_root_writes_nothing() {
        let design: Vec<String> = PixelToken::ALL.map(|token| token.css(Scale::ONE)).to_vec();
        assert_eq!(design, ["1", "1px", "1px", ".5px", "3px", "2.5px"]);
        assert_eq!(PixelToken::style_attr(Scale::ONE), "");
    }

    #[test]
    fn a_hairline_is_one_device_pixel_at_every_fractional_scale() {
        for scale in [Scale(150), Scale(180), Scale(210)] {
            assert_eq!(PixelToken::Hair.device_pixels(scale), Some(1));
            assert_eq!(PixelToken::Hairline.device_pixels(scale), Some(1));
            assert_eq!(PixelToken::Px.device_pixels(scale), Some(1));
        }
        assert_eq!(PixelToken::Hair.device_pixels(Scale(240)), Some(2));
    }

    #[test]
    fn the_root_writes_every_input_at_another_scale() {
        let style = PixelToken::style_attr(Scale(180));
        assert_eq!(
            style,
            "--scale-dpr:1.5;--scale-px:0.66667px;--scale-hair:0.66667px;\
             --scale-hairline:0.66667px;--scale-ring:3.33334px;--scale-focus-ring:2px;"
        );
    }
}
