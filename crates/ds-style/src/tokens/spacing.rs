//! The spacing scale (design/01-LAYOUT.md section 2): every padding, gap and margin in `S`'s
//! app surfaces is one of these steps. The scale is dense, not a geometric ramp, so each token
//! is named by its own pixel value: `--s-8` is 8 px, `--s-1-5` is 1.5 px.
//!
//! The eighteen common steps (design/34-MODERN-LOOK.md section 2.4 adds 20, 24, 28, 32, 40 and 48 for the larger Look's margins and group gaps), plus the three values 04-COMPONENTS quotes exactly for a quire
//! component and the common steps miss: 1.5 (the chip's and the image provider mark's
//! padding), 13 (the hover card's inline padding) and 15 (the toast's leading padding). The
//! section's other odd values (40, 46, 50, 56) belong to mail surfaces quire does not draw; the
//! floating clamp margin and the card inset are both `--s-8`.

use crate::tokens::token::{Token, TokenScope};
use ds_core::word::Word;

/// One step of the spacing scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Word, Token)]
#[token(prefix = "s-", kind = fixed)]
#[non_exhaustive]
pub enum SpacingToken {
    /// `--s-1`: 1 px.
    #[token(name = "1", value = "1px")]
    S1,
    /// `--s-1-5`: 1.5 px, the chip's and the image mark's padding (04-COMPONENTS sections 10, 28).
    #[token(name = "1-5", value = "1.5px")]
    S1Half,
    /// `--s-2`: 2 px.
    #[token(name = "2", value = "2px")]
    S2,
    /// `--s-3`: 3 px.
    #[token(name = "3", value = "3px")]
    S3,
    /// `--s-4`: 4 px.
    #[token(name = "4", value = "4px")]
    S4,
    /// `--s-5`: 5 px.
    #[token(name = "5", value = "5px")]
    S5,
    /// `--s-6`: 6 px.
    #[token(name = "6", value = "6px")]
    S6,
    /// `--s-7`: 7 px.
    #[token(name = "7", value = "7px")]
    S7,
    /// `--s-8`: 8 px.
    #[token(name = "8", value = "8px")]
    S8,
    /// `--s-9`: 9 px.
    #[token(name = "9", value = "9px")]
    S9,
    /// `--s-10`: 10 px.
    #[token(name = "10", value = "10px")]
    S10,
    /// `--s-11`: 11 px.
    #[token(name = "11", value = "11px")]
    S11,
    /// `--s-12`: 12 px.
    #[token(name = "12", value = "12px")]
    S12,
    /// `--s-13`: 13 px, the hover card's inline padding (04-COMPONENTS section 22).
    #[token(name = "13", value = "13px")]
    S13,
    /// `--s-14`: 14 px.
    #[token(name = "14", value = "14px")]
    S14,
    /// `--s-15`: 15 px, the toast's leading padding (04-COMPONENTS section 23).
    #[token(name = "15", value = "15px")]
    S15,
    /// `--s-16`: 16 px.
    #[token(name = "16", value = "16px")]
    S16,
    /// `--s-18`: 18 px.
    #[token(name = "18", value = "18px")]
    S18,
    /// `--s-20`: 20 px.
    #[token(name = "20", value = "20px")]
    S20,
    /// `--s-22`: 22 px.
    #[token(name = "22", value = "22px")]
    S22,
    /// `--s-24`: 24 px.
    #[token(name = "24", value = "24px")]
    S24,
    /// `--s-26`: 26 px.
    #[token(name = "26", value = "26px")]
    S26,
    /// `--s-28`: 28 px.
    #[token(name = "28", value = "28px")]
    S28,
    /// `--s-32`: 32 px.
    #[token(name = "32", value = "32px")]
    S32,
    /// `--s-36`: 36 px.
    #[token(name = "36", value = "36px")]
    S36,
    /// `--s-40`: 40 px.
    #[token(name = "40", value = "40px")]
    S40,
    /// `--s-48`: 48 px.
    #[token(name = "48", value = "48px")]
    S48,
}

impl SpacingToken {
    /// The step in tenths of a pixel: `15` is `1.5px`, read from the token's own value.
    pub fn tenths(self) -> u16 {
        let value = self.css_value(TokenScope::BASE);
        let pixels: f32 = value.as_str().trim_end_matches("px").parse().unwrap_or(0.0);
        (pixels * 10.0).round() as u16
    }
}

#[cfg(test)]
mod tests {
    use super::SpacingToken;
    use ds_core::word::Word;

    #[test]
    fn the_scale_ascends() {
        let tenths: Vec<u16> = SpacingToken::ALL
            .iter()
            .map(|token| token.tenths())
            .collect();
        assert!(
            tenths.windows(2).all(|pair| pair[0] < pair[1]),
            "{tenths:?}"
        );
    }
}
