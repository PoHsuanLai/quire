//! The spacing scale (design/01-LAYOUT.md section 2): every padding, gap and margin in `S`'s
//! app surfaces is one of these steps. The scale is dense, not a geometric ramp, so each token
//! is named by its own pixel value: `--s-8` is 8 px, `--s-1-5` is 1.5 px.
//!
//! The eighteen common steps, plus the three values 04-COMPONENTS quotes exactly for a quire
//! component and the common steps miss: 1.5 (the chip's and the image provider mark's
//! padding), 13 (the hover card's inline padding) and 15 (the toast's leading padding). The
//! section's other odd values (40, 46, 50, 56) belong to mail surfaces quire does not draw; the
//! floating clamp margin and the card inset are both `--s-8`.

use super::name::VarName;

/// One step of the spacing scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpacingToken {
    /// `--s-1`: 1 px.
    S1,
    /// `--s-1-5`: 1.5 px, the chip's and the image mark's padding (04-COMPONENTS sections 10, 28).
    S1Half,
    /// `--s-2`: 2 px.
    S2,
    /// `--s-3`: 3 px.
    S3,
    /// `--s-4`: 4 px.
    S4,
    /// `--s-5`: 5 px.
    S5,
    /// `--s-6`: 6 px.
    S6,
    /// `--s-7`: 7 px.
    S7,
    /// `--s-8`: 8 px.
    S8,
    /// `--s-9`: 9 px.
    S9,
    /// `--s-10`: 10 px.
    S10,
    /// `--s-11`: 11 px.
    S11,
    /// `--s-12`: 12 px.
    S12,
    /// `--s-13`: 13 px, the hover card's inline padding (04-COMPONENTS section 22).
    S13,
    /// `--s-14`: 14 px.
    S14,
    /// `--s-15`: 15 px, the toast's leading padding (04-COMPONENTS section 23).
    S15,
    /// `--s-16`: 16 px.
    S16,
    /// `--s-18`: 18 px.
    S18,
    /// `--s-22`: 22 px.
    S22,
    /// `--s-26`: 26 px.
    S26,
    /// `--s-36`: 36 px.
    S36,
}

impl SpacingToken {
    /// Every step, smallest first, in stylesheet order.
    pub const ALL: [SpacingToken; 21] = [
        SpacingToken::S1,
        SpacingToken::S1Half,
        SpacingToken::S2,
        SpacingToken::S3,
        SpacingToken::S4,
        SpacingToken::S5,
        SpacingToken::S6,
        SpacingToken::S7,
        SpacingToken::S8,
        SpacingToken::S9,
        SpacingToken::S10,
        SpacingToken::S11,
        SpacingToken::S12,
        SpacingToken::S13,
        SpacingToken::S14,
        SpacingToken::S15,
        SpacingToken::S16,
        SpacingToken::S18,
        SpacingToken::S22,
        SpacingToken::S26,
        SpacingToken::S36,
    ];

    /// The custom property: `--s-1`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            SpacingToken::S1 => "--s-1",
            SpacingToken::S1Half => "--s-1-5",
            SpacingToken::S2 => "--s-2",
            SpacingToken::S3 => "--s-3",
            SpacingToken::S4 => "--s-4",
            SpacingToken::S5 => "--s-5",
            SpacingToken::S6 => "--s-6",
            SpacingToken::S7 => "--s-7",
            SpacingToken::S8 => "--s-8",
            SpacingToken::S9 => "--s-9",
            SpacingToken::S10 => "--s-10",
            SpacingToken::S11 => "--s-11",
            SpacingToken::S12 => "--s-12",
            SpacingToken::S13 => "--s-13",
            SpacingToken::S14 => "--s-14",
            SpacingToken::S15 => "--s-15",
            SpacingToken::S16 => "--s-16",
            SpacingToken::S18 => "--s-18",
            SpacingToken::S22 => "--s-22",
            SpacingToken::S26 => "--s-26",
            SpacingToken::S36 => "--s-36",
        })
    }

    /// The step in tenths of a logical pixel: 15 is 1.5 px, 80 is 8 px.
    pub fn tenths(self) -> u16 {
        match self {
            SpacingToken::S1 => 10,
            SpacingToken::S1Half => 15,
            SpacingToken::S2 => 20,
            SpacingToken::S3 => 30,
            SpacingToken::S4 => 40,
            SpacingToken::S5 => 50,
            SpacingToken::S6 => 60,
            SpacingToken::S7 => 70,
            SpacingToken::S8 => 80,
            SpacingToken::S9 => 90,
            SpacingToken::S10 => 100,
            SpacingToken::S11 => 110,
            SpacingToken::S12 => 120,
            SpacingToken::S13 => 130,
            SpacingToken::S14 => 140,
            SpacingToken::S15 => 150,
            SpacingToken::S16 => 160,
            SpacingToken::S18 => 180,
            SpacingToken::S22 => 220,
            SpacingToken::S26 => 260,
            SpacingToken::S36 => 360,
        }
    }

    /// The step as written in CSS and in its name: `8`, `1.5`.
    fn number(self) -> String {
        let tenths = self.tenths();
        match tenths % 10 {
            0 => (tenths / 10).to_string(),
            part => format!("{}.{part}", tenths / 10),
        }
    }

    /// The CSS value: `8px`, `1.5px`.
    pub fn css(self) -> String {
        format!("{}px", self.number())
    }
}

#[cfg(test)]
mod tests {
    use super::SpacingToken;

    #[test]
    fn each_name_is_its_own_value() {
        for token in SpacingToken::ALL {
            let name = format!("--s-{}", token.number().replace('.', "-"));
            assert_eq!(token.var().as_str(), name);
            assert_eq!(token.css(), format!("{}px", token.number()));
        }
        assert_eq!(SpacingToken::S1Half.css(), "1.5px");
    }

    #[test]
    fn the_scale_ascends() {
        let tenths = SpacingToken::ALL.map(SpacingToken::tenths);
        assert!(
            tenths.windows(2).all(|pair| pair[0] < pair[1]),
            "{tenths:?}"
        );
    }
}
