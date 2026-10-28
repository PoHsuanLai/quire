//! The faces, shipped inside the binary as bytes (design/02-TYPE.md section 2): Inter and Inter
//! Display (the interface's faces), Space Mono (the code face), Noto Serif, and Bricolage
//! Grotesque and Karla (the faces the Editorial look Extension names, with Space Mono).
//!
//! On Blitz the bytes are registered with the renderer's font context once (`ds-blitz`'s
//! `register_fonts`); nothing about fonts is in the stylesheet.
//!
//! The files are subset TTFs (latin and latin-ext, Google Fonts' own split), made from mailo's
//! WOFF2 copies by `scripts/subset-fonts.sh`, because fontique reads sfnt and not WOFF2
//!. The script also names every face's family the way `--font-*` asks for it.
//! Space Mono's latin files are the exception: they are subset from the Space Mono 1.003 release
//! with the latin file's own code points plus U+2190-2193, because Google's latin range keeps
//! `↑` and `↓` but drops `←` and `→`, and a Small key cap's fallback arrows read as dashes
//! (FINDINGS "Text, fonts and editing"). Every other glyph is the same outline.
//! Inter is cut from the official Inter 4.1 release by `scripts/cut-inter.sh` (version and
//! SHA-256 recorded there): its variable font pinned to `opsz` 14 for Inter (400-700, italic
//! 400) and to `opsz` 32 for Inter Display (500-800), since the renderer sets no optical size.
//! The upright Inter text face also carries the key-cap glyphs (`⌃ ⌥ ⇧ ⌘ ⌫ ↩ ↵ ⇥ ⌦ ⇞ ⇟` and the
//! arrows, `scripts/cut-inter.sh`), because Google's latin range has none of them and a
//! `KeyEquivalent` fell to the system's face: U+2303 drew as a caret. Every stack names "Inter"
//! after its own face (`tokens::type_scale::Family`), so the glyphs come from here, in the code
//! face's key caps too.
//! Licences are in `assets/fonts/OFL-*.txt`.

use crate::tokens::type_scale::Family;

/// Upright or italic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaceStyle {
    /// Roman.
    Normal,
    /// Italic.
    Italic,
}

/// The weights a face covers: one for a static instance, a range for a variable face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WeightRange {
    /// The lightest weight.
    pub min: u16,
    /// The heaviest weight.
    pub max: u16,
}

/// Which unicode subset a file carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subset {
    /// Basic Latin and Latin-1, general punctuation.
    Latin,
    /// Latin Extended.
    LatinExt,
}

/// One font file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Face {
    /// The family name it registers as, the name a `--font-*` stack leads with: `"Inter"`.
    pub name: &'static str,
    /// The job it was cut for (Space Mono also does
    /// [`Family::Code`]).
    pub family: Family,
    /// Upright or italic.
    pub style: FaceStyle,
    /// The weights it covers.
    pub weight: WeightRange,
    /// The subset it carries.
    pub subset: Subset,
    /// The file.
    pub bytes: &'static [u8],
}

macro_rules! face {
    ($name:literal, $family:expr, $style:expr, $min:expr, $max:expr, $subset:expr, $file:literal) => {
        Face {
            name: $name,
            family: $family,
            style: $style,
            weight: WeightRange {
                min: $min,
                max: $max,
            },
            subset: $subset,
            bytes: include_bytes!(concat!("../assets/fonts/", $file)),
        }
    };
}

