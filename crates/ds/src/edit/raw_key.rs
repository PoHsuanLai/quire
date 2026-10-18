//! A key press as an app that draws its own region (a terminal) needs it: which physical key,
//! down or up or repeating, and what it types. Pure: the surface reads the event, and this is the
//! rule that reads the text off it.

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
    /// The text the press types (none on a release): the logical character when no Ctrl or Super
    /// is held, else none.
    pub text: Option<String>,
}

impl RawKey {
    /// The event `phase` of the key `key` (physical `code`).
    pub fn new(
        phase: KeyPhase,
        (key, code): (Key, Code),
        (modifiers, location): (Modifiers, Location),
    ) -> Self {
        let text = match phase {
            KeyPhase::Press | KeyPhase::Repeat => typed(&key, modifiers),
            KeyPhase::Release => None,
        };
        RawKey {
            phase,
            key,
            code,
            modifiers,
            location,
            text,
        }
    }
}

/// What a character key types: itself, unless a command key (Ctrl,
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
    use dioxus::prelude::{Code, Key, Location, Modifiers};

    fn character(text: &str) -> Key {
        Key::Character(text.to_owned())
    }

    fn made(key: Key, modifiers: Modifiers) -> RawKey {
        RawKey::new(
            KeyPhase::Press,
            (key, Code::KeyA),
            (modifiers, Location::Standard),
        )
    }

    #[test]
    fn text_is_the_character_unless_a_chord() {
        let cases: Vec<(&str, RawKey, Option<&str>)> = vec![
            (
                "the character",
                made(character("A"), Modifiers::SHIFT),
                Some("A"),
            ),
            (
                "Alt keeps its character",
                made(character("b"), Modifiers::ALT),
                Some("b"),
            ),
            (
                "Ctrl types nothing",
                made(character("c"), Modifiers::CONTROL),
                None,
            ),
            (
                "Super types nothing",
                made(character("t"), Modifiers::SUPER),
                None,
            ),
            (
                "Meta types nothing",
                made(character("t"), Modifiers::META),
                None,
            ),
            (
                "a named key types nothing",
                made(Key::Enter, Modifiers::empty()),
                None,
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
        );
        assert_eq!(up.text, None);
    }
}
