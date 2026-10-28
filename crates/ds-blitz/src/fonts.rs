//! Registering `ds::style::fonts::FACES` with the renderer's shared font context, once (spike S11), through
//! `blitz_kit::fonts::SharedFonts`.
//!
//! The base context (system fonts with a colour emoji face ahead of the monochrome ones (`quire_emoji`), blitz's
//! list-bullet face and every quire face) is built once
//! and cloned: its collection and source cache are shared, so every window, snapshot and harness
//! document resolves the same registered families without registering or parsing them again.

use blitz_kit::fonts::{FontFaces, SharedFonts};
use ds::style::fonts::FACES;
use quire_emoji::{EmojiChoice, Fonts, prefer_colour};
use std::sync::LazyLock;

/// The context every document starts from, built on first use.
static BASE: LazyLock<SharedFonts> = LazyLock::new(|| {
    let fonts = SharedFonts::system_with(&faces());
    // The collection is shared, so preferring the colour emoji face through one clone
    // changes the generic lists every later clone reads.
    let mut context = fonts.for_document();
    let mut fonts_of = Fonts {
        collection: &mut context.collection,
        source_cache: &mut context.source_cache,
    };
    prefer_colour(&mut fonts_of, &EmojiChoice::from_env());
    fonts
});

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

    /// The Editorial look Extension's sheet: its stacks lead with faces quire ships.
    const EDITORIAL: &str = include_str!("../../../extensions/org.quire.look.Editorial/look.css");

    /// Every family name a `font-family` stack leads with, the standard stacks' and the Editorial
    /// look's: Inter, Inter Display, Bricolage Grotesque, Karla, Space Mono, Noto Serif.
    fn names() -> Vec<String> {
        let mut names: Vec<String> = Family::ALL
            .iter()
            .map(|family| family.face_name())
            .chain(EDITORIAL.lines().filter_map(|line| {
                let stack = line.trim().strip_prefix("--font-")?.split_once(':')?.1;
                Some(
                    stack
                        .trim()
                        .strip_prefix('"')?
                        .split('"')
                        .next()?
                        .to_owned(),
                )
            }))
            .collect();
        names.sort();
        names.dedup();
        names
    }

    #[test]
    fn the_stacks_and_the_editorial_look_name_six_faces() {
        assert_eq!(
            names(),
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
        for name in names() {
            assert!(
                fonts.collection.family_by_name(&name).is_some(),
                "{name} is not registered"
            );
        }
    }

    #[test]
    fn the_shared_context_carries_the_faces() {
        let mut fonts = font_context();
        for name in names() {
            assert!(
                fonts.collection.family_by_name(&name).is_some(),
                "{name} missing from font_context()"
            );
        }
    }
}
