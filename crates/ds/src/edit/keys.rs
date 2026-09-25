//! Reading a key press on the surface: text to insert, a clipboard gesture, or a key for the
//! app. Pure, so the table below is the whole rule.

use crate::edit::input::KeyInput;
use dioxus::prelude::{Key, Modifiers};

/// What a key press on the surface means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// Insert this text.
    Text(String),
    /// Paste the clipboard.
    Paste,
    /// Cut the selection.
    Cut,
    /// Copy the selection.
    Copy,
    /// Hand the key to the app.
    Key(KeyInput),
}

/// What pressing `key` with `modifiers` held means. Ctrl or Super (Cmd) makes a chord; Alt alone
/// does not, since it types characters on some layouts (macOS Option, AltGr reported as Alt).
pub fn classify(key: &Key, modifiers: Modifiers) -> KeyAction {
    let command = modifiers.intersects(Modifiers::CONTROL | Modifiers::META);
    let shift = modifiers.contains(Modifiers::SHIFT);
    match key {
        Key::Character(text) if command => match text.to_lowercase().as_str() {
            "v" => KeyAction::Paste,
            "x" => KeyAction::Cut,
            "c" => KeyAction::Copy,
            _ => chord(key, modifiers),
        },
        Key::Character(text) => KeyAction::Text(text.clone()),
        Key::Insert if shift => KeyAction::Paste,
        Key::Insert if command => KeyAction::Copy,
        Key::Delete if shift => KeyAction::Cut,
        _ => chord(key, modifiers),
    }
}

fn chord(key: &Key, modifiers: Modifiers) -> KeyAction {
    KeyAction::Key(KeyInput {
        key: key.clone(),
        modifiers,
    })
}

#[cfg(test)]
mod tests {
    use super::{KeyAction, classify};
    use crate::edit::input::KeyInput;
    use dioxus::prelude::{Key, Modifiers};

    fn text(s: &str) -> Key {
        Key::Character(s.to_owned())
    }

    fn key(key: Key, modifiers: Modifiers) -> KeyAction {
        KeyAction::Key(KeyInput { key, modifiers })
    }

    #[test]
    fn keys_read_as_text_gestures_or_keys() {
        let cases = [
            (
                text("a"),
                Modifiers::empty(),
                KeyAction::Text("a".to_owned()),
            ),
            (text("A"), Modifiers::SHIFT, KeyAction::Text("A".to_owned())),
            (
                text(" "),
                Modifiers::empty(),
                KeyAction::Text(" ".to_owned()),
            ),
            (text("å"), Modifiers::ALT, KeyAction::Text("å".to_owned())),
            (text("v"), Modifiers::CONTROL, KeyAction::Paste),
            (text("V"), Modifiers::META, KeyAction::Paste),
            (text("x"), Modifiers::CONTROL, KeyAction::Cut),
            (text("c"), Modifiers::CONTROL, KeyAction::Copy),
            (Key::Insert, Modifiers::SHIFT, KeyAction::Paste),
            (Key::Insert, Modifiers::CONTROL, KeyAction::Copy),
            (Key::Delete, Modifiers::SHIFT, KeyAction::Cut),
            (
                text("b"),
                Modifiers::CONTROL,
                key(text("b"), Modifiers::CONTROL),
            ),
            (
                Key::Enter,
                Modifiers::empty(),
                key(Key::Enter, Modifiers::empty()),
            ),
            (
                Key::Enter,
                Modifiers::SHIFT,
                key(Key::Enter, Modifiers::SHIFT),
            ),
            (
                Key::Backspace,
                Modifiers::empty(),
                key(Key::Backspace, Modifiers::empty()),
            ),
            (
                Key::Delete,
                Modifiers::empty(),
                key(Key::Delete, Modifiers::empty()),
            ),
            (
                Key::Tab,
                Modifiers::empty(),
                key(Key::Tab, Modifiers::empty()),
            ),
        ];
        for (pressed, held, expected) in cases {
            assert_eq!(
                classify(&pressed, held),
                expected,
                "{pressed:?} with {held:?}"
            );
        }
    }
}