/// Every face the design system ships.
pub static FACES: &[Face] = &[
    face!(
        "Inter",
        Family::Ui,
        FaceStyle::Normal,
        400,
        700,
        Subset::Latin,
        "inter-normal-400-700-latin.ttf"
    ),
    face!(
        "Inter",
        Family::Ui,
        FaceStyle::Normal,
        400,
        700,
        Subset::LatinExt,
        "inter-normal-400-700-latin-ext.ttf"
    ),
    face!(
        "Inter",
        Family::Ui,
        FaceStyle::Italic,
        400,
        400,
        Subset::Latin,
        "inter-italic-400-latin.ttf"
    ),
    face!(
        "Inter",
        Family::Ui,
        FaceStyle::Italic,
        400,
        400,
        Subset::LatinExt,
        "inter-italic-400-latin-ext.ttf"
    ),
    face!(
        "Inter Display",
        Family::Display,
        FaceStyle::Normal,
        500,
        800,
        Subset::Latin,
        "inter-display-normal-500-800-latin.ttf"
    ),
    face!(
        "Inter Display",
        Family::Display,
        FaceStyle::Normal,
        500,
        800,
        Subset::LatinExt,
        "inter-display-normal-500-800-latin-ext.ttf"
    ),
    face!(
        "Bricolage Grotesque",
        Family::Display,
        FaceStyle::Normal,
        500,
        800,
        Subset::Latin,
        "bricolage-grotesque-normal-500-800-latin.ttf"
    ),
    face!(
        "Bricolage Grotesque",
        Family::Display,
        FaceStyle::Normal,
        500,
        800,
        Subset::LatinExt,
        "bricolage-grotesque-normal-500-800-latin-ext.ttf"
    ),
    face!(
        "Karla",
        Family::Ui,
        FaceStyle::Normal,
        400,
        700,
        Subset::Latin,
        "karla-normal-400-700-latin.ttf"
    ),
    face!(
        "Karla",
        Family::Ui,
        FaceStyle::Normal,
        400,
        700,
        Subset::LatinExt,
        "karla-normal-400-700-latin-ext.ttf"
    ),
    face!(
        "Karla",
        Family::Ui,
        FaceStyle::Italic,
        400,
        400,
        Subset::Latin,
        "karla-italic-400-latin.ttf"
    ),
    face!(
        "Karla",
        Family::Ui,
        FaceStyle::Italic,
        400,
        400,
        Subset::LatinExt,
        "karla-italic-400-latin-ext.ttf"
    ),
    face!(
        "Space Mono",
        Family::Data,
        FaceStyle::Normal,
        400,
        400,
        Subset::Latin,
        "space-mono-normal-400-latin.ttf"
    ),
    face!(
        "Space Mono",
        Family::Data,
        FaceStyle::Normal,
        400,
        400,
        Subset::LatinExt,
        "space-mono-normal-400-latin-ext.ttf"
    ),
    face!(
        "Space Mono",
        Family::Data,
        FaceStyle::Normal,
        700,
        700,
        Subset::Latin,
        "space-mono-normal-700-latin.ttf"
    ),
    face!(
        "Space Mono",
        Family::Data,
        FaceStyle::Normal,
        700,
        700,
        Subset::LatinExt,
        "space-mono-normal-700-latin-ext.ttf"
    ),
    face!(
        "Noto Serif",
        Family::Serif,
        FaceStyle::Normal,
        400,
        700,
        Subset::Latin,
        "noto-serif-normal-400-700-latin.ttf"
    ),
    face!(
        "Noto Serif",
        Family::Serif,
        FaceStyle::Normal,
        400,
        700,
        Subset::LatinExt,
        "noto-serif-normal-400-700-latin-ext.ttf"
    ),
    face!(
        "Noto Serif",
        Family::Serif,
        FaceStyle::Italic,
        400,
        700,
        Subset::Latin,
        "noto-serif-italic-400-700-latin.ttf"
    ),
    face!(
        "Noto Serif",
        Family::Serif,
        FaceStyle::Italic,
        400,
        700,
        Subset::LatinExt,
        "noto-serif-italic-400-700-latin-ext.ttf"
    ),
];

#[cfg(test)]
mod tests {
    use super::{FACES, FaceStyle, Subset};
    use crate::tokens::type_scale::Family;
    use ds_core::word::Word;

