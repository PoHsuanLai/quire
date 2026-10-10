//! A key press as chordkit reads it: the one place a toolkit's modifiers and key become a
//! [`chordkit::KeyInput`]. Everything that resolves a key to an action goes through it, so no
//! other file decides what Ctrl, Super or Meta mean.

use chordkit::{Key as ChordKey, KeyInput, Modifier, Modifiers as ChordModifiers, NamedKey};
use keyboard_types::{Key, Modifiers};

/// Each toolkit modifier flag and the physical modifier it reports. Super and Meta stay apart
/// here; the platform's normaliser folds them (`chordkit::Platform::meta_as`).
const FLAGS: [(Modifiers, Modifier); 5] = [
    (Modifiers::CONTROL, Modifier::Ctrl),
    (Modifiers::ALT, Modifier::Alt),
    (Modifiers::SHIFT, Modifier::Shift),
    (Modifiers::SUPER, Modifier::Super),
    (Modifiers::META, Modifier::Meta),
];

/// The physical modifiers `held` reports. Lock and AltGraph flags are not chord modifiers.
pub fn modifiers_of(held: Modifiers) -> ChordModifiers {
    FLAGS
        .iter()
        .filter(|(flag, _)| held.contains(*flag))
        .map(|(_, modifier)| *modifier)
        .collect()
}

/// The toolkit flags that stand for the physical modifiers `held`: the reverse of
/// [`modifiers_of`], for a test or a driver that presses a chord.
pub fn modifiers_held(held: ChordModifiers) -> Modifiers {
    FLAGS
        .iter()
        .filter(|(_, modifier)| held.contains(*modifier))
        .fold(Modifiers::empty(), |flags, (flag, _)| flags | *flag)
}

/// The chord key `key` is, or `None` for a key that is no chord key (a modifier on its own, a
/// multi-character string from an IME, a media key).
pub fn chord_key_of(key: &Key) -> Option<ChordKey> {
    let named = |key: NamedKey| Some(ChordKey::Named(key));
    match key {
        Key::Character(text) => character(text),
        Key::Enter => named(NamedKey::Enter),
        Key::Tab => named(NamedKey::Tab),
        Key::Escape => named(NamedKey::Escape),
        Key::Backspace => named(NamedKey::Backspace),
        Key::Delete => named(NamedKey::Delete),
        Key::Insert => named(NamedKey::Insert),
        Key::Home => named(NamedKey::Home),
        Key::End => named(NamedKey::End),
        Key::PageUp => named(NamedKey::PageUp),
        Key::PageDown => named(NamedKey::PageDown),
        Key::ArrowUp => named(NamedKey::Up),
        Key::ArrowDown => named(NamedKey::Down),
        Key::ArrowLeft => named(NamedKey::Left),
        Key::ArrowRight => named(NamedKey::Right),
        Key::PrintScreen => named(NamedKey::PrintScreen),
        other => function_key(other),
    }
}

/// The key press `key` with `modifiers` held, as chordkit takes it. The key is as the toolkit
/// reports it: a shifted symbol (`!` for Shift+1) is not unshifted here, so an app chord that
/// holds Shift with a symbol key needs the host's unshifted key.
pub fn key_input(key: &Key, modifiers: Modifiers) -> Option<KeyInput> {
    Some(KeyInput::new(modifiers_of(modifiers), chord_key_of(key)?))
}

fn character(text: &str) -> Option<ChordKey> {
    let mut chars = text.chars();
    let first = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(match first {
        ' ' => ChordKey::Named(NamedKey::Space),
        other => ChordKey::character(other),
    })
}

/// `F1` to `F24`, read from the key's name (`F5`), so no arm per function key.
fn function_key(key: &Key) -> Option<ChordKey> {
    let name = key.to_string();
    let digits = name.strip_prefix('F')?;
    ChordKey::function(digits.parse().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chordkit::{Chord, Desktop, Platform};

    type Case = (
        &'static str,
        Platform,
        Modifiers,
        &'static str,
        &'static str,
    );

    fn ours() -> Platform {
        Platform::Linux {
            desktop: Desktop::Ours,
        }
    }

    fn kde() -> Platform {
        Platform::Linux {
            desktop: Desktop::Kde,
        }
    }

    #[test]
    fn super_meta_and_ctrl_normalise_per_platform() {
        let cases: &[Case] = &[
            (
                "mac super",
                Platform::MacOs,
                Modifiers::SUPER,
                "c",
                "Super+C",
            ),
            ("mac meta", Platform::MacOs, Modifiers::META, "c", "Super+C"),
            (
                "mac ctrl",
                Platform::MacOs,
                Modifiers::CONTROL,
                "c",
                "Ctrl+C",
            ),
            ("ours super", ours(), Modifiers::SUPER, "C", "Super+C"),
            ("ours meta", ours(), Modifiers::META, "c", "Super+C"),
            ("ours ctrl", ours(), Modifiers::CONTROL, "c", "Ctrl+C"),
            (
                "windows ctrl",
                Platform::Windows,
                Modifiers::CONTROL,
                "c",
                "Ctrl+C",
            ),
            (
                "windows meta",
                Platform::Windows,
                Modifiers::META,
                "c",
                "Super+C",
            ),
            (
                "kde meta shift",
                kde(),
                Modifiers::META | Modifiers::SHIFT,
                "s",
                "Shift+Super+S",
            ),
            (
                "locks are not modifiers",
                kde(),
                Modifiers::CONTROL | Modifiers::CAPS_LOCK | Modifiers::NUM_LOCK,
                "a",
                "Ctrl+A",
            ),
        ];
        for (name, platform, held, key, expected) in cases {
            let input = key_input(&Key::Character((*key).to_owned()), *held);
            let wanted: Chord = expected
                .parse()
                .unwrap_or_else(|e| panic!("{name}: {expected}: {e}"));
            let got = input.map(|input| input.normalise(*platform));
            assert_eq!(got, Some(wanted), "{name}");
        }
    }

    #[test]
    fn flags_and_modifiers_round_trip() {
        let held = Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::SUPER | Modifiers::META;
        assert_eq!(modifiers_held(modifiers_of(held)), held);
        assert_eq!(modifiers_held(ChordModifiers::NONE), Modifiers::empty());
    }

    #[test]
    fn keys_map_to_chord_keys_and_the_rest_to_none() {
        let cases: Vec<(Key, Option<ChordKey>)> = vec![
            (
                Key::Character(" ".into()),
                Some(ChordKey::Named(NamedKey::Space)),
            ),
            (Key::Character("Z".into()), Some(ChordKey::Char('z'))),
            (Key::Character("ab".into()), None),
            (Key::Enter, Some(ChordKey::Named(NamedKey::Enter))),
            (Key::ArrowLeft, Some(ChordKey::Named(NamedKey::Left))),
            (Key::F5, ChordKey::function(5)),
            (Key::F24, ChordKey::function(24)),
            (Key::Control, None),
            (Key::Fn, None),
            (Key::AudioVolumeUp, None),
        ];
        for (key, want) in cases {
            assert_eq!(chord_key_of(&key), want, "{key:?}");
        }
    }
}
