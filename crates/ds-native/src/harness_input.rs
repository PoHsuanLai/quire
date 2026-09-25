//! The harness's input vocabulary: quire's keys and pointer buttons as the Blitz events a window
//! would deliver for them.

use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
};
use ds::{Key, Point, PointerButton};
use keyboard_types::{Code, Key as DomKey, Modifiers};

/// The Blitz button, and the held-buttons set while it is down, for a quire pointer button.
pub(crate) fn blitz_button(button: PointerButton) -> (MouseEventButton, MouseEventButtons) {
    match button {
        PointerButton::Primary => (MouseEventButton::Main, MouseEventButtons::Primary),
        PointerButton::Secondary => (MouseEventButton::Secondary, MouseEventButtons::Secondary),
        PointerButton::Middle => (MouseEventButton::Auxiliary, MouseEventButtons::Auxiliary),
    }
}

/// The pointer buttons the harness's mouse holds down: a move between a press and its release
/// carries them, so a drag is a drag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeldButtons(MouseEventButtons);

impl HeldButtons {
    /// Whether `button` is down.
    pub fn contains(self, button: PointerButton) -> bool {
        self.0.contains(blitz_button(button).1)
    }

    /// Whether none is down.
    pub fn is_empty(self) -> bool {
        self.0.is_empty()
    }

    /// These, with `button` down too.
    pub(crate) fn with(self, button: PointerButton) -> Self {
        HeldButtons(self.0 | blitz_button(button).1)
    }

    /// These, with `button` up.
    pub(crate) fn without(self, button: PointerButton) -> Self {
        HeldButtons(self.0 - blitz_button(button).1)
    }

    /// Blitz's set.
    pub(crate) fn blitz(self) -> MouseEventButtons {
        self.0
    }
}

/// A mouse pointer event at `at`, for `button`, with `buttons` held and `mods` down.
pub(crate) fn pointer(
    at: Point,
    button: MouseEventButton,
    buttons: MouseEventButtons,
    mods: Modifiers,
) -> BlitzPointerEvent {
    let (x, y) = (at.x.0, at.y.0);
    BlitzPointerEvent {
        id: BlitzPointerId::Mouse,
        is_primary: true,
        coords: PointerCoords {
            page_x: x,
            page_y: y,
            screen_x: x,
            screen_y: y,
            client_x: x,
            client_y: y,
        },
        button,
        buttons,
        mods,
        details: Default::default(),
        element: Default::default(),
        active_pointers: Default::default(),
    }
}

/// The DOM key and physical code for a quire key.
pub(crate) fn keyboard(key: Key) -> (DomKey, Code) {
    match key {
        Key::Ctrl => (DomKey::Control, Code::ControlLeft),
        Key::Shift => (DomKey::Shift, Code::ShiftLeft),
        Key::Alt => (DomKey::Alt, Code::AltLeft),
        Key::Super => (DomKey::Meta, Code::MetaLeft),
        Key::Char(c) => (DomKey::Character(c.to_string()), letter(c)),
        Key::Space => (DomKey::Character(" ".into()), Code::Space),
        Key::Enter => (DomKey::Enter, Code::Enter),
        Key::Escape => (DomKey::Escape, Code::Escape),
        Key::Tab => (DomKey::Tab, Code::Tab),
        Key::Backspace => (DomKey::Backspace, Code::Backspace),
        Key::Up => (DomKey::ArrowUp, Code::ArrowUp),
        Key::Down => (DomKey::ArrowDown, Code::ArrowDown),
        Key::Left => (DomKey::ArrowLeft, Code::ArrowLeft),
        Key::Right => (DomKey::ArrowRight, Code::ArrowRight),
        Key::Home => (DomKey::Home, Code::Home),
        Key::End => (DomKey::End, Code::End),
        Key::Delete => (DomKey::Delete, Code::Delete),
        Key::PageUp => (DomKey::PageUp, Code::PageUp),
        Key::PageDown => (DomKey::PageDown, Code::PageDown),
        Key::Insert => (DomKey::Insert, Code::Insert),
    }
}

/// The modifier flag a held quire key sets; any other key sets none.
pub(crate) fn modifier(key: Key) -> Modifiers {
    match key {
        Key::Ctrl => Modifiers::CONTROL,
        Key::Shift => Modifiers::SHIFT,
        Key::Alt => Modifiers::ALT,
        Key::Super => Modifiers::META,
        _ => Modifiers::empty(),
    }
}

/// The physical key a US layout types `c` with, where it is a letter or a digit.
fn letter(c: char) -> Code {
    format!("Key{}", c.to_ascii_uppercase())
        .parse()
        .or_else(|_| format!("Digit{c}").parse())
        .unwrap_or(Code::Unidentified)
}

#[cfg(test)]
mod tests {
    use super::{keyboard, letter, modifier};
    use ds::Key;
    use keyboard_types::Code;

    const LETTERS: &[(char, Code)] = &[
        ('a', Code::KeyA),
        ('Z', Code::KeyZ),
        ('7', Code::Digit7),
        ('/', Code::Unidentified),
    ];

    #[test]
    fn letters_map_to_their_physical_key() {
        for &(c, code) in LETTERS {
            assert_eq!(letter(c), code, "{c:?}");
        }
    }

    #[test]
    fn held_keys_are_modifier_flags() {
        let cases = [
            (Key::Ctrl, keyboard_types::Modifiers::CONTROL),
            (Key::Alt, keyboard_types::Modifiers::ALT),
            (Key::Char('k'), keyboard_types::Modifiers::empty()),
        ];
        for (key, want) in cases {
            assert_eq!(modifier(key), want, "{key:?}");
        }
    }

    #[test]
    fn escape_is_the_named_key() {
        assert_eq!(
            keyboard(Key::Escape),
            (keyboard_types::Key::Escape, Code::Escape)
        );
    }
}
