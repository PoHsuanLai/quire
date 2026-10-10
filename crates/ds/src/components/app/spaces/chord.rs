//! The key that switches an app's own Space: the platform's primary modifier and the digits 1
//! to 9.

use chordkit::{Context, Key as ChordKey, Platform};
use dioxus::prelude::{Key, Modifiers};
use ds_core::command::chord_of;
use ds_core::standard_action::SpaceNumber;
use ds_core::vocab::{Shortcut, ShortcutKey};

/// Which chord goes with 1 to 9. The default is the platform's primary modifier (Command on a Mac
/// and our desktop, Ctrl elsewhere, chordkit's `Primary`): an app's own Spaces are the app's. The
/// desktop's own workspace switch is a standard action (`space_switch::space_pressed`), which
/// design/27 section 8 keeps off the primary modifier on our desktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SwitchChord {
    /// Primary+1 to Primary+9 (⌘1 to ⌘9 on a Mac).
    #[default]
    Primary,
    /// No chord: the app reaches its Spaces through its menu and command palette, and the
    /// controller reads no keys (a terminal, whose ⌘1 to ⌘9 are its tabs).
    Unbound,
}

impl SwitchChord {
    /// The Space a key press asks for on `platform`: a digit 1 to 9 with this chord's modifier
    /// held and nothing else.
    pub fn pressed(
        self,
        platform: Platform,
        key: &Key,
        modifiers: Modifiers,
    ) -> Option<SpaceNumber> {
        match self {
            SwitchChord::Unbound => None,
            SwitchChord::Primary => primary_digit(platform, key, modifiers),
        }
    }

    /// The hint beside the dot at `index` (from zero): `⌘1`, drawn per platform by whoever draws
    /// it. None past the ninth.
    pub fn shortcut(self, index: usize) -> Shortcut {
        let digit = u32::try_from(index + 1)
            .ok()
            .and_then(|n| char::from_digit(n, 10))
            .filter(|_| index < 9);
        match (self, digit) {
            (SwitchChord::Primary, Some(digit)) => {
                Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char(digit)])
            }
            _ => Shortcut::default(),
        }
    }
}

fn primary_digit(platform: Platform, key: &Key, modifiers: Modifiers) -> Option<SpaceNumber> {
    let chord = chord_of(platform, key, modifiers)?;
    if chord.modifiers() != platform.primary(Context::Normal) {
        return None;
    }
    let ChordKey::Char(digit) = chord.key() else {
        return None;
    };
    SpaceNumber::new(u8::try_from(digit.to_digit(10)?).ok()?)
}
