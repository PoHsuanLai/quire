//! Space switching by key (design/30 section 2.11, design/27 section 8 decision 5): the
//! platform's Switch-to-Space-n action (Ctrl+1 to Ctrl+9 on our desktop, where Command+1 to 9 are
//! left to the apps), as the keymap binds it. The Space's colour then cross-fades over `--t-big`
//! (the frame's own transition, not this file's).

use chordkit::{Action, Context, Keymap, StandardAction};
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::resolve;
use ds_core::standard_action::SpaceNumber;
use ds_core::vocab::Shortcut;

/// The Space a key press asks for under `keymap`: the one its Switch-to-Space action names, or
/// `None` for any other key (and for a platform with no key for a numbered desktop).
pub fn space_pressed(keymap: &Keymap, key: &Key, modifiers: Modifiers) -> Option<SpaceNumber> {
    match resolve(keymap, key, modifiers, Context::Normal)? {
        Action::Standard(StandardAction::SwitchToSpace(space)) => Some(space),
        _ => None,
    }
}

/// The shortcut that switches to `space`, for a hint beside its dot.
pub fn space_shortcut(space: SpaceNumber) -> Shortcut {
    Shortcut::standard(StandardAction::SwitchToSpace(space))
}

#[cfg(test)]
mod tests {
    use super::{space_pressed, space_shortcut};
    use chordkit::{Desktop, Keymap, Platform};
    use dioxus::prelude::{Key, Modifiers};
    use ds_core::standard_action::SpaceNumber;

    fn key(text: &str) -> Key {
        Key::Character(text.to_string())
    }

    fn ours() -> Platform {
        Platform::Linux {
            desktop: Desktop::Ours,
        }
    }

    type Case = (Platform, Key, Modifiers, Option<SpaceNumber>);

    #[test]
    fn only_the_platforms_space_chord_switches() {
        let kde = Platform::Linux {
            desktop: Desktop::Kde,
        };
        let cases: Vec<Case> = vec![
            (ours(), key("1"), Modifiers::CONTROL, SpaceNumber::new(1)),
            (ours(), key("9"), Modifiers::CONTROL, SpaceNumber::new(9)),
            (ours(), key("0"), Modifiers::CONTROL, None),
            (ours(), key("1"), Modifiers::empty(), None),
            (ours(), key("1"), Modifiers::META, None),
            (ours(), key("1"), Modifiers::SUPER, None),
            (
                ours(),
                key("1"),
                Modifiers::CONTROL | Modifiers::SHIFT,
                None,
            ),
            (ours(), key("a"), Modifiers::CONTROL, None),
            (ours(), key("12"), Modifiers::CONTROL, None),
            (ours(), Key::Enter, Modifiers::CONTROL, None),
            (kde, Key::F3, Modifiers::CONTROL, SpaceNumber::new(3)),
            (kde, key("3"), Modifiers::CONTROL, None),
            (Platform::Windows, key("3"), Modifiers::CONTROL, None),
        ];
        for (platform, pressed, modifiers, want) in cases {
            let keymap = Keymap::conventional(platform);
            assert_eq!(
                space_pressed(&keymap, &pressed, modifiers),
                want,
                "{platform:?} {pressed:?} {modifiers:?}"
            );
        }
    }

    #[test]
    fn the_hint_is_control_and_the_digit() {
        let three = SpaceNumber::new(3).expect("3 is a space");
        assert_eq!(space_shortcut(three).glyphs(), "⌃3");
    }
}
