//! A colour emoji face ahead of the monochrome ones in every generic family (FINDINGS "Colour
//! emoji").
//!
//! Parley takes each emoji cluster from the first family of the element's stack, then the
//! `emoji` generic, that maps it. `system-ui` and `sans-serif` expand to the whole fontconfig
//! match list, which on the reference machine reaches Symbola (monochrome, no flags, no ZWJ
//! sequences) before the `emoji` generic is ever consulted. So the colour face is inserted into
//! each generic list just before its first family that maps an emoji: text fonts earlier in the
//! list still map the letters, digits and spaces (Noto Color Emoji maps digits to empty glyphs).
//! The face is detected, not named: the one `QUIRE_EMOJI_FAMILY` names, else the first family of
//! the `emoji` generic that carries a COLR table (CBDT draws nothing on our renderers), else the
//! COLRv1 file the installer or a distro package left on disk (`emoji_file`), registered here.

use crate::emoji_file::colrv1_emoji_file;
use blitz_dom::FontContext;
use parley::fontique::{FamilyId, GenericFamily, QueryFamily, QueryFont, QueryStatus};
use std::sync::Arc;

/// The environment variable naming the colour emoji family to use instead of the detected one.
pub const EMOJI_FAMILY_ENV: &str = "QUIRE_EMOJI_FAMILY";

/// A character every emoji font maps and a text font does not.
const PROBE: char = '\u{1F600}';

/// The generics a stack reaches monochrome emoji through.
const GENERICS: [GenericFamily; 10] = [
    GenericFamily::SystemUi,
    GenericFamily::UiSansSerif,
    GenericFamily::SansSerif,
    GenericFamily::Serif,
    GenericFamily::UiSerif,
    GenericFamily::Monospace,
    GenericFamily::UiMonospace,
    GenericFamily::UiRounded,
    GenericFamily::Cursive,
    GenericFamily::Fantasy,
];

/// Which family paints colour emoji.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EmojiChoice {
    /// The first COLR family the system lists for `emoji`, else a COLRv1 font file on disk.
    #[default]
    Detect,
    /// This family, trusted without a table check.
    Named(String),
}

impl EmojiChoice {
    /// [`EMOJI_FAMILY_ENV`] if set and not empty, else [`EmojiChoice::Detect`].
    pub fn from_env() -> EmojiChoice {
        match std::env::var(EMOJI_FAMILY_ENV) {
            Ok(name) if !name.is_empty() => EmojiChoice::Named(name),
            _ => EmojiChoice::Detect,
        }
    }
}

/// Put the colour face ahead of the first emoji-mapping family of every generic and first in
/// the `emoji` generic. Does nothing where no colour face is found.
pub(crate) fn prefer_colour(ctx: &mut FontContext, choice: &EmojiChoice) {
    let Some(colour) = colour_face(ctx, choice) else {
        return;
    };
    for generic in GENERICS {
        let list: Vec<FamilyId> = ctx.collection.generic_families(generic).collect();
        let covers: Vec<bool> = list.iter().map(|id| maps_emoji(ctx, *id)).collect();
        let merged = insert_before_first(&list, &covers, colour);
        ctx.collection
            .set_generic_families(generic, merged.into_iter());
    }
    let emoji: Vec<FamilyId> = ctx
        .collection
        .generic_families(GenericFamily::Emoji)
        .collect();
    let covers = vec![true; emoji.len()];
    let merged = insert_before_first(&emoji, &covers, colour);
    ctx.collection
        .set_generic_families(GenericFamily::Emoji, merged.into_iter());
}

/// `list` with `face` once, just before the first entry whose `covers` flag is set (at the end
/// if none is). Unchanged where `face` already comes before every covering entry.
fn insert_before_first(list: &[FamilyId], covers: &[bool], face: FamilyId) -> Vec<FamilyId> {
    let at = covers.iter().position(|covered| *covered);
    let face_first = at.is_some_and(|at| list[at] == face);
    match at {
        Some(_) if face_first => list.to_vec(),
        _ => {
            let at = at.unwrap_or(list.len());
            let (before, after) = list.split_at(at);
            let others = |id: &FamilyId| *id != face;
            before
                .iter()
                .copied()
                .filter(others)
                .chain(std::iter::once(face))
                .chain(after.iter().copied().filter(others))
                .collect()
        }
    }
}

/// The family to paint colour emoji with: the one `choice` names, else the first COLR family of
/// the `emoji` generic, else the COLRv1 file [`colrv1_emoji_file`] finds, registered.
fn colour_face(ctx: &mut FontContext, choice: &EmojiChoice) -> Option<FamilyId> {
    match choice {
        EmojiChoice::Named(name) => ctx.collection.family_id(name),
        EmojiChoice::Detect => system_colour_face(ctx).or_else(|| register_colrv1_file(ctx)),
    }
}

fn system_colour_face(ctx: &mut FontContext) -> Option<FamilyId> {
    let emoji: Vec<FamilyId> = ctx
        .collection
        .generic_families(GenericFamily::Emoji)
        .collect();
    emoji
        .into_iter()
        .find(|id| first_font(ctx, *id, |font| has_table(font, *b"COLR")))
}

/// The family of the COLRv1 file on this machine, registered with the collection. The
/// registered family shadows a system family of the same name (the bitmap build's), so a stack
/// that names `Noto Color Emoji` reaches the colour face too.
fn register_colrv1_file(ctx: &mut FontContext) -> Option<FamilyId> {
    let bytes = std::fs::read(colrv1_emoji_file()?).ok()?;
    let blob = parley::fontique::Blob::new(Arc::new(bytes) as _);
    let families = ctx.collection.register_fonts(blob, None);
    families.into_iter().next().map(|(id, _)| id)
}

fn maps_emoji(ctx: &mut FontContext, id: FamilyId) -> bool {
    first_font(ctx, id, |font| {
        font.charmap()
            .and_then(|charmap| charmap.map(PROBE))
            .is_some_and(|glyph| glyph != 0)
    })
}

/// What `read` says of the family's best-matching font; false where it does not load.
fn first_font(ctx: &mut FontContext, id: FamilyId, read: impl Fn(&QueryFont) -> bool) -> bool {
    let mut query = ctx.collection.query(&mut ctx.source_cache);
    query.set_families([QueryFamily::Id(id)]);
    let mut found = false;
    query.matches_with(|font| {
        found = read(font);
        QueryStatus::Stop
    });
    found
}

fn has_table(font: &QueryFont, tag: [u8; 4]) -> bool {
    crate::sfnt::table_listed(font.blob.data(), font.index, tag)
}

#[cfg(test)]
mod tests {
    use super::insert_before_first;
    use parley::fontique::FamilyId;

    #[test]
    fn the_colour_face_goes_before_the_first_family_that_maps_emoji() {
        let [text, more, mono, colour] = std::array::from_fn(|_| FamilyId::new());
        let list = [text, more, mono, colour];
        let covers = [false, false, true, true];
        assert_eq!(
            insert_before_first(&list, &covers, colour),
            [text, more, colour, mono]
        );
        let covers = [true, false, false, false];
        assert_eq!(
            insert_before_first(&list, &covers, colour),
            [colour, text, more, mono]
        );
        let covers = [false; 4];
        assert_eq!(
            insert_before_first(&list, &covers, colour),
            [text, more, mono, colour]
        );
    }

    #[test]
    fn a_list_already_led_by_the_colour_face_is_unchanged() {
        let [colour, text] = std::array::from_fn(|_| FamilyId::new());
        assert_eq!(
            insert_before_first(&[text, colour], &[false, true], colour),
            [text, colour]
        );
    }
}
