//! The words of a tooltip, in one house style (owner decision 2026-10-09, design/30 section 2.5):
//! a tooltip is "just a noun with keybinds", written `Name  ⌘K`: a short noun phrase in title
//! case, two spaces, then the key as quire draws it. No parentheses, no sentence, no full stop.
//!
//! Every tip quire draws is built here, so the style cannot drift. The tip is separate from the
//! accessible name: a control's `aria-label` keeps its full wording ("Star this thread") while
//! its tip is terse ("Star  S").

use crate::keys::Keys;
use ds_core::vocab::Shortcut;
use std::fmt;

/// What stands between a tip's name and its key: two spaces.
pub const TIP_SEPARATOR: &str = "  ";

/// A tip: a name and, when the control has one, its key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TipText {
    name: String,
    shortcut: Option<Shortcut>,
}

impl TipText {
    /// A tip with no key.
    pub fn new(name: impl Into<String>) -> Self {
        TipText {
            name: name.into(),
            shortcut: None,
        }
    }

    /// The same tip with `shortcut`; one with no keys at all adds nothing.
    pub fn with_shortcut(self, shortcut: Shortcut) -> Self {
        let shortcut = Some(shortcut).filter(|shortcut| !shortcut.0.is_empty());
        TipText { shortcut, ..self }
    }

    /// The same tip with `shortcut` when there is one.
    pub fn with_shortcut_opt(self, shortcut: Option<Shortcut>) -> Self {
        match shortcut {
            Some(shortcut) => self.with_shortcut(shortcut),
            None => self,
        }
    }

    /// The name alone.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The tip as drawn on our desktop: `Name  ⌘K`, or `Name` with no key.
    pub fn render(&self) -> String {
        self.render_with(Shortcut::glyphs)
    }

    /// The tip as `keys` draws its key for the platform: `Name  ⌘K` on a Mac-style one,
    /// `Name  Ctrl+K` elsewhere.
    pub fn render_in(&self, keys: &Keys) -> String {
        self.render_with(|shortcut| keys.text_of(shortcut))
    }

    fn render_with(&self, draw: impl Fn(&Shortcut) -> String) -> String {
        match &self.shortcut {
            Some(shortcut) => format!("{}{TIP_SEPARATOR}{}", self.name, draw(shortcut)),
            None => self.name.clone(),
        }
    }
}

impl fmt::Display for TipText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

impl From<&str> for TipText {
    fn from(name: &str) -> Self {
        TipText::new(name)
    }
}

impl From<String> for TipText {
    fn from(name: String) -> Self {
        TipText::new(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ds_core::vocab::ShortcutKey;

    fn cmd(c: char) -> Shortcut {
        Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char(c)])
    }

    #[test]
    fn a_tip_with_a_key_is_the_name_two_spaces_and_the_glyphs() {
        assert_eq!(
            TipText::new("Archive").with_shortcut(cmd('k')).render(),
            "Archive  ⌘K"
        );
    }

    #[test]
    fn modifiers_come_in_the_macs_order_whatever_order_they_were_given() {
        let keys = Shortcut(vec![
            ShortcutKey::Super,
            ShortcutKey::Shift,
            ShortcutKey::Char('z'),
        ]);
        assert_eq!(
            TipText::new("Redo").with_shortcut(keys).render(),
            "Redo  ⇧⌘Z"
        );
    }

    #[test]
    fn a_tip_with_no_key_is_the_name_alone() {
        assert_eq!(TipText::new("Star").render(), "Star");
        assert_eq!(
            TipText::new("Star").with_shortcut_opt(None).to_string(),
            "Star"
        );
    }

    #[test]
    fn a_shortcut_with_no_keys_adds_no_separator() {
        let tip = TipText::new("Space 10").with_shortcut(Shortcut(Vec::new()));
        assert_eq!(tip.render(), "Space 10");
    }

    #[test]
    fn a_tip_keeps_its_name_apart_from_its_key() {
        let tip = TipText::from("New Space").with_shortcut(cmd('n'));
        assert_eq!(tip.name(), "New Space");
        assert_eq!(tip.render(), "New Space  ⌘N");
    }
}
