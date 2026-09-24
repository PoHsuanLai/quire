use std::str::FromStr;

use crate::{IconsError, Srgb8};

/// A plate gradient family from the Candy shelf (design/08-ICONS.md 2.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Red,
    Amber,
    Green,
    Blue,
    Violet,
    Paper,
}

/// The two gradient stops: `base` at the top-left, `deep` at the bottom-right (135deg).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stops {
    pub base: Srgb8,
    pub deep: Srgb8,
}

impl Family {
    /// Every family, in the doc's table order.
    pub const ALL: [Family; 6] = [
        Family::Red,
        Family::Amber,
        Family::Green,
        Family::Blue,
        Family::Violet,
        Family::Paper,
    ];

    /// The family's gradient stops (08 2.3 table).
    pub const fn stops(self) -> Stops {
        let (base, deep) = match self {
            Family::Red => (0xE8483C, 0xB7352B),
            Family::Amber => (0xF0A81E, 0x8E5A05),
            Family::Green => (0x28B24A, 0x1A7A33),
            Family::Blue => (0x2B7CFF, 0x0B5FE0),
            Family::Violet => (0x8B5CF0, 0x6B3FCC),
            Family::Paper => (0xFFFFFF, 0xF1F3EE),
        };
        Stops {
            base: Srgb8::hex(base),
            deep: Srgb8::hex(deep),
        }
    }

    /// The family's soft tint (08 2.3; the paper family's is Post `--surface-2`).
    pub const fn soft(self) -> Srgb8 {
        Srgb8::hex(match self {
            Family::Red => 0xFBE3E1,
            Family::Amber => 0xFAEBD2,
            Family::Green => 0xDCF1E1,
            Family::Blue => 0xE2EBFF,
            Family::Violet => 0xECE4FB,
            Family::Paper => 0xF1F3EE,
        })
    }

    /// The lower-case name used on the command line.
    pub const fn name(self) -> &'static str {
        match self {
            Family::Red => "red",
            Family::Amber => "amber",
            Family::Green => "green",
            Family::Blue => "blue",
            Family::Violet => "violet",
            Family::Paper => "paper",
        }
    }
}

impl FromStr for Family {
    type Err = IconsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Family::ALL
            .into_iter()
            .find(|f| f.name() == s)
            .ok_or_else(|| IconsError::UnknownFamily(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for f in Family::ALL {
            assert_eq!(f.name().parse::<Family>().ok(), Some(f), "{f:?}");
        }
        assert!("teal".parse::<Family>().is_err());
    }
}
