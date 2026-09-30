//! Space switching by key (design/30 section 2.11, design/27 section 8 decision 5): Ctrl+1 to
//! Ctrl+9 switch to that Space, and Command+1 to 9 are left to the apps. The Space's colour then
//! cross-fades over `--t-big` (the frame's own transition, not this file's).

use dioxus::prelude::{Key, Modifiers};
use ds_core::standard_action::{SpaceNumber, StandardAction};
use ds_core::vocab::Shortcut;

/// The Space a key press asks for: a digit 1 to 9 with Control held and nothing else.
pub fn space_pressed(key: &Key, modifiers: Modifiers) -> Option<SpaceNumber> {
    if modifiers != Modifiers::CONTROL {
        return None;
    }
    let Key::Character(text) = key else {
        return None;
    };
    let mut chars = text.chars();
    let digit = chars.next().filter(|_| chars.next().is_none())?;
    let n = u8::try_from(digit.to_digit(10)?).ok()?;
    SpaceNumber::new(n)
}

/// The shortcut that switches to `space`, for a hint beside its dot.
pub fn space_shortcut(space: SpaceNumber) -> Shortcut {
    Shortcut::standard(StandardAction::SwitchToSpace(space))
}

#[cfg(test)]
mod tests {
    use super::{space_pressed, space_shortcut};
    use dioxus::prelude::{Key, Modifiers};
    use ds_core::standard_action::SpaceNumber;

    fn key(text: &str) -> Key {
        Key::Character(text.to_string())
    }

    #[test]
    fn only_control_and_a_digit_from_one_to_nine_switches() {
        let cases = [
            (key("1"), Modifiers::CONTROL, SpaceNumber::new(1)),
            (key("9"), Modifiers::CONTROL, SpaceNumber::new(9)),
            (key("0"), Modifiers::CONTROL, None),
            (key("1"), Modifiers::empty(), None),
            (key("1"), Modifiers::META, None),
            (key("1"), Modifiers::CONTROL | Modifiers::SHIFT, None),
            (key("a"), Modifiers::CONTROL, None),
            (key("12"), Modifiers::CONTROL, None),
            (Key::Enter, Modifiers::CONTROL, None),
        ];
        for (pressed, modifiers, want) in cases {
            assert_eq!(space_pressed(&pressed, modifiers), want, "{pressed:?}");
        }
    }

    #[test]
    fn the_hint_is_control_and_the_digit() {
        let three = SpaceNumber::new(3).expect("3 is a space");
        assert_eq!(space_shortcut(three).glyphs(), "⌃3");
    }
}
