//! Resolving a key press against a keymap, pure: what the windows' `ds::keys` does on a dioxus
//! event, so a compositor or a test reads a key the way a window does.

use super::key_input::{chord_key_of, key_input, modifiers_of};
use chordkit::{Action, Chord, Context, Key as ChordKey, KeyInput, Keymap, NamedKey, Platform};
use keyboard_types::{Key, Modifiers};

/// The action `key` with `modifiers` held triggers in `context`, if any.
pub fn resolve(
    keymap: &Keymap,
    key: &Key,
    modifiers: Modifiers,
    context: Context,
) -> Option<Action> {
    keymap.resolve(&key_input(key, modifiers)?, context)
}

/// The canonical chord `key` with `modifiers` held is on `platform`.
pub fn chord_of(platform: Platform, key: &Key, modifiers: Modifiers) -> Option<Chord> {
    Some(key_input(key, modifiers)?.normalise(platform))
}

/// Whether the key press is typing: a character or editing key with no Ctrl, Alt or Super
/// (Shift alone still types). The test a type-ahead list or a text field uses before it takes
/// the key. A character string longer than one character (an IME's commit) types like any
/// character.
pub fn types_text(key: &Key, modifiers: Modifiers) -> bool {
    let stand_in = || matches!(key, Key::Character(_)).then_some(ChordKey::Char('a'));
    chord_key_of(key)
        .or_else(stand_in)
        .is_some_and(|chord_key| {
            Chord::new(modifiers_of(modifiers), chord_key).belongs_to_text_field()
        })
}

/// Whether `held` holds `platform`'s primary modifier (Command on a Mac or our desktop, Ctrl
/// elsewhere): the test for a wheel zoom, which has no key to resolve.
pub fn holds_primary(platform: Platform, held: Modifiers) -> bool {
    let probe = KeyInput::new(modifiers_of(held), ChordKey::Named(NamedKey::Space));
    let folded = probe.normalise(platform).modifiers();
    platform
        .primary(Context::Normal)
        .iter()
        .all(|modifier| folded.contains(modifier))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordkit::{Desktop, SpaceNumber, StandardAction};

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

    fn standard(action: StandardAction) -> Option<Action> {
        Some(Action::Standard(action))
    }

    fn space(n: u8) -> Option<Action> {
        SpaceNumber::new(n).map(|n| Action::Standard(StandardAction::SwitchToSpace(n)))
    }

    type Case = (
        &'static str,
        Platform,
        Context,
        Modifiers,
        &'static str,
        Option<Action>,
    );

    #[test]
    fn copy_paste_undo_and_spaces_follow_the_platform() {
        let shift = Modifiers::SHIFT;
        let cases: Vec<Case> = vec![
            (
                "ours cmd c",
                ours(),
                Context::Normal,
                Modifiers::SUPER,
                "c",
                standard(StandardAction::Copy),
            ),
            (
                "ours meta v",
                ours(),
                Context::TextEntry,
                Modifiers::META,
                "v",
                standard(StandardAction::Paste),
            ),
            (
                "ours ctrl c is not copy",
                ours(),
                Context::Normal,
                Modifiers::CONTROL,
                "c",
                None,
            ),
            (
                "mac cmd z",
                Platform::MacOs,
                Context::Normal,
                Modifiers::SUPER,
                "z",
                standard(StandardAction::Undo),
            ),
            (
                "mac cmd shift z",
                Platform::MacOs,
                Context::Normal,
                Modifiers::SUPER | shift,
                "z",
                standard(StandardAction::Redo),
            ),
            (
                "windows ctrl c",
                Platform::Windows,
                Context::Normal,
                Modifiers::CONTROL,
                "c",
                standard(StandardAction::Copy),
            ),
            (
                "windows ctrl y",
                Platform::Windows,
                Context::Normal,
                Modifiers::CONTROL,
                "y",
                standard(StandardAction::Redo),
            ),
            (
                "windows super c is not copy",
                Platform::Windows,
                Context::Normal,
                Modifiers::SUPER,
                "c",
                None,
            ),
            (
                "kde ctrl v",
                kde(),
                Context::TextEntry,
                Modifiers::CONTROL,
                "v",
                standard(StandardAction::Paste),
            ),
            (
                "ours ctrl 3 is space 3",
                ours(),
                Context::Normal,
                Modifiers::CONTROL,
                "3",
                space(3),
            ),
            (
                "ours cmd 3 is the app's",
                ours(),
                Context::Normal,
                Modifiers::SUPER,
                "3",
                None,
            ),
            (
                "windows ctrl 3 has no space",
                Platform::Windows,
                Context::Normal,
                Modifiers::CONTROL,
                "3",
                None,
            ),
            (
                "windows terminal keeps ctrl c",
                Platform::Windows,
                Context::Terminal,
                Modifiers::CONTROL,
                "c",
                None,
            ),
            (
                "windows terminal ctrl shift c",
                Platform::Windows,
                Context::Terminal,
                Modifiers::CONTROL | shift,
                "c",
                standard(StandardAction::Copy),
            ),
            (
                "ours terminal cmd c",
                ours(),
                Context::Terminal,
                Modifiers::SUPER,
                "c",
                standard(StandardAction::Copy),
            ),
            (
                "typing is no action",
                kde(),
                Context::TextEntry,
                Modifiers::empty(),
                "c",
                None,
            ),
        ];
        for (name, platform, context, held, key, want) in cases {
            let keymap = Keymap::conventional(platform);
            let pressed = Key::Character(key.to_owned());
            assert_eq!(resolve(&keymap, &pressed, held, context), want, "{name}");
        }
    }

    #[test]
    fn typing_is_a_character_or_editing_key_without_ctrl_alt_or_super() {
        let cases = [
            (Key::Character("a".into()), Modifiers::empty(), true),
            (Key::Character("A".into()), Modifiers::SHIFT, true),
            (Key::Character("a".into()), Modifiers::CONTROL, false),
            (Key::Character("a".into()), Modifiers::ALT, false),
            (Key::Character("a".into()), Modifiers::META, false),
            (Key::Character("a".into()), Modifiers::SUPER, false),
            (Key::Character("你好".into()), Modifiers::empty(), true),
            (Key::Character("你好".into()), Modifiers::CONTROL, false),
            (Key::Backspace, Modifiers::empty(), true),
            (Key::Escape, Modifiers::empty(), false),
        ];
        for (key, held, want) in cases {
            assert_eq!(types_text(&key, held), want, "{key:?} {held:?}");
        }
    }

    #[test]
    fn the_primary_modifier_is_command_on_a_mac_and_ctrl_elsewhere() {
        let cases = [
            (ours(), Modifiers::SUPER, true),
            (ours(), Modifiers::META, true),
            (ours(), Modifiers::CONTROL, false),
            (Platform::MacOs, Modifiers::SUPER | Modifiers::SHIFT, true),
            (Platform::Windows, Modifiers::CONTROL, true),
            (Platform::Windows, Modifiers::SUPER, false),
            (kde(), Modifiers::CONTROL | Modifiers::ALT, true),
            (kde(), Modifiers::empty(), false),
        ];
        for (platform, held, want) in cases {
            assert_eq!(holds_primary(platform, held), want, "{platform:?} {held:?}");
        }
    }
}
