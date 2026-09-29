//! The harness's input vocabulary: quire's keys and pointer buttons as the Blitz events a window
//! would deliver for them.

use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, PointerCoords,
};
use ds::{Point, PointerButton, ShortcutKey};
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
pub(crate) fn keyboard(key: ShortcutKey) -> (DomKey, Code) {
    match key {
        ShortcutKey::Ctrl => (DomKey::Control, Code::ControlLeft),
        ShortcutKey::Shift => (DomKey::Shift, Code::ShiftLeft),
        ShortcutKey::Alt => (DomKey::Alt, Code::AltLeft),
        ShortcutKey::Super => (DomKey::Meta, Code::MetaLeft),
        ShortcutKey::Char(c) => (DomKey::Character(c.to_string()), letter(c)),
        ShortcutKey::Space => (DomKey::Character(" ".into()), Code::Space),
        ShortcutKey::Enter => (DomKey::Enter, Code::Enter),
        ShortcutKey::Escape => (DomKey::Escape, Code::Escape),
        ShortcutKey::Tab => (DomKey::Tab, Code::Tab),
        ShortcutKey::Backspace => (DomKey::Backspace, Code::Backspace),
        ShortcutKey::Up => (DomKey::ArrowUp, Code::ArrowUp),
        ShortcutKey::Down => (DomKey::ArrowDown, Code::ArrowDown),
        ShortcutKey::Left => (DomKey::ArrowLeft, Code::ArrowLeft),
        ShortcutKey::Right => (DomKey::ArrowRight, Code::ArrowRight),
        ShortcutKey::Home => (DomKey::Home, Code::Home),
        ShortcutKey::End => (DomKey::End, Code::End),
        ShortcutKey::Delete => (DomKey::Delete, Code::Delete),
        ShortcutKey::PageUp => (DomKey::PageUp, Code::PageUp),
        ShortcutKey::PageDown => (DomKey::PageDown, Code::PageDown),
        ShortcutKey::Insert => (DomKey::Insert, Code::Insert),
        ShortcutKey::ContextMenu => (DomKey::ContextMenu, Code::ContextMenu),
    }
}

/// The modifier flag a held quire key sets; any other key sets none.
pub(crate) fn modifier(key: ShortcutKey) -> Modifiers {
    match key {
        ShortcutKey::Ctrl => Modifiers::CONTROL,
        ShortcutKey::Shift => Modifiers::SHIFT,
        ShortcutKey::Alt => Modifiers::ALT,
        ShortcutKey::Super => Modifiers::META,
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
    use ds::ShortcutKey;
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
            (ShortcutKey::Ctrl, keyboard_types::Modifiers::CONTROL),
            (ShortcutKey::Alt, keyboard_types::Modifiers::ALT),
            (ShortcutKey::Char('k'), keyboard_types::Modifiers::empty()),
        ];
        for (key, want) in cases {
            assert_eq!(modifier(key), want, "{key:?}");
        }
    }

    #[test]
    fn escape_is_the_named_key() {
        assert_eq!(
            keyboard(ShortcutKey::Escape),
            (keyboard_types::Key::Escape, Code::Escape)
        );
    }
}
