//! Colour values as the token table holds them: an 8-bit sRGB hex, optionally with an alpha.
//!
//! No floats, so every colour is `Eq` and a token table can be compared in a test.

/// An 8-bit sRGB colour, written `#rrggbb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hex(pub [u8; 3]);

impl Hex {
    /// Parse `#rgb` or `#rrggbb`, any case; `None` for anything else.
    pub fn parse(text: &str) -> Option<Hex> {
        let digits = text.strip_prefix('#')?;
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let nibbles = digits
            .bytes()
            .map(|byte| {
                (byte as char)
                    .to_digit(16)
                    .and_then(|n| u8::try_from(n).ok())
            })
            .collect::<Option<Vec<u8>>>()?;
        match nibbles[..] {
            [r, g, b] => Some(Hex([r * 17, g * 17, b * 17])),
            [r1, r0, g1, g0, b1, b0] => Some(Hex([r1 * 16 + r0, g1 * 16 + g0, b1 * 16 + b0])),
            _ => None,
        }
    }

    /// Lower-case `#rrggbb`.
    pub fn css(self) -> String {
        let [r, g, b] = self.0;
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

/// Opacity in thousandths: 1000 is opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Alpha(pub u16);

impl Alpha {
    /// The alpha as CSS writes it: `.22`, `.8`, `1`, `0`.
    pub(crate) fn css(self) -> String {
        thousandths(i64::from(self.0))
    }
}

/// A token's value: solid, or a colour at an alpha (`rgba(...)`), which is how the washes,
/// the scrim and the frame pills are written now that `color-mix()` is gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Colour {
    /// An opaque colour.
    Solid(Hex),
    /// A colour at an alpha.
    Alpha(Hex, Alpha),
}

impl Colour {
    /// The CSS text: `#rrggbb` or `rgba(r,g,b,.a)`.
    pub fn css(self) -> String {
        match self {
            Colour::Solid(hex) => hex.css(),
            Colour::Alpha(Hex([r, g, b]), alpha) => format!("rgba({r},{g},{b},{})", alpha.css()),
        }
    }
}

/// A fixed-point number in thousandths as the shortest CSS decimal: `1040` is `1.04`, `955`
/// is `.955`, `-2000` is `-2`, `0` is `0`. Shared by every token that stores thousandths.
pub(crate) fn thousandths(value: i64) -> String {
    let sign = if value < 0 { "-" } else { "" };
    let magnitude = value.unsigned_abs();
    let whole = magnitude / 1000;
    let fraction = magnitude % 1000;
    if fraction == 0 {
        return format!("{sign}{whole}");
    }
    let digits = format!("{fraction:03}");
    let digits = digits.trim_end_matches('0');
    if whole == 0 {
        format!("{sign}.{digits}")
    } else {
        format!("{sign}{whole}.{digits}")
    }
}

#[cfg(test)]
mod tests {
    use super::{Alpha, Colour, Hex, thousandths};

    #[test]
    fn hex_parses_both_lengths_and_rejects_the_rest() {
        const CASES: &[(&str, Option<[u8; 3]>)] = &[
            ("#23508F", Some([0x23, 0x50, 0x8f])),
            ("#fff", Some([255, 255, 255])),
            ("#A1b", Some([0xaa, 0x11, 0xbb])),
            ("23508F", None),
            ("#23508", None),
            ("#23508G", None),
            ("#", None),
        ];
        for &(text, want) in CASES {
            assert_eq!(Hex::parse(text).map(|hex| hex.0), want, "{text}");
        }
    }

    #[test]
    fn colours_write_the_way_the_stylesheet_does() {
        const CASES: &[(Colour, &str)] = &[
            (Colour::Solid(Hex([0x23, 0x50, 0x8f])), "#23508f"),
            (Colour::Alpha(Hex([0, 0, 0]), Alpha(220)), "rgba(0,0,0,.22)"),
            (
                Colour::Alpha(Hex([255, 255, 255]), Alpha(1000)),
                "rgba(255,255,255,1)",
            ),
            (
                Colour::Alpha(Hex([44, 122, 87]), Alpha(160)),
                "rgba(44,122,87,.16)",
            ),
        ];
        for &(colour, want) in CASES {
            assert_eq!(colour.css(), want, "{colour:?}");
        }
    }

    #[test]
    fn thousandths_are_the_shortest_decimal() {
        const CASES: &[(i64, &str)] = &[
            (1040, "1.04"),
            (955, ".955"),
            (-2000, "-2"),
            (0, "0"),
            (2200, "2.2"),
            (1420, "1.42"),
            (-500, "-.5"),
        ];
        for &(value, want) in CASES {
            assert_eq!(thousandths(value), want, "{value}");
        }
    }
}
