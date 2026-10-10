//! What a key means to a menu panel before it knows the choice under the selection (design/06
//! section 2.4, design/13 section 13.3.2): the arrows move and wrap, Home and End jump to the
//! ends, Enter picks, Right opens a submenu, Left and Escape close one level, letters jump to
//! the next item that starts with them.

use crate::stack::roving::{Edge, Step};
use dioxus::prelude::*;
use ds_core::command::types_text;

/// What a key does in a menu panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum KeyAct {
    /// Up or Down.
    Move(Step),
    /// Home or End.
    Edge(Edge),
    /// Enter or Tab.
    Pick,
    /// Right: open the selected parent's submenu.
    Open,
    /// Left: close this level's open submenu, or go back to the parent menu.
    Back,
    /// Escape: close one level.
    Close,
    /// Letters typed: jump to the choice that starts with them.
    Jump(String),
}

/// A panel's reading of `key`; `None` for a key it leaves alone. Letters only count with no
/// Ctrl, Alt or Super.
pub(crate) fn key_act(key: &Key, modifiers: Modifiers) -> Option<KeyAct> {
    let chord = !types_text(key, modifiers);
    match key {
        Key::ArrowDown => Some(KeyAct::Move(Step::Down)),
        Key::ArrowUp => Some(KeyAct::Move(Step::Up)),
        Key::Home => Some(KeyAct::Edge(Edge::First)),
        Key::End => Some(KeyAct::Edge(Edge::Last)),
        Key::ArrowRight => Some(KeyAct::Open),
        Key::ArrowLeft => Some(KeyAct::Back),
        Key::Enter | Key::Tab => Some(KeyAct::Pick),
        Key::Escape => Some(KeyAct::Close),
        Key::Character(text) if !chord && !text.trim().is_empty() => {
            Some(KeyAct::Jump(text.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyAct, key_act};
    use crate::stack::roving::{Edge, Step};
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn menu_keys_follow_section_2_4_and_13_3() {
        let none = Modifiers::empty();
        let a = Key::Character("a".to_string());
        #[rustfmt::skip]
        let cases: Vec<(Key, Modifiers, Option<KeyAct>)> = vec![
            (Key::ArrowDown, none, Some(KeyAct::Move(Step::Down))),
            (Key::ArrowUp, none, Some(KeyAct::Move(Step::Up))),
            (Key::Home, none, Some(KeyAct::Edge(Edge::First))),
            (Key::End, none, Some(KeyAct::Edge(Edge::Last))),
            (Key::ArrowRight, none, Some(KeyAct::Open)),
            (Key::ArrowLeft, none, Some(KeyAct::Back)),
            (Key::Enter, none, Some(KeyAct::Pick)),
            (Key::Tab, none, Some(KeyAct::Pick)),
            (Key::Escape, none, Some(KeyAct::Close)),
            (a.clone(), none, Some(KeyAct::Jump("a".to_string()))),
            (a.clone(), Modifiers::SHIFT, Some(KeyAct::Jump("a".to_string()))),
            (a, Modifiers::CONTROL, None),
            (Key::Character(" ".to_string()), none, None),
            (Key::Backspace, none, None),
        ];
        for (key, modifiers, want) in cases {
            assert_eq!(key_act(&key, modifiers), want, "{key:?} {modifiers:?}");
        }
    }
}