    #[test]
    fn every_family_ships_both_subsets() {
        // Each (name, style, weight) appears once per subset, and no file is empty.
        for face in FACES {
            assert!(!face.bytes.is_empty(), "{face:?} is empty");
            let twins = FACES
                .iter()
                .filter(|other| {
                    other.name == face.name
                        && other.style == face.style
                        && other.weight == face.weight
                })
                .map(|other| other.subset)
                .collect::<Vec<_>>();
            assert_eq!(
                twins.len(),
                2,
                "{} {:?} {:?}",
                face.name,
                face.style,
                face.weight
            );
            assert!(twins.contains(&Subset::Latin) && twins.contains(&Subset::LatinExt));
        }
        // Every stack leads with a face that ships upright.
        for family in Family::ALL {
            let name = family.face_name();
            assert!(
                FACES
                    .iter()
                    .any(|face| face.name == name && face.style == FaceStyle::Normal),
                "{family:?} leads with {name}, which ships no upright face"
            );
        }
    }

    /// The key-cap glyphs the shortcuts draw (design/30 section 2.1), each a character of a
    /// shipped face, so a `KeyEquivalent` never depends on the system's fallback.
    #[test]
    fn every_key_cap_glyph_is_in_a_shipped_face() {
        use ds_core::vocab::ShortcutKey;
        use skrifa::{FontRef, MetadataProvider};
        let keys = [
            ShortcutKey::Ctrl,
            ShortcutKey::Shift,
            ShortcutKey::Alt,
            ShortcutKey::Super,
            ShortcutKey::Enter,
            ShortcutKey::Tab,
            ShortcutKey::Backspace,
            ShortcutKey::Up,
            ShortcutKey::Down,
            ShortcutKey::Left,
            ShortcutKey::Right,
            ShortcutKey::Home,
            ShortcutKey::End,
            ShortcutKey::Delete,
            ShortcutKey::PageUp,
            ShortcutKey::PageDown,
        ];
        let inter = FACES
            .iter()
            .find(|face| {
                face.name == "Inter"
                    && face.style == FaceStyle::Normal
                    && face.subset == Subset::Latin
            })
            .expect("the upright Inter latin face ships");
        let charmap = FontRef::new(inter.bytes)
            .expect("Inter is an sfnt")
            .charmap();
        // `↩` is the Return key Mac menus draw; `↵` is what `ShortcutKey::Enter` draws.
        let glyphs = keys
            .iter()
            .flat_map(|key| key.glyph().chars().collect::<Vec<_>>())
            .chain(['\u{21A9}']);
        for glyph in glyphs {
            assert!(
                charmap.map(glyph).is_some(),
                "U+{:04X} ({glyph}) is not in the shipped Inter face",
                u32::from(glyph)
            );
        }
    }

    /// A glyph a face lacks falls to the next family in the stack, so each stack that leads with
    /// another face names Inter next.
    #[test]
    fn every_stack_falls_back_to_inter_before_the_system() {
        use crate::tokens::token::{Token, TokenScope};
        for family in [Family::Display, Family::Ui, Family::Data, Family::Code] {
            let value = family.css_value(TokenScope::BASE);
            let stack: Vec<&str> = value.as_str().split(',').collect();
            let at = stack
                .iter()
                .position(|name| name.trim_matches('"') == "Inter");
            assert!(at.is_some_and(|at| at <= 1), "{family:?}: {value}");
        }
    }

    #[test]
    fn every_face_is_a_font() {
        // An sfnt a font context can register: TrueType outlines (0x00010000) or CFF (`OTTO`).
        // A WOFF2 (`wOF2`) would be skipped by fontique, which reads sfnt only.
        const SFNT: [&[u8]; 2] = [&[0, 1, 0, 0], b"OTTO"];
        for face in FACES {
            let magic = face.bytes.get(..4).unwrap_or_default();
            assert!(
                SFNT.contains(&magic),
                "{} {:?} {:?} starts {magic:02x?}",
                face.name,
                face.style,
                face.subset
            );
        }
    }
}
