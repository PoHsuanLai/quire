//! The spacing scale (design/01-LAYOUT.md section 2): every padding, gap and margin in `S`'s
//! app surfaces is one of these eighteen steps. The scale is dense, not a geometric ramp, so
//! each token is named by its own pixel value: `--s-8` is 8 px.
//!
//! The section's odd values (1.5, 40, 46, 50, 56) belong to one component each and stay in
//! that component's sheet; the floating clamp margin and the card inset are both `--s-8`.

use super::name::VarName;

/// One step of the spacing scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpacingToken {
    /// `--s-1`: 1 px.
    S1,
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
    /// `--s-14`: 14 px.
    S14,
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
    pub const ALL: [SpacingToken; 18] = [
        SpacingToken::S1,
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
        SpacingToken::S14,
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
            SpacingToken::S14 => "--s-14",
            SpacingToken::S16 => "--s-16",
            SpacingToken::S18 => "--s-18",
            SpacingToken::S22 => "--s-22",
            SpacingToken::S26 => "--s-26",
            SpacingToken::S36 => "--s-36",
        })
    }

    /// The step in logical pixels.
    pub fn px(self) -> u16 {
        match self {
            SpacingToken::S1 => 1,
            SpacingToken::S2 => 2,
            SpacingToken::S3 => 3,
            SpacingToken::S4 => 4,
            SpacingToken::S5 => 5,
            SpacingToken::S6 => 6,
            SpacingToken::S7 => 7,
            SpacingToken::S8 => 8,
            SpacingToken::S9 => 9,
            SpacingToken::S10 => 10,
            SpacingToken::S11 => 11,
            SpacingToken::S12 => 12,
            SpacingToken::S14 => 14,
            SpacingToken::S16 => 16,
            SpacingToken::S18 => 18,
            SpacingToken::S22 => 22,
            SpacingToken::S26 => 26,
            SpacingToken::S36 => 36,
        }
    }

    /// The CSS value: `8px`.
    pub fn css(self) -> String {
        format!("{}px", self.px())
    }
}

#[cfg(test)]
mod tests {
    use super::SpacingToken;

    #[test]
    fn each_name_is_its_own_value() {
        for token in SpacingToken::ALL {
            assert_eq!(token.var().as_str(), format!("--s-{}", token.px()));
        }
    }

    #[test]
    fn the_scale_ascends() {
        let px = SpacingToken::ALL.map(SpacingToken::px);
        assert!(px.windows(2).all(|pair| pair[0] < pair[1]), "{px:?}");
    }
}
