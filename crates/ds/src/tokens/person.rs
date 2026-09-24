//! Identity colours as tokens (design/03-COLOR.md section 13): the eight person swatches
//! `--c-person-1..8`, the account and pin colours a consumer takes in order when it has stored
//! none. They are data, not theme, so they are the same in both schemes. The hashed hue of a
//! person with no stored colour is [`crate::PersonHue`] (`crate::person_hue`).

use super::hex::{Colour, Hex};

/// One of the eight person swatches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PersonSwatch {
    /// `--c-person-1` `#5B4FC4`, indigo.
    Indigo,
    /// `--c-person-2` `#2F7F6E`, teal.
    Teal,
    /// `--c-person-3` `#B0662E`, rust.
    Rust,
    /// `--c-person-4` `#3C8A5B`, green.
    Green,
    /// `--c-person-5` `#7A4A9E`, plum.
    Plum,
    /// `--c-person-6` `#C0782E`, ochre.
    Ochre,
    /// `--c-person-7` `#2E7F8C`, sea.
    Sea,
    /// `--c-person-8` `#6D7A3A`, olive.
    Olive,
}

impl PersonSwatch {
    /// Every swatch, in the order a consumer hands them out (mailo's `AVATAR`).
    pub const ALL: [PersonSwatch; 8] = [
        PersonSwatch::Indigo,
        PersonSwatch::Teal,
        PersonSwatch::Rust,
        PersonSwatch::Green,
        PersonSwatch::Plum,
        PersonSwatch::Ochre,
        PersonSwatch::Sea,
        PersonSwatch::Olive,
    ];

    /// The swatch at `index`, wrapping: the first account takes the first swatch, the ninth
    /// the first again.
    pub fn nth(index: usize) -> Self {
        Self::ALL[index % Self::ALL.len()]
    }

    /// The custom property: `--c-person-1` … `--c-person-8`.
    pub fn var(self) -> String {
        let number = Self::ALL
            .iter()
            .position(|swatch| *swatch == self)
            .map_or(0, |index| index + 1);
        format!("--c-person-{number}")
    }

    /// The colour (design/03-COLOR.md section 13's accounts and pins).
    pub fn hex(self) -> Hex {
        Hex(match self {
            PersonSwatch::Indigo => [0x5B, 0x4F, 0xC4],
            PersonSwatch::Teal => [0x2F, 0x7F, 0x6E],
            PersonSwatch::Rust => [0xB0, 0x66, 0x2E],
            PersonSwatch::Green => [0x3C, 0x8A, 0x5B],
            PersonSwatch::Plum => [0x7A, 0x4A, 0x9E],
            PersonSwatch::Ochre => [0xC0, 0x78, 0x2E],
            PersonSwatch::Sea => [0x2E, 0x7F, 0x8C],
            PersonSwatch::Olive => [0x6D, 0x7A, 0x3A],
        })
    }

    /// The colour as a [`Colour`], for `AvatarTone::Account`.
    pub fn colour(self) -> Colour {
        Colour::Solid(self.hex())
    }
}

#[cfg(test)]
mod tests {
    use super::PersonSwatch;

    #[test]
    fn the_swatches_are_mailos_avatar_order() {
        const AVATAR: [&str; 8] = [
            "#5B4FC4", "#2F7F6E", "#B0662E", "#3C8A5B", "#7A4A9E", "#C0782E", "#2E7F8C", "#6D7A3A",
        ];
        for (index, want) in AVATAR.iter().enumerate() {
            let swatch = PersonSwatch::nth(index);
            assert_eq!(swatch.hex().css().to_uppercase(), *want, "{swatch:?}");
            assert_eq!(swatch.var(), format!("--c-person-{}", index + 1));
            assert_eq!(PersonSwatch::nth(index + 8), swatch, "wraps");
        }
    }
}
