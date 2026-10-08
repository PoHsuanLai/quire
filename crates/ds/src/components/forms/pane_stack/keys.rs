//! What a key means to a pane stack, pure: the keys that go back one page.

use crate::edit::keys::command_keys;
use dioxus::prelude::{Key, Modifiers};

/// Whether `key` with `modifiers` asks to go back: Escape, the command chord `[` (the `Super` key
/// is the host's command key, as everywhere in the catalogue) and Alt+Left.
pub(crate) fn goes_back(key: &Key, modifiers: Modifiers) -> bool {
    match key {
        Key::Escape => modifiers.is_empty(),
        Key::Character(text) if text == "[" => command_keys(modifiers) == Modifiers::META,
        Key::ArrowLeft => modifiers == Modifiers::ALT,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::goes_back;
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn only_the_back_chords_go_back() {
        let bracket = || Key::Character("[".into());
        let cases = [
            ("escape", Key::Escape, Modifiers::empty(), true),
            ("command bracket", bracket(), Modifiers::META, true),
            ("alt left", Key::ArrowLeft, Modifiers::ALT, true),
            (
                "bare bracket types a bracket",
                bracket(),
                Modifiers::empty(),
                false,
            ),
            (
                "bare left moves a caret",
                Key::ArrowLeft,
                Modifiers::empty(),
                false,
            ),
            ("shift escape", Key::Escape, Modifiers::SHIFT, false),
            (
                "command right bracket",
                Key::Character("]".into()),
                Modifiers::META,
                false,
            ),
        ];
        for (name, key, modifiers, want) in cases {
            assert_eq!(goes_back(&key, modifiers), want, "{name}");
        }
    }
}
