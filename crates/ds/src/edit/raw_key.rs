//! A key press as an app that draws its own region (a terminal) needs it: which physical key,
//! down or up or repeating, what it types, and what it would be unmodified. Pure: the surface
//! reads the event and the host's extras, and this is the rule that puts them together.

use crate::host::keys::KeyExtras;
use dioxus::prelude::{Code, Key, Location, Modifiers};
use ds_core::command::types_text;

/// Where in its press a key is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyPhase {
    /// The key went down.
    Press,
    /// The key is held and the platform repeats it.
    Repeat,
    /// The key came up.
    Release,
}

/// One key event on a raw-keys surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawKey {
    /// Down, repeating or up.
    pub phase: KeyPhase,
    /// The logical key: what the layout makes of the physical one.
    pub key: Key,
    /// The physical key, whatever the layout.
    pub code: Code,
    /// The modifiers held. A modifier key's own press arrives without its bit and its release
    /// with it, because the platform tells the modifiers after the key.
    pub modifiers: Modifiers,
    /// Which of two like keys it is, where the platform says.
    pub location: Location,
    /// The text the press types (none on a release). What the platform reports where it reports any; else the
    /// logical character when no Ctrl or Super is held; else none.
    pub text: Option<String>,
    /// The logical key with no modifier held, where the platform reports it (it depends on the
    /// layout, which the physical code does not). `None` where it does not.
    pub unshifted: Option<Key>,
}

impl RawKey {
    /// The event `phase` of the key `key` (physical `code`), made with `extras` from the host.
    pub fn new(
        phase: KeyPhase,
        (key, code): (Key, Code),
        (modifiers, location): (Modifiers, Location),
        extras: KeyExtras,
    ) -> Self {
        let text = match phase {
            KeyPhase::Press | KeyPhase::Repeat => extras.text.or_else(|| typed(&key, modifiers)),
            KeyPhase::Release => None,
        };
        RawKey {
            phase,
            key,
            code,
            modifiers,
            location,
            text,
            unshifted: extras.unshifted,
        }
    }
}

/// What a character key types when the platform said nothing: itself, unless a command key (Ctrl,
/// Super or Meta) makes it a chord. Alt alone still types (macOS Option, AltGr reported as Alt).
fn typed(key: &Key, modifiers: Modifiers) -> Option<String> {
    match key {
        Key::Character(text) if types_text(key, modifiers - Modifiers::ALT) => Some(text.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyPhase, RawKey};
    use crate::host::keys::KeyExtras;
    use dioxus::prelude::{Code, Key, Location, Modifiers};

    fn character(text: &str) -> Key {
        Key::Character(text.to_owned())
    }

    fn made(key: Key, modifiers: Modifiers, extras: KeyExtras) -> RawKey {
        RawKey::new(
            KeyPhase::Press,
            (key, Code::KeyA),
            (modifiers, Location::Standard),
            extras,
        )
    }

    #[test]
    fn text_is_the_platforms_when_given_else_the_character_unless_a_chord() {
        let said = |text: &str| KeyExtras {
            text: Some(text.to_owned()),
            unshifted: None,
        };
        let cases: Vec<(&str, RawKey, Option<&str>)> = vec![
            (
                "no extras: the character",
                made(character("A"), Modifiers::SHIFT, KeyExtras::default()),
                Some("A"),
            ),
            (
                "no extras, Alt keeps its character",
                made(character("b"), Modifiers::ALT, KeyExtras::default()),
                Some("b"),
            ),
            (
                "no extras, Ctrl types nothing",
                made(character("c"), Modifiers::CONTROL, KeyExtras::default()),
                None,
            ),
            (
                "no extras, Super types nothing",
                made(character("t"), Modifiers::SUPER, KeyExtras::default()),
                None,
            ),
            (
                "no extras, Meta types nothing",
                made(character("t"), Modifiers::META, KeyExtras::default()),
                None,
            ),
            (
                "no extras, a named key types nothing",
                made(Key::Enter, Modifiers::empty(), KeyExtras::default()),
                None,
            ),
            (
                "the platform's text wins (a dead key)",
                made(character("e"), Modifiers::empty(), said("é")),
                Some("é"),
            ),
            (
                "the platform's text wins over a chord's silence",
                made(character("a"), Modifiers::CONTROL, said("\u{1}")),
                Some("\u{1}"),
            ),
        ];
        for (name, key, want) in cases {
            assert_eq!(key.text.as_deref(), want, "{name}");
        }
    }

    #[test]
    fn a_release_types_nothing() {
        let up = RawKey::new(
            KeyPhase::Release,
            (character("a"), Code::KeyA),
            (Modifiers::empty(), Location::Standard),
            KeyExtras {
                text: Some("a".to_owned()),
                unshifted: None,
            },
        );
        assert_eq!(up.text, None);
    }

    #[test]
    fn the_unshifted_key_is_only_what_the_platform_reported() {
        let none = made(character("!"), Modifiers::SHIFT, KeyExtras::default());
        assert_eq!(none.unshifted, None);
        let given = made(
            character("!"),
            Modifiers::SHIFT,
            KeyExtras {
                text: None,
                unshifted: Some(character("1")),
            },
        );
        assert_eq!(given.unshifted, Some(character("1")));
        assert_eq!(given.text.as_deref(), Some("!"));
    }
}
