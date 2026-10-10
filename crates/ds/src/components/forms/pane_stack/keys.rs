//! What a key means to a pane stack, pure: the keys that go back one page.

use crate::keys::pane_back_action;
use chordkit::{Action, Context, Keymap};
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::resolve;

/// Whether `key` with `modifiers` asks to go back under `keymap`: Escape, Alt+Left, and the
/// chord quire's `quire.pane-back` action holds (Command and `[` by default).
pub(crate) fn goes_back(keymap: &Keymap, key: &Key, modifiers: Modifiers) -> bool {
    match key {
        Key::Escape => modifiers.is_empty(),
        Key::ArrowLeft => modifiers == Modifiers::ALT,
        _ => {
            let back = pane_back_action().map(Action::App);
            back.is_some() && resolve(keymap, key, modifiers, Context::Normal) == back
        }
    }
}

#[cfg(test)]
mod tests {
    use super::goes_back;
    use chordkit::{AppAction, AppId, Desktop, Keymap, Platform};
    use dioxus::prelude::{Key, Modifiers};

    fn registered(platform: Platform) -> Keymap {
        let mut keymap = Keymap::conventional(platform);
        let app = AppId::new("quire").expect("an app name");
        let back = AppAction::new("quire.pane-back").expect("an action id");
        let chord = "Primary+[".parse().expect("a chord");
        keymap.register(&app, &[(back, chord)]).expect("free");
        keymap
    }

    #[test]
    fn only_the_back_chords_go_back() {
        let ours = Platform::Linux {
            desktop: Desktop::Ours,
        };
        let bracket = || Key::Character("[".into());
        let cases = [
            ("escape", ours, Key::Escape, Modifiers::empty(), true),
            ("command bracket", ours, bracket(), Modifiers::SUPER, true),
            ("meta bracket", ours, bracket(), Modifiers::META, true),
            (
                "ctrl bracket on a Mac-style desktop",
                ours,
                bracket(),
                Modifiers::CONTROL,
                false,
            ),
            (
                "ctrl bracket on windows",
                Platform::Windows,
                bracket(),
                Modifiers::CONTROL,
                true,
            ),
            ("alt left", ours, Key::ArrowLeft, Modifiers::ALT, true),
            (
                "bare bracket types a bracket",
                ours,
                bracket(),
                Modifiers::empty(),
                false,
            ),
            (
                "bare left moves a caret",
                ours,
                Key::ArrowLeft,
                Modifiers::empty(),
                false,
            ),
            ("shift escape", ours, Key::Escape, Modifiers::SHIFT, false),
            (
                "command right bracket",
                ours,
                Key::Character("]".into()),
                Modifiers::SUPER,
                false,
            ),
        ];
        for (name, platform, key, modifiers, want) in cases {
            let keymap = registered(platform);
            assert_eq!(goes_back(&keymap, &key, modifiers), want, "{name}");
        }
    }
}
