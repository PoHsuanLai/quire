//! Reading the command key off a modifier set. Pure, so a compositor or any non-UI consumer reads
//! a chord the way the windows do.

use keyboard_types::Modifiers;

/// `modifiers` with Super counted as Meta: a window reports the Command key as Super, and the
/// catalogue's chords are written with Meta, so each reads its modifiers through this. Apps that
/// read modifiers off a dioxus key or wheel event go through this (or [`is_command`]) too, so a
/// chord matches in a window as it does in the harness.
pub fn command_keys(modifiers: Modifiers) -> Modifiers {
    if modifiers.contains(Modifiers::SUPER) {
        (modifiers - Modifiers::SUPER) | Modifiers::META
    } else {
        modifiers
    }
}

/// Whether `modifiers` hold the command key: Ctrl, or Cmd (reported as Meta or Super). The test
/// for an app's own chord or Cmd+wheel zoom read from a dioxus event.
pub fn is_command(modifiers: Modifiers) -> bool {
    command_keys(modifiers).intersects(Modifiers::CONTROL | Modifiers::META)
}

#[cfg(test)]
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
