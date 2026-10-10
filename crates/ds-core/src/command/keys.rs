//! Reading the command key off a modifier set. Pure, so a compositor or any non-UI consumer reads
//! a chord the way the windows do. Superseded by the keymap: an app resolves a key to an action
//! (`ds::keys`, `command::resolve`) instead of asking which modifier is held.

use super::key_input::modifiers_of;
use chordkit::{Key as ChordKey, KeyInput, Modifier, NamedKey, Platform};
use keyboard_types::Modifiers;

/// Counts Super as Meta, through chordkit's normaliser (which folds the two into one OS key).
fn folded(modifiers: Modifiers) -> Modifiers {
    let probe = KeyInput::new(modifiers_of(modifiers), ChordKey::Named(NamedKey::Space));
    let os_key = probe
        .normalise(Platform::MacOs)
        .modifiers()
        .contains(Modifier::Super);
    let rest = modifiers - (Modifiers::SUPER | Modifiers::META);
    if os_key { rest | Modifiers::META } else { rest }
}

/// `modifiers` with Super counted as Meta: a window reports the Command key as Super, and the
/// catalogue's chords are written with Meta, so each reads its modifiers through this.
#[deprecated(
    note = "ask the keymap instead: `ds::keys::Keys::action_of` or `ds_core::command::resolve` \
            resolve a key to an action for the platform; type-ahead uses `types_text`, a wheel \
            zoom `holds_primary`"
)]
pub fn command_keys(modifiers: Modifiers) -> Modifiers {
    folded(modifiers)
}

/// Whether `modifiers` hold the command key: Ctrl, or Cmd (reported as Meta or Super).
#[deprecated(
    note = "ask the keymap instead: `ds::keys::Keys::action_of` or `ds_core::command::resolve` \
            resolve a key to an action for the platform; a wheel zoom uses `holds_primary`"
)]
pub fn is_command(modifiers: Modifiers) -> bool {
    folded(modifiers).intersects(Modifiers::CONTROL | Modifiers::META)
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::{command_keys, is_command};
    use keyboard_types::Modifiers;

    #[test]
    fn the_command_key_is_ctrl_meta_or_super_and_nothing_else() {
        let cases = [
            (Modifiers::CONTROL, true),
            (Modifiers::META, true),
            (Modifiers::SUPER, true),
            (Modifiers::SUPER | Modifiers::SHIFT, true),
            (Modifiers::ALT, false),
            (Modifiers::SHIFT, false),
            (Modifiers::empty(), false),
        ];
        for (held, command) in cases {
            assert_eq!(is_command(held), command, "{held:?}");
        }
    }

    #[test]
    fn super_reads_as_meta() {
        assert_eq!(
            command_keys(Modifiers::SUPER | Modifiers::SHIFT),
            Modifiers::META | Modifiers::SHIFT
        );
        assert_eq!(command_keys(Modifiers::ALT), Modifiers::ALT);
    }
}
