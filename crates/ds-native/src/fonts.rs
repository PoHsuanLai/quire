//! Registering `ds::FACES` with the renderer's shared font context, once (spike S11).
//!
//! The faces' bytes are wrapped in blobs once per process, and the base context (system fonts,
//! blitz's list-bullet face and every quire face) is built once and cloned: its collection and
//! source cache are shared, so every window, snapshot and harness document resolves the same
//! registered families without registering or parsing them again.

use parley::FontContext;
use parley::fontique::{Blob, Collection, CollectionOptions, SourceCache};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

/// Every quire face as a blob, made once, so each registration shares one allocation.
static FACE_BLOBS: LazyLock<Vec<Blob<u8>>> = LazyLock::new(|| {
    ds::FACES
        .iter()
        .map(|face| Blob::new(Arc::new(face.bytes) as _))
        .collect()
});

/// The context every document starts from, built on first use.
static BASE: LazyLock<Mutex<FontContext>> = LazyLock::new(|| Mutex::new(base_context()));

/// Register every quire face with `fonts`.
pub fn register_fonts(fonts: &mut parley::FontContext) {
    for blob in FACE_BLOBS.iter() {
        fonts.collection.register_fonts(blob.clone(), None);
    }
}

/// A font context with the system fonts and every quire face registered: what `launch`,
/// `snapshot` and `Harness` hand their documents. Clones share one collection and one source
/// cache, so the faces are registered once per process.
pub fn font_context() -> parley::FontContext {
    BASE.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// System fonts, blitz's bullet face (a document given its own context does not get it
/// otherwise) and the quire faces, in a shared collection and source cache.
fn base_context() -> FontContext {
    let mut fonts = FontContext {
        collection: Collection::new(CollectionOptions {
            shared: true,
            system_fonts: true,
        }),
        source_cache: SourceCache::new_shared(),
    };
    fonts
        .collection
        .register_fonts(Blob::new(Arc::new(blitz_dom::BULLET_FONT) as _), None);
    register_fonts(&mut fonts);
    fonts
}

#[cfg(test)]
mod tests {
    use super::{font_context, register_fonts};
    use ds::{Family, Typeface};

    /// Every family name a token's `font-family` stack leads with, in either typeface: Inter,
    /// Inter Display, Bricolage Grotesque, Karla, Space Mono, Noto Serif.
    fn names() -> Vec<(Family, &'static str)> {
        let mut names: Vec<(Family, &'static str)> = Typeface::ALL
            .into_iter()
            .flat_map(|typeface| {
                Family::ALL
                    .into_iter()
                    .map(move |family| (family, family.face_name(typeface)))
            })
            .collect();
        names.sort_by_key(|(_, name)| *name);
        names.dedup_by_key(|(_, name)| *name);
        names
    }

    #[test]
    fn both_typefaces_name_six_faces() {
        let mut got: Vec<&str> = names().into_iter().map(|(_, name)| name).collect();
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
        let mut fonts = parley::FontContext {
            collection: parley::fontique::Collection::new(parley::fontique::CollectionOptions {
                shared: false,
                system_fonts: false,
            }),
            source_cache: parley::fontique::SourceCache::default(),
        };
        for (family, name) in names() {
            assert!(
                fonts.collection.family_by_name(name).is_none(),
                "{family:?}: {name} before registering"
            );
        }
        register_fonts(&mut fonts);
        for (family, name) in names() {
            assert!(
                fonts.collection.family_by_name(name).is_some(),
                "{family:?}: {name} is not registered"
            );
        }
    }

    #[test]
    fn the_shared_context_carries_the_faces() {
        let mut fonts = font_context();
        for (family, name) in names() {
            assert!(
                fonts.collection.family_by_name(name).is_some(),
                "{family:?}: {name} missing from font_context()"
            );
        }
    }
}
