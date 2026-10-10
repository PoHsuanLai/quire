//! Chords and actions drawn without a runtime or a `Shortcut`: pure functions of a platform (or a
//! keymap) and chordkit's own types, for code that has neither a window nor a `Keys` handle (a
//! command panel's hints, a help sheet built in a worker). They draw as `shortcut_text` does for
//! a chord the keymap binds: `⌘C` on a Mac-style platform, `Ctrl+C` in a word-style one.

use super::shortcut_text::{KeyCap, arrow, split_caps};
use chordkit::{Action, Chord, Keymap, Platform};

/// `chord` as `platform` shows it: `⇧⌘Z` on a Mac or our desktop, `Ctrl+Shift+Z` elsewhere.
pub fn chord_text(platform: Platform, chord: &Chord) -> String {
    chord.display(platform)
}

/// The caps `chord` is drawn on for `platform`: the modifiers, then the key.
pub fn chord_caps(platform: Platform, chord: &Chord) -> Vec<KeyCap> {
    caps_of(&chord_text(platform, chord))
}

/// `action` as `keymap`'s platform shows its first (best) chord in window content; `None` when
/// the keymap binds it to nothing.
pub fn shortcut_text_for(keymap: &Keymap, action: &Action) -> Option<String> {
    let chord = keymap.chords_of(action).into_iter().next()?;
    Some(chord_text(keymap.platform(), &chord))
}

/// The caps of [`shortcut_text_for`]; empty when the keymap binds `action` to nothing.
pub fn shortcut_caps_for(keymap: &Keymap, action: &Action) -> Vec<KeyCap> {
    shortcut_text_for(keymap, action)
        .map(|text| caps_of(&text))
        .unwrap_or_default()
}

fn caps_of(text: &str) -> Vec<KeyCap> {
    split_caps(text)
        .into_iter()
        .map(|text| KeyCap {
            kind: arrow(&text),
            text,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordkit::{Desktop, StandardAction};

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

    #[test]
    fn an_actions_text_follows_the_platform() {
        let copy = Action::Standard(StandardAction::Copy);
        let cases = [
            ("mac", Platform::MacOs, "⌘C"),
            ("ours", ours(), "⌘C"),
            ("windows", Platform::Windows, "Ctrl+C"),
            ("kde", kde(), "Ctrl+C"),
        ];
        for (name, platform, want) in cases {
            let keymap = Keymap::conventional(platform);
            assert_eq!(
                shortcut_text_for(&keymap, &copy).as_deref(),
                Some(want),
                "{name}"
            );
        }
    }

    #[test]
    fn a_chords_text_and_caps_follow_the_platform() {
        let Ok(chord) = "Ctrl+Shift+Z".parse::<Chord>() else {
            panic!("a chord");
        };
        let caps = |platform| -> Vec<String> {
            chord_caps(platform, &chord)
                .into_iter()
                .map(|cap| cap.text)
                .collect()
        };
        assert_eq!(chord_text(Platform::Windows, &chord), "Ctrl+Shift+Z");
        assert_eq!(caps(Platform::Windows), ["Ctrl", "Shift", "Z"]);
        assert_eq!(caps(Platform::MacOs), ["⌃", "⇧", "Z"]);
    }

    #[test]
    fn an_unbound_action_has_no_text() {
        let keymap = Keymap::conventional(Platform::Windows);
        let nothing = chordkit::AppAction::new("mail.nothing").map(Action::App);
        let text = nothing
            .ok()
            .and_then(|action| shortcut_text_for(&keymap, &action));
        assert_eq!(text, None);
    }
}
