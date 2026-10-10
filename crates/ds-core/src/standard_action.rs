//! The standard shortcuts: every combination the platform reserves for a meaning, and the few
//! this desktop reserves for itself (design/27-HIG-PARITY.md section 6.2; design/06-INTERACTIONS.md
//! section 2). The actions and their chords are chordkit's (`StandardAction`, its conventions
//! table); this file binds them to the design vocabulary: [`Shortcut::standard`] is an action's
//! chord on this desktop, written in Mac terms (Command is [`ShortcutKey::Super`], drawn `⌘`), and
//! [`Shortcut::custom`] asks chordkit's registration whether an app may take a combination.

use crate::vocab::{Shortcut, ShortcutKey};
use chordkit::{AppAction, AppId, Convention, ConventionTable, Desktop, Keymap, Platform};

pub use chordkit::{Conflict, SpaceNumber, StandardAction};

/// The platform a [`Shortcut`] is written for: this desktop, whose Command key is the Mac's.
pub(crate) const DESIGN_PLATFORM: Platform = Platform::Linux {
    desktop: Desktop::Ours,
};

impl Shortcut {
    /// The keys of a standard action on this desktop: the only way to bind one. Empty for an
    /// action this desktop's conventions leave unbound.
    pub fn standard(action: StandardAction) -> Shortcut {
        match ConventionTable::for_platform(DESIGN_PLATFORM).lookup(action) {
            Some(Convention::Bound(chord)) => {
                Shortcut::from_default_chord(chord).unwrap_or_default()
            }
            Some(Convention::Unbound(_)) | None => Shortcut::default(),
        }
    }

    /// A shortcut of the app's own, modifiers put in the Mac's order; refused with chordkit's
    /// plain-words [`Conflict`] when this desktop keeps the combination for itself or a standard
    /// action has it (design/27 section 6.2: never repurpose one).
    pub fn custom(keys: impl IntoIterator<Item = ShortcutKey>) -> Result<Shortcut, Conflict> {
        let shortcut = Shortcut(normalized(keys));
        let (Some(chord), Some((app, action))) = (shortcut.default_chord(), probe()) else {
            return Ok(shortcut);
        };
        Keymap::conventional(DESIGN_PLATFORM).register(&app, &[(action, chord)])?;
        Ok(shortcut)
    }
}

/// The app and action a combination is tried as.
fn probe() -> Option<(AppId, AppAction)> {
    Some((
        AppId::new("shortcut").ok()?,
        AppAction::new("shortcut.custom").ok()?,
    ))
}

/// `keys` with its modifiers first, deduplicated, in the Mac's order (⌃⌥⇧⌘), then the rest in
/// the order given, letters lower-cased.
pub(crate) fn normalized(keys: impl IntoIterator<Item = ShortcutKey>) -> Vec<ShortcutKey> {
    let keys: Vec<ShortcutKey> = keys
        .into_iter()
        .map(|key| match key {
            ShortcutKey::Char(c) => ShortcutKey::Char(c.to_ascii_lowercase()),
            other => other,
        })
        .collect();
    let mut mods: Vec<(u8, ShortcutKey)> = keys
        .iter()
        .filter_map(|key| key.modifier_rank().map(|rank| (rank, *key)))
        .collect();
    mods.sort_by_key(|(rank, _)| *rank);
    mods.dedup();
    let rest = keys
        .iter()
        .copied()
        .filter(|key| key.modifier_rank().is_none());
    mods.into_iter().map(|(_, key)| key).chain(rest).collect()
}
