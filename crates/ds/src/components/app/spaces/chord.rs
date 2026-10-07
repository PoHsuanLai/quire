//! The key that switches Space: a modifier and the digits 1 to 9.

use dioxus::prelude::{Key, Modifiers};
use ds_core::standard_action::SpaceNumber;
use ds_core::vocab::{Shortcut, ShortcutKey};

/// Which modifier goes with 1 to 9. The default is quire's primary modifier, Command (Super):
/// an app's own Spaces are the app's. Control is the desktop's workspace switch
/// (`space_switch::space_pressed`), left free of apps by design/27 section 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SwitchChord {
    /// ⌘1 to ⌘9.
    #[default]
    Command,
    /// ⌃1 to ⌃9.
    Control,
}

impl SwitchChord {
    fn modifiers(self) -> Modifiers {
        match self {
            SwitchChord::Command => Modifiers::META,
            SwitchChord::Control => Modifiers::CONTROL,
        }
    }

    /// The Space a key press asks for: a digit 1 to 9 with this modifier held and nothing else.
    pub fn pressed(self, key: &Key, modifiers: Modifiers) -> Option<SpaceNumber> {
        if modifiers != self.modifiers() {
            return None;
        }
        let Key::Character(text) = key else {
            return None;
        };
        let mut chars = text.chars();
        let digit = chars.next().filter(|_| chars.next().is_none())?;
        SpaceNumber::new(u8::try_from(digit.to_digit(10)?).ok()?)
    }

    /// The hint beside the dot at `index` (from zero): `⌘1`. None past the ninth.
    pub fn shortcut(self, index: usize) -> Shortcut {
        let digit = u32::try_from(index + 1)
            .ok()
            .and_then(|n| char::from_digit(n, 10))
            .filter(|_| index < 9);
        let modifier = match self {
            SwitchChord::Command => ShortcutKey::Super,
            SwitchChord::Control => ShortcutKey::Ctrl,
        };
        Shortcut(digit.map_or_else(Vec::new, |digit| vec![modifier, ShortcutKey::Char(digit)]))
    }
}
