//! What a secure field refuses: the keys that would copy or cut its text out. Paste stays.

use crate::edit::keys::{KeyAction, classify};
use chordkit::Keymap;
use dioxus::prelude::{Key, Modifiers};

/// Whether `key` with `modifiers` could copy or cut text out of a field: the platform's copy and
/// cut chords under `keymap`, Ctrl+Insert (copy) and Shift+Delete (cut), and, whatever the
/// keymap says, C or X under Control, Command or Meta. A secure field refuses every chord any
/// layer below might still read as copy or cut: Blitz's own text actions take Control and Command
/// until they resolve through the keymap (FINDINGS "Keys as actions"), and a leak is not undone.
pub(crate) fn takes_text_out(keymap: &Keymap, key: &Key, modifiers: Modifiers) -> bool {
    matches!(
        classify(keymap, key, modifiers),
        KeyAction::Copy | KeyAction::Cut
    ) || any_layers_copy(key, modifiers)
}

/// C or X held with Control, Command (Super) or Meta: a copy or cut to some layer on some platform.
fn any_layers_copy(key: &Key, modifiers: Modifiers) -> bool {
    let commanding = modifiers.intersects(Modifiers::CONTROL | Modifiers::SUPER | Modifiers::META);
    let letter = matches!(key, Key::Character(text) if text.eq_ignore_ascii_case("c") || text.eq_ignore_ascii_case("x"));
    commanding && letter
}

#[cfg(test)]
mod tests {
    use super::takes_text_out;
    use chordkit::{Desktop, Keymap, Platform};
    use dioxus::prelude::{Key, Modifiers};

    type Case = (&'static str, Platform, Key, Modifiers, bool);

    fn ours() -> Platform {
        Platform::Linux {
            desktop: Desktop::Ours,
        }
    }

    #[test]
    fn copy_and_cut_chords_are_refused_and_paste_and_typing_are_not() {
        let none = Modifiers::empty();
        let ctrl = Modifiers::CONTROL;
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let cases: Vec<Case> = vec![
            ("kde ctrl c", kde, Key::Character("c".into()), ctrl, true),
            ("kde ctrl X", kde, Key::Character("X".into()), ctrl, true),
            (
                "ours cmd c",
                ours(),
                Key::Character("c".into()),
                Modifiers::SUPER,
                true,
            ),
            (
                "ours meta x",
                ours(),
                Key::Character("x".into()),
                Modifiers::META,
                true,
            ),
            (
                "ours ctrl c is refused too (Blitz still copies on it)",
                ours(),
                Key::Character("c".into()),
                ctrl,
                true,
            ),
            ("ctrl insert", kde, Key::Insert, ctrl, true),
            ("shift delete", kde, Key::Delete, Modifiers::SHIFT, true),
            ("kde ctrl v", kde, Key::Character("v".into()), ctrl, false),
            ("shift insert", kde, Key::Insert, Modifiers::SHIFT, false),
            ("plain c", kde, Key::Character("c".into()), none, false),
            ("plain delete", kde, Key::Delete, none, false),
            ("kde ctrl a", kde, Key::Character("a".into()), ctrl, false),
        ];
        for (name, platform, key, modifiers, want) in cases {
            let keymap = Keymap::conventional(platform);
            assert_eq!(takes_text_out(&keymap, &key, modifiers), want, "{name}");
        }
    }
}
