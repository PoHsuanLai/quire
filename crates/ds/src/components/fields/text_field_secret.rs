//! What a secure field refuses: the keys that would copy or cut its text out. Paste stays.

use dioxus::prelude::{Key, Modifiers};

/// Whether `key` with `modifiers` copies or cuts text out of a field: the action key with C or X
/// (Control, or Super where it is the action key), Control+Insert (copy) and Shift+Delete (cut).
pub(crate) fn takes_text_out(key: &Key, modifiers: Modifiers) -> bool {
    let action = modifiers.intersects(Modifiers::CONTROL | Modifiers::META);
    match key {
        Key::Character(text) => action && matches!(text.to_lowercase().as_str(), "c" | "x"),
        Key::Insert => modifiers.contains(Modifiers::CONTROL),
        Key::Delete => modifiers.contains(Modifiers::SHIFT),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::takes_text_out;
    use dioxus::prelude::{Key, Modifiers};

    #[test]
    fn copy_and_cut_chords_are_refused_and_paste_and_typing_are_not() {
        let none = Modifiers::empty();
        let ctrl = Modifiers::CONTROL;
        let cases: Vec<(&str, Key, Modifiers, bool)> = vec![
            ("ctrl c", Key::Character("c".into()), ctrl, true),
            ("ctrl X", Key::Character("X".into()), ctrl, true),
            ("super c", Key::Character("c".into()), Modifiers::META, true),
            ("ctrl insert", Key::Insert, ctrl, true),
            ("shift delete", Key::Delete, Modifiers::SHIFT, true),
            ("ctrl v", Key::Character("v".into()), ctrl, false),
            ("shift insert", Key::Insert, Modifiers::SHIFT, false),
            ("plain c", Key::Character("c".into()), none, false),
            ("plain delete", Key::Delete, none, false),
            ("ctrl a", Key::Character("a".into()), ctrl, false),
        ];
        for (name, key, modifiers, want) in cases {
            assert_eq!(takes_text_out(&key, modifiers), want, "{name}");
        }
    }
}
