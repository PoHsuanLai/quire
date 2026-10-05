//! A key equivalent as `com.canonical.dbusmenu` writes it: the key names of one chord, modifiers
//! first (`["Super", "Z"]`; the Mac's Command key is `Super`, as Qt's exporter names Meta).

use ds_core::vocab::{Shortcut, ShortcutKey};
use serde::{Deserialize, Serialize};

/// The key names of one chord: modifiers in the Mac's order, then the key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Chord(pub Vec<String>);

/// A key name that is no key of ours.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a key name")]
pub struct ChordError(pub String);

/// Every named (non-character) key and its dbusmenu name; a character is its own name.
const NAMED: &[(ShortcutKey, &str)] = &[
    (ShortcutKey::Ctrl, "Control"),
    (ShortcutKey::Alt, "Alt"),
    (ShortcutKey::Shift, "Shift"),
    (ShortcutKey::Super, "Super"),
    (ShortcutKey::Space, "Space"),
    (ShortcutKey::Enter, "Return"),
    (ShortcutKey::Escape, "Escape"),
    (ShortcutKey::Tab, "Tab"),
    (ShortcutKey::Backspace, "BackSpace"),
    (ShortcutKey::Up, "Up"),
    (ShortcutKey::Down, "Down"),
    (ShortcutKey::Left, "Left"),
    (ShortcutKey::Right, "Right"),
    (ShortcutKey::Home, "Home"),
    (ShortcutKey::End, "End"),
    (ShortcutKey::Delete, "Delete"),
    (ShortcutKey::PageUp, "Page_Up"),
    (ShortcutKey::PageDown, "Page_Down"),
    (ShortcutKey::Insert, "Insert"),
    (ShortcutKey::ContextMenu, "Menu"),
];

fn name_of(key: ShortcutKey) -> String {
    match key {
        ShortcutKey::Char(c) => c.to_uppercase().collect(),
        other => NAMED
            .iter()
            .find(|(named, _)| *named == other)
            .map(|(_, name)| (*name).to_owned())
            .unwrap_or_default(),
    }
}

fn key_of(name: &str) -> Result<ShortcutKey, ChordError> {
    if let Some((key, _)) = NAMED.iter().find(|(_, own)| *own == name) {
        return Ok(*key);
    }
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(ShortcutKey::Char(c.to_ascii_lowercase())),
        _ => Err(ChordError(name.to_owned())),
    }
}

impl Chord {
    /// The chord of `shortcut`, its keys in the order they are drawn.
    pub fn of(shortcut: &Shortcut) -> Chord {
        Chord(shortcut.keys().into_iter().map(name_of).collect())
    }

    /// The shortcut the names write.
    pub fn shortcut(&self) -> Result<Shortcut, ChordError> {
        self.0
            .iter()
            .map(|name| key_of(name))
            .collect::<Result<Vec<_>, _>>()
            .map(|keys| Shortcut(Shortcut(keys).keys()))
    }
}
