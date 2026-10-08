//! MarkFace: a provider mark described as data, a letter of one or two characters on a colour
//! (`#RRGGBB`), as a host that ships its providers as data supplies it. Quire never names the
//! host; it validates the pair and picks the letter's ink itself.

use ds_core::colour::contrast::ratio;
use ds_style::tokens::hex::Hex;

/// The neutral colour a face falls back to when its own is not `#RRGGBB`: the IMAP grey.
pub(crate) const NEUTRAL: &str = "#5D6660";

/// The dark ink a letter takes on a light colour (the light scheme's `--ink`).
const DARK_INK: &str = "#202020";

/// The white ink a letter takes on a dark colour.
const WHITE_INK: &str = "#FFFFFF";

/// A provider mark as data: the letter and the colour it sits on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MarkFace {
    /// One or two characters; more are cut to two, blank draws no face.
    pub letter: String,
    /// The colour as `#RRGGBB`; anything else draws the neutral mark.
    pub colour: String,
}

/// How many characters the letter has, which decides its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum LetterCount {
    /// One character.
    One,
    /// Two characters: set smaller to fit the mark.
    Two,
}

impl LetterCount {
    /// The `data-letters` value.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            LetterCount::One => "1",
            LetterCount::Two => "2",
        }
    }
}

/// A face that is safe to draw: a trimmed letter and a valid colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DrawnFace {
    pub(crate) letter: String,
    pub(crate) count: LetterCount,
    pub(crate) colour: String,
    pub(crate) ink: &'static str,
}

impl MarkFace {
    /// A face of `letter` on `colour`.
    pub fn new(letter: impl Into<String>, colour: impl Into<String>) -> Self {
        Self {
            letter: letter.into(),
            colour: colour.into(),
        }
    }

    /// The ink for the letter on `colour`: white or the dark ink, whichever contrasts more.
    pub fn ink_on(colour: &str) -> &'static str {
        let against = |ink: &str| ratio(ink, colour).unwrap_or_default();
        if against(WHITE_INK) >= against(DARK_INK) {
            WHITE_INK
        } else {
            DARK_INK
        }
    }

    /// What to draw: `None` when the letter is blank (the named variant stands in); a bad
    /// colour keeps nothing of the face and draws the neutral `@` mark's colour.
    pub(crate) fn drawn(&self) -> Option<DrawnFace> {
        let letter: String = self.letter.trim().chars().take(2).collect();
        let count = match letter.chars().count() {
            0 => return None,
            1 => LetterCount::One,
            _ => LetterCount::Two,
        };
        let valid = self.colour.len() == 7 && Hex::parse(&self.colour).is_some();
        let (letter, count, colour) = if valid {
            (letter, count, self.colour.clone())
        } else {
            ("@".to_owned(), LetterCount::One, NEUTRAL.to_owned())
        };
        let ink = Self::ink_on(&colour);
        Some(DrawnFace {
            letter,
            count,
            colour,
            ink,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_colour_draws_the_neutral_mark() {
        for colour in ["red", "#fff", "5D6660", "#12345G", "#1234567"] {
            let drawn = MarkFace::new("Ab", colour).drawn().expect("drawn");
            assert_eq!(
                (drawn.letter.as_str(), drawn.colour.as_str()),
                ("@", NEUTRAL)
            );
        }
    }

    #[test]
    fn a_blank_letter_draws_nothing_and_a_long_one_is_cut_to_two() {
        assert_eq!(MarkFace::new("  ", "#112233").drawn(), None);
        let drawn = MarkFace::new("Abc", "#112233").drawn().expect("drawn");
        assert_eq!(
            (drawn.letter.as_str(), drawn.count),
            ("Ab", LetterCount::Two)
        );
    }

    /// Every face the shipped providers use: the letter is large display text, so 3:1 is the floor.
    const SHIPPED: [(&str, &str); 13] = [
        ("A", "#D97757"),
        ("C", "#D97757"),
        ("O", "#10A37F"),
        ("Cx", "#10A37F"),
        ("G", "#8E75B2"),
        ("G", "#1C449B"),
        ("K", "#16191E"),
        ("N", "#0082C9"),
        ("OR", "#6467F2"),
        ("Ol", "#1F1F1F"),
        ("LM", "#4B3CC9"),
        ("L", "#8A5A44"),
        ("A", "#5D6660"),
    ];

    #[test]
    fn every_shipped_face_is_legible() {
        for (letter, colour) in SHIPPED {
            let drawn = MarkFace::new(letter, colour).drawn().expect("drawn");
            assert_eq!(drawn.colour, colour);
            let got = ratio(drawn.ink, colour).expect("hex pair");
            assert!(
                got >= 3.0,
                "{letter} on {colour} with {}: {got:.2}",
                drawn.ink
            );
        }
    }

    #[test]
    fn the_ink_is_whichever_contrasts_more() {
        assert_eq!(MarkFace::ink_on("#FFE066"), DARK_INK);
        assert_eq!(MarkFace::ink_on("#101010"), WHITE_INK);
    }
}
