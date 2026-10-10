//! A [`Shortcut`] as chordkit's portable chord and back. The shortcut is written in Mac terms
//! (Command is [`ShortcutKey::Super`]); chordkit says what that is on each platform, so
//! Command here is `Primary` there and the other modifiers stay what they are.

use crate::standard_action::normalized;
use crate::vocab::{Shortcut, ShortcutKey};
use chordkit::{Chord, DefaultChord, Key, Modifier, Modifiers, NamedKey, PrimaryUse};

impl Shortcut {
    /// The portable chord this shortcut is: Command is `Primary`. `None` for no key at all, or
    /// a key chordkit has no name for (the Menu key).
    pub fn default_chord(&self) -> Option<DefaultChord> {
        let keys = self.keys();
        let primary = if keys.contains(&ShortcutKey::Super) {
            PrimaryUse::Held
        } else {
            PrimaryUse::Unused
        };
        let modifiers: Modifiers = keys.iter().filter_map(|key| modifier_of(*key)).collect();
        let key = keys.iter().find(|key| key.modifier_rank().is_none())?;
        Some(DefaultChord::new(
            primary,
            Chord::new(modifiers, chord_key(*key)?),
        ))
    }

    /// The shortcut a portable chord is, written in Mac terms; `None` for a key the shortcut
    /// vocabulary has no cap for (function keys).
    pub fn from_default_chord(chord: DefaultChord) -> Option<Shortcut> {
        let base = chord.base();
        let held = match chord.primary() {
            PrimaryUse::Held => Some(Modifier::Super),
            PrimaryUse::Unused => None,
        };
        let modifiers = base.modifiers().iter().chain(held).map(shortcut_modifier);
        let keys = modifiers.chain([shortcut_key(base.key())?]);
        Some(Shortcut(normalized(keys)))
    }
}

fn modifier_of(key: ShortcutKey) -> Option<Modifier> {
    match key {
        ShortcutKey::Ctrl => Some(Modifier::Ctrl),
        ShortcutKey::Alt => Some(Modifier::Alt),
        ShortcutKey::Shift => Some(Modifier::Shift),
        _ => None,
    }
}

fn shortcut_modifier(modifier: Modifier) -> ShortcutKey {
    match modifier {
        Modifier::Ctrl => ShortcutKey::Ctrl,
        Modifier::Alt => ShortcutKey::Alt,
        Modifier::Shift => ShortcutKey::Shift,
        Modifier::Super | Modifier::Meta => ShortcutKey::Super,
    }
}

/// Each named key chordkit and the shortcut vocabulary both have.
const NAMED: [(ShortcutKey, NamedKey); 15] = [
    (ShortcutKey::Space, NamedKey::Space),
    (ShortcutKey::Enter, NamedKey::Enter),
    (ShortcutKey::Escape, NamedKey::Escape),
    (ShortcutKey::Tab, NamedKey::Tab),
    (ShortcutKey::Backspace, NamedKey::Backspace),
    (ShortcutKey::Up, NamedKey::Up),
    (ShortcutKey::Down, NamedKey::Down),
    (ShortcutKey::Left, NamedKey::Left),
    (ShortcutKey::Right, NamedKey::Right),
    (ShortcutKey::Home, NamedKey::Home),
    (ShortcutKey::End, NamedKey::End),
    (ShortcutKey::Delete, NamedKey::Delete),
    (ShortcutKey::PageUp, NamedKey::PageUp),
    (ShortcutKey::PageDown, NamedKey::PageDown),
    (ShortcutKey::Insert, NamedKey::Insert),
];

fn chord_key(key: ShortcutKey) -> Option<Key> {
    match key {
        ShortcutKey::Char(c) => Some(Key::character(c)),
        other => NAMED
            .iter()
            .find(|(shortcut, _)| *shortcut == other)
            .map(|(_, named)| Key::Named(*named)),
    }
}

fn shortcut_key(key: Key) -> Option<ShortcutKey> {
    match key {
        Key::Char(c) => Some(ShortcutKey::Char(c)),
        Key::Named(named) => NAMED
            .iter()
            .find(|(_, candidate)| *candidate == named)
            .map(|(shortcut, _)| *shortcut),
        Key::Tap(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> DefaultChord {
        text.parse().unwrap_or_else(|e| panic!("{text}: {e}"))
    }

    #[test]
    fn command_is_primary_and_the_other_modifiers_stay() {
        let cases = [
            (
                Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('K')]),
                "Primary+K",
            ),
            (
                Shortcut(vec![
                    ShortcutKey::Shift,
                    ShortcutKey::Super,
                    ShortcutKey::Char('z'),
                ]),
                "Primary+Shift+Z",
            ),
            (
                Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Char('1')]),
                "Ctrl+1",
            ),
            (
                Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('+')]),
                "Primary++",
            ),
            (Shortcut(vec![ShortcutKey::Enter]), "Enter"),
        ];
        for (shortcut, text) in cases {
            assert_eq!(shortcut.default_chord(), Some(parsed(text)), "{text}");
        }
        assert_eq!(Shortcut::default().default_chord(), None);
        assert_eq!(
            Shortcut(vec![ShortcutKey::ContextMenu]).default_chord(),
            None
        );
    }

    #[test]
    fn a_chord_round_trips_through_a_shortcut() {
        for text in [
            "Primary+Shift+Z",
            "Ctrl+Primary+S",
            "Ctrl+Up",
            "Alt+Primary+Escape",
        ] {
            let chord = parsed(text);
            let shortcut = Shortcut::from_default_chord(chord);
            assert_eq!(
                shortcut.and_then(|s| s.default_chord()),
                Some(chord),
                "{text}"
            );
        }
        assert_eq!(Shortcut::from_default_chord(parsed("F3")), None);
    }
}
