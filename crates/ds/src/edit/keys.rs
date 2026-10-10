//! Reading a key press on the surface: text to insert, a clipboard gesture, or a key for the
//! app. Pure, so the table below is the whole rule. Which chord is a paste is the keymap's
//! (Command+V on a Mac and our desktop, Ctrl+V elsewhere); the CUA clipboard keys every platform
//! shares (Shift+Insert, Ctrl+Insert, Shift+Delete) are read as they are.

use crate::edit::input::KeyInput;
use chordkit::{Action, Context, Keymap, StandardAction};
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::{resolve, types_text};

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

/// What pressing `key` with `modifiers` held means under `keymap`. A character with no Ctrl or
/// Super types; Alt alone does not make a chord, since it types characters on some layouts
/// (macOS Option, AltGr reported as Alt). A chord the keymap binds to Paste, Cut or Copy is that
/// gesture; any other is the app's.
pub fn classify(keymap: &Keymap, key: &Key, modifiers: Modifiers) -> KeyAction {
    if let Some(gesture) = shared_clipboard_key(key, modifiers) {
        return gesture;
    }
    if let Key::Character(text) = key
        && types_text(key, modifiers - Modifiers::ALT)
    {
        return KeyAction::Text(text.clone());
    }
    match resolve(keymap, key, modifiers, Context::TextEntry) {
        Some(Action::Standard(StandardAction::Paste)) => KeyAction::Paste,
        Some(Action::Standard(StandardAction::Cut)) => KeyAction::Cut,
        Some(Action::Standard(StandardAction::Copy)) => KeyAction::Copy,
        _ => chord(key, modifiers),
    }
}

/// Shift+Insert pastes, Ctrl+Insert copies and Shift+Delete cuts on every platform.
fn shared_clipboard_key(key: &Key, modifiers: Modifiers) -> Option<KeyAction> {
    match key {
        Key::Insert if modifiers == Modifiers::SHIFT => Some(KeyAction::Paste),
        Key::Insert if modifiers == Modifiers::CONTROL => Some(KeyAction::Copy),
        Key::Delete if modifiers == Modifiers::SHIFT => Some(KeyAction::Cut),
        _ => None,
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
    use chordkit::{Desktop, Keymap, Platform};
    use dioxus::prelude::{Key, Modifiers};

    fn text(s: &str) -> Key {
        Key::Character(s.to_owned())
    }

    fn key(key: Key, modifiers: Modifiers) -> KeyAction {
        KeyAction::Key(KeyInput { key, modifiers })
    }

    fn ours() -> Platform {
        Platform::Linux {
            desktop: Desktop::Ours,
        }
    }

    fn kde() -> Platform {
        Platform::Linux {
            desktop: Desktop::Kde,
        }
    }

    type Case = (Platform, Key, Modifiers, KeyAction);

    #[test]
    fn keys_read_as_text_gestures_or_keys_on_each_platform() {
        let plain = Modifiers::empty();
        let cases: Vec<Case> = vec![
            (ours(), text("a"), plain, KeyAction::Text("a".to_owned())),
            (
                ours(),
                text("A"),
                Modifiers::SHIFT,
                KeyAction::Text("A".to_owned()),
            ),
            (ours(), text(" "), plain, KeyAction::Text(" ".to_owned())),
            (
                ours(),
                text("å"),
                Modifiers::ALT,
                KeyAction::Text("å".to_owned()),
            ),
            (
                ours(),
                text("你好"),
                plain,
                KeyAction::Text("你好".to_owned()),
            ),
            (ours(), text("v"), Modifiers::SUPER, KeyAction::Paste),
            (ours(), text("V"), Modifiers::META, KeyAction::Paste),
            (ours(), text("x"), Modifiers::SUPER, KeyAction::Cut),
            (ours(), text("c"), Modifiers::SUPER, KeyAction::Copy),
            (
                ours(),
                text("v"),
                Modifiers::CONTROL,
                key(text("v"), Modifiers::CONTROL),
            ),
            (kde(), text("v"), Modifiers::CONTROL, KeyAction::Paste),
            (kde(), text("x"), Modifiers::CONTROL, KeyAction::Cut),
            (kde(), text("c"), Modifiers::CONTROL, KeyAction::Copy),
            (
                kde(),
                text("v"),
                Modifiers::SUPER,
                key(text("v"), Modifiers::SUPER),
            ),
            (
                kde(),
                text("b"),
                Modifiers::CONTROL,
                key(text("b"), Modifiers::CONTROL),
            ),
            (ours(), Key::Insert, Modifiers::SHIFT, KeyAction::Paste),
            (kde(), Key::Insert, Modifiers::CONTROL, KeyAction::Copy),
            (kde(), Key::Delete, Modifiers::SHIFT, KeyAction::Cut),
            (ours(), Key::Enter, plain, key(Key::Enter, plain)),
            (
                ours(),
                Key::Enter,
                Modifiers::SHIFT,
                key(Key::Enter, Modifiers::SHIFT),
            ),
            (ours(), Key::Backspace, plain, key(Key::Backspace, plain)),
            (ours(), Key::Delete, plain, key(Key::Delete, plain)),
            (ours(), Key::Tab, plain, key(Key::Tab, plain)),
        ];
        for (platform, pressed, held, expected) in cases {
            let keymap = Keymap::conventional(platform);
            assert_eq!(
                classify(&keymap, &pressed, held),
                expected,
                "{platform:?} {pressed:?} with {held:?}"
            );
        }
    }
}
