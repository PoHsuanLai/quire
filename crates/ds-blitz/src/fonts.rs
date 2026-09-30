//! Registering `ds::style::fonts::FACES` with the renderer's shared font context, once (spike S11), through
//! `blitz_kit::fonts::SharedFonts`.
//!
//! The base context (system fonts, blitz's list-bullet face and every quire face) is built once
//! and cloned: its collection and source cache are shared, so every window, snapshot and harness
//! document resolves the same registered families without registering or parsing them again.

use blitz_kit::fonts::{FontFaces, SharedFonts};
use ds::style::fonts::FACES;
use std::sync::LazyLock;

/// The context every document starts from, built on first use.
static BASE: LazyLock<SharedFonts> = LazyLock::new(|| SharedFonts::system_with(&faces()));

/// Every quire face's bytes.
fn faces() -> FontFaces {
    FontFaces(FACES.iter().map(|face| face.bytes).collect())
}

/// A font context with the system fonts and every quire face registered: what `launch`,
/// `snapshot` and `Harness` hand their documents. Clones share one collection and one source
/// cache, so the faces are registered once per process.
pub fn font_context() -> parley::FontContext {
    BASE.for_document()
}

#[cfg(test)]
mod tests {
    use super::{faces, font_context};
    use blitz_kit::fonts::SharedFonts;
    use ds::prelude::*;
    use ds::style::tokens::type_scale::Family;

    /// Every family name a token's `font-family` stack leads with, in either typeface: Inter,
    /// Inter Display, Bricolage Grotesque, Karla, Space Mono, Noto Serif.
    fn names() -> Vec<(Family, String)> {
        let mut names: Vec<(Family, String)> = Typeface::ALL
            .iter()
            .copied()
            .flat_map(|typeface| {
                Family::ALL
                    .iter()
                    .copied()
                    .map(move |family| (family, family.face_name(typeface)))
            })
            .collect();
        names.sort_by_key(|(_, name)| name.clone());
        names.dedup_by_key(|(_, name)| name.clone());
        names
    }

    #[test]
    fn both_typefaces_name_six_faces() {
        let mut got: Vec<String> = names().into_iter().map(|(_, name)| name).collect();
        got.sort_unstable();
        assert_eq!(
            got,
            [
                "Bricolage Grotesque",
                "Inter",
                "Inter Display",
                "Karla",
                "Noto Serif",
                "Space Mono"
            ]
        );
    }

    #[test]
    fn every_family_resolves_after_registration() {
        let mut fonts = SharedFonts::bundled(&faces()).for_document();
        for (family, name) in names() {
            assert!(
                fonts.collection.family_by_name(&name).is_some(),
                "{family:?}: {name} is not registered"
            );
        }
    }

    #[test]
    fn the_shared_context_carries_the_faces() {
        let mut fonts = font_context();
        for (family, name) in names() {
            assert!(
                fonts.collection.family_by_name(&name).is_some(),
                "{family:?}: {name} missing from font_context()"
            );
        }
    }
}
