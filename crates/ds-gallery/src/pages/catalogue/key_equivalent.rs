//! KeyEquivalent: the plain text and the key caps, in the Mac's key order, at each size.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::controls::key_equivalent::{KeyEquivalent, KeyStyle};
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The KeyEquivalent section.
#[component]
pub fn KeyEquivalentSection() -> Element {
    let shortcuts = [
        Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]),
        Shortcut(vec![
            ShortcutKey::Super,
            ShortcutKey::Shift,
            ShortcutKey::Alt,
            ShortcutKey::Ctrl,
            ShortcutKey::Enter,
        ]),
        // ⌃⌘S: the bundled Inter draws the Control glyph (a system fallback drew a caret).
        Shortcut(vec![
            ShortcutKey::Ctrl,
            ShortcutKey::Super,
            ShortcutKey::Char('s'),
        ]),
        Shortcut(vec![
            ShortcutKey::Escape,
            ShortcutKey::Tab,
            ShortcutKey::Space,
        ]),
        Shortcut(vec![
            ShortcutKey::Up,
            ShortcutKey::Down,
            ShortcutKey::Left,
            ShortcutKey::Right,
        ]),
    ];
    rsx! {
        Section { title: "KeyEquivalent", note: "Symbols in the order Control, Option, Shift, Command, then the key. Text is a menu item's trailing key equivalent; Cap draws each key as a key cap. Static: no hover, no press.",
            for style in KeyStyle::ALL.iter().copied() {
                for size in [ControlSize::Mini, ControlSize::Small, ControlSize::Regular] {
                    div { class: "g-row",
                        span { class: "g-name g-type-name", "{style.slug()} {size.slug()}" }
                        for shortcut in shortcuts.iter().cloned() {
                            KeyEquivalent { shortcut, style, size }
                        }
                    }
                }
            }
        }
    }
}
