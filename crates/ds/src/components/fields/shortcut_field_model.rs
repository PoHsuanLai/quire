//! What the shortcut field decides from a key while it records: the pure step, apart from the
//! view that carries it out.

use chordkit::{Chord, Key as ChordKey, Modifier, Modifiers as ChordModifiers, NamedKey, Platform};
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::{chord_of, modifiers_of};
use ds_core::word::Word;

/// What one recording ended with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShortcutRecorded {
    /// The person typed this combination.
    Chord(Chord),
    /// The person cleared the shortcut (Backspace or Delete).
    Cleared,
    /// The person gave up: Escape, or the keyboard went elsewhere.
    Cancelled,
}

/// Another action that already uses the combination in the field.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ShortcutClash {
    /// Nothing else uses it.
    #[default]
    None,
    /// The action with this name does; the field says so under itself.
    With(String),
}

/// Whether the field is listening for a combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub(crate) enum Capture {
    /// Showing its shortcut.
    #[default]
    Idle,
    /// Reading the next combination.
    Listening,
}

/// What a key means to a field that is listening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Heard {
    /// A combination to offer.
    Offer(Chord),
    /// Clear the shortcut.
    Clear,
    /// Stop listening.
    Cancel,
    /// Not a shortcut yet (a lone modifier, a plain letter): the field keeps the key and goes on
    /// listening.
    Wait,
    /// Tab moving the keyboard on: not the field's.
    Leave,
}

/// What `key` with `modifiers` held means to a field listening on `platform`. Escape, Backspace,
/// Delete and Tab are meant bare; with a modifier they are combinations like any other.
pub(crate) fn hear(platform: Platform, key: &Key, modifiers: Modifiers) -> Heard {
    let bare = modifiers_of(modifiers).is_empty();
    match (key, bare) {
        (Key::Escape, true) => Heard::Cancel,
        (Key::Backspace | Key::Delete, true) => Heard::Clear,
        (Key::Tab, true) => Heard::Leave,
        _ => match chord_of(platform, key, modifiers).filter(is_shortcut) {
            Some(chord) => Heard::Offer(chord),
            None => Heard::Wait,
        },
    }
}

/// Whether `chord` is something to bind: it holds a modifier (Shift alone with a character is
/// typing), or it is a function key.
fn is_shortcut(chord: &Chord) -> bool {
    let modifiers = chord.modifiers();
    let typing = modifiers == ChordModifiers::of(Modifier::Shift)
        && matches!(chord.key(), ChordKey::Char(_));
    let function = matches!(chord.key(), ChordKey::Named(NamedKey::F(_)));
    function || (!modifiers.is_empty() && !typing)
}

#[cfg(test)]
mod tests {
    use super::{Heard, hear};
    use chordkit::{
        Chord, Desktop, Key as ChordKey, Modifier, Modifiers as ChordModifiers, NamedKey, Platform,
    };
    use dioxus::prelude::{Key, Modifiers};

    fn ours() -> Platform {
        Platform::Linux {
            desktop: Desktop::Ours,
        }
    }

    fn chord(modifiers: &[Modifier], key: ChordKey) -> Heard {
        Heard::Offer(Chord::new(
            modifiers.iter().copied().collect::<ChordModifiers>(),
            key,
        ))
    }

    #[test]
    fn a_key_is_a_combination_a_command_or_nothing_yet() {
        let letter = |c: &str| Key::Character(c.to_owned());
        let cases: Vec<(&str, Key, Modifiers, Heard)> = vec![
            (
                "escape cancels",
                Key::Escape,
                Modifiers::empty(),
                Heard::Cancel,
            ),
            (
                "backspace clears",
                Key::Backspace,
                Modifiers::empty(),
                Heard::Clear,
            ),
            (
                "delete clears",
                Key::Delete,
                Modifiers::empty(),
                Heard::Clear,
            ),
            ("tab leaves", Key::Tab, Modifiers::empty(), Heard::Leave),
            (
                "caps lock does not make escape a combination",
                Key::Escape,
                Modifiers::CAPS_LOCK,
                Heard::Cancel,
            ),
            (
                "a bare letter waits",
                letter("k"),
                Modifiers::empty(),
                Heard::Wait,
            ),
            (
                "shift and a letter is typing",
                letter("K"),
                Modifiers::SHIFT,
                Heard::Wait,
            ),
            (
                "a lone modifier waits",
                Key::Shift,
                Modifiers::SHIFT,
                Heard::Wait,
            ),
            (
                "a bare arrow waits",
                Key::ArrowUp,
                Modifiers::empty(),
                Heard::Wait,
            ),
            (
                "command and a letter",
                letter("k"),
                Modifiers::SUPER,
                chord(&[Modifier::Super], ChordKey::Char('k')),
            ),
            (
                "shift command and a letter",
                letter("K"),
                Modifiers::SUPER | Modifiers::SHIFT,
                chord(&[Modifier::Super, Modifier::Shift], ChordKey::Char('k')),
            ),
            (
                "control and an arrow",
                Key::ArrowLeft,
                Modifiers::CONTROL,
                chord(&[Modifier::Ctrl], ChordKey::Named(NamedKey::Left)),
            ),
            (
                "a function key needs no modifier",
                Key::F5,
                Modifiers::empty(),
                chord(&[], ChordKey::Named(NamedKey::F(5))),
            ),
            (
                "option escape is a combination",
                Key::Escape,
                Modifiers::ALT,
                chord(&[Modifier::Alt], ChordKey::Named(NamedKey::Escape)),
            ),
        ];
        for (name, key, modifiers, want) in cases {
            assert_eq!(hear(ours(), &key, modifiers), want, "{name}");
        }
    }
}
