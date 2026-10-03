//! A shortcut as text: the `cmd+n` the conformance file lists a command's key equivalent as.

use crate::vocab::{Shortcut, ShortcutKey};

impl ShortcutKey {
    /// The key's name in a chord: lower case, the Command key `cmd`.
    fn chord_name(self) -> String {
        match self {
            ShortcutKey::Ctrl => "ctrl".to_owned(),
            ShortcutKey::Shift => "shift".to_owned(),
            ShortcutKey::Alt => "alt".to_owned(),
            ShortcutKey::Super => "cmd".to_owned(),
            ShortcutKey::Char('+') => "plus".to_owned(),
            ShortcutKey::Char(c) => c.to_lowercase().collect(),
            ShortcutKey::Space => "space".to_owned(),
            ShortcutKey::Enter => "enter".to_owned(),
            ShortcutKey::Escape => "escape".to_owned(),
            ShortcutKey::Tab => "tab".to_owned(),
            ShortcutKey::Backspace => "backspace".to_owned(),
            ShortcutKey::Up => "up".to_owned(),
            ShortcutKey::Down => "down".to_owned(),
            ShortcutKey::Left => "left".to_owned(),
            ShortcutKey::Right => "right".to_owned(),
            ShortcutKey::Home => "home".to_owned(),
            ShortcutKey::End => "end".to_owned(),
            ShortcutKey::Delete => "delete".to_owned(),
            ShortcutKey::PageUp => "pageup".to_owned(),
            ShortcutKey::PageDown => "pagedown".to_owned(),
            ShortcutKey::Insert => "insert".to_owned(),
            ShortcutKey::ContextMenu => "menu".to_owned(),
        }
    }
}

impl Shortcut {
    /// The keys as text, modifiers first in the Mac's order, joined by `+`: `cmd+n`,
    /// `shift+cmd+z`. A modifier alone, or no key at all, is still written.
    pub fn chord(&self) -> String {
        self.keys()
            .into_iter()
            .map(ShortcutKey::chord_name)
            .collect::<Vec<_>>()
            .join("+")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standard_action::StandardAction;

    #[test]
    fn a_shortcut_is_written_as_its_chord() {
        let cases: Vec<(&str, Shortcut, &str)> = vec![
            ("new", Shortcut::standard(StandardAction::New), "cmd+n"),
            (
                "redo",
                Shortcut::standard(StandardAction::Redo),
                "shift+cmd+z",
            ),
            (
                "paste and match style",
                Shortcut::standard(StandardAction::PasteAndMatchStyle),
                "alt+shift+cmd+v",
            ),
            (
                "given out of order",
                Shortcut(vec![
                    ShortcutKey::Char('E'),
                    ShortcutKey::Super,
                    ShortcutKey::Ctrl,
                ]),
                "ctrl+cmd+e",
            ),
            ("a bare key", Shortcut(vec![ShortcutKey::Char('e')]), "e"),
            (
                "the plus key",
                Shortcut::standard(StandardAction::Bigger),
                "cmd+plus",
            ),
            ("escape", Shortcut(vec![ShortcutKey::Escape]), "escape"),
            ("none", Shortcut::default(), ""),
        ];
        for (name, shortcut, want) in cases {
            assert_eq!(shortcut.chord(), want, "{name}");
        }
    }
}
