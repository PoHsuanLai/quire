//! The scroll key a winit key press is, if it is one.

use blitz_kit::scroll::keys::{ScrollKey, scroll_key};
use dioxus_native::winit::keyboard::{Key as WinitKey, NamedKey};
use keyboard_types::{Key, Modifiers};

/// The scroll key `key` with `held` modifiers is: arrows, Page Up and Down, Home and End, Space.
pub(super) fn scroll_key_of(key: &WinitKey, held: Modifiers) -> Option<ScrollKey> {
    let named = match key {
        WinitKey::Named(NamedKey::ArrowUp) => Key::ArrowUp,
        WinitKey::Named(NamedKey::ArrowDown) => Key::ArrowDown,
        WinitKey::Named(NamedKey::ArrowLeft) => Key::ArrowLeft,
        WinitKey::Named(NamedKey::ArrowRight) => Key::ArrowRight,
        WinitKey::Named(NamedKey::PageUp) => Key::PageUp,
        WinitKey::Named(NamedKey::PageDown) => Key::PageDown,
        WinitKey::Named(NamedKey::Home) => Key::Home,
        WinitKey::Named(NamedKey::End) => Key::End,
        WinitKey::Character(text) if text.as_str() == " " => Key::Character(" ".into()),
        _ => return None,
    };
    scroll_key(&named, held)
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitz_kit::scroll::geom::{Dir, ScrollAxis};

    #[test]
    fn the_keys_that_scroll() {
        // name, key, modifiers, the scroll key
        let cases = [
            (
                "down",
                WinitKey::Named(NamedKey::ArrowDown),
                Modifiers::empty(),
                Some(ScrollKey::Line(ScrollAxis::Y, Dir::Pos)),
            ),
            (
                "page down",
                WinitKey::Named(NamedKey::PageDown),
                Modifiers::empty(),
                Some(ScrollKey::Page(Dir::Pos)),
            ),
            (
                "space",
                WinitKey::Character(" ".into()),
                Modifiers::empty(),
                Some(ScrollKey::Page(Dir::Pos)),
            ),
            (
                "shift space",
                WinitKey::Character(" ".into()),
                Modifiers::SHIFT,
                Some(ScrollKey::Page(Dir::Neg)),
            ),
            (
                "end",
                WinitKey::Named(NamedKey::End),
                Modifiers::empty(),
                Some(ScrollKey::Edge(Dir::Pos)),
            ),
            (
                "alt down is a shortcut",
                WinitKey::Named(NamedKey::ArrowDown),
                Modifiers::ALT,
                None,
            ),
            (
                "a letter",
                WinitKey::Character("j".into()),
                Modifiers::empty(),
                None,
            ),
            (
                "enter",
                WinitKey::Named(NamedKey::Enter),
                Modifiers::empty(),
                None,
            ),
        ];
        for (name, key, held, want) in cases {
            assert_eq!(scroll_key_of(&key, held), want, "{name}");
        }
    }
}
