//! How a [`Shortcut`] reads on a platform: through chordkit's chord display, so a Mac-style
//! platform draws `⇧⌘Z`, Windows `Ctrl+Y`, KDE its own order. A shortcut that is a standard
//! action's shows the chord the keymap really binds it to (a person's or the system's change
//! included); any other shortcut shows its own chord with Command as the platform's primary.

use crate::standard_action::DESIGN_PLATFORM;
use crate::vocab::{GlyphKind, Shortcut};
use chordkit::{Action, Chord, Context, Convention, ConventionTable, Keymap, StandardAction};

/// One key cap of a drawn shortcut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCap {
    /// The text on the cap.
    pub text: String,
    /// The face rule the cap needs, if any (arrows draw larger).
    pub kind: Option<GlyphKind>,
}

/// `shortcut` as `keymap`'s platform shows it.
pub fn shortcut_text(keymap: &Keymap, shortcut: &Shortcut) -> String {
    if !is_chord(shortcut) {
        return shortcut.glyphs();
    }
    let platform = keymap.platform();
    let own = shortcut
        .default_chord()
        .and_then(|chord| chord.resolve(platform, Context::Normal));
    let live = owner_of(shortcut)
        .and_then(|action| keymap.chords_of(&Action::Standard(action)).first().copied());
    match live.or(own) {
        // The design glyphs (`ShortcutKey::glyph`) draw what the shortcut itself says; chordkit
        // draws a chord the keymap changed, and every chord on a word-style platform.
        Some(drawn) if platform.is_mac_style() && Some(drawn) == own => shortcut.glyphs(),
        Some(drawn) => drawn.display(platform),
        None => shortcut.glyphs(),
    }
}

/// The caps `shortcut` is drawn on: the modifiers, then the key.
pub fn shortcut_caps(keymap: &Keymap, shortcut: &Shortcut) -> Vec<KeyCap> {
    if !is_chord(shortcut) {
        return shortcut
            .keys()
            .into_iter()
            .map(|key| KeyCap {
                text: key.glyph(),
                kind: key.glyph_kind(),
            })
            .collect();
    }
    split_caps(&shortcut_text(keymap, shortcut))
        .into_iter()
        .map(|text| KeyCap {
            kind: arrow(&text),
            text,
        })
        .collect()
}

/// Whether `shortcut` is a chord: modifiers and exactly one key. A lone modifier, a key sequence
/// or no key at all has no chord to ask the platform about, so it is drawn as written.
fn is_chord(shortcut: &Shortcut) -> bool {
    let keys = shortcut.keys();
    let plain = keys.iter().filter(|key| key.is_plain()).count();
    plain == 1
}

/// The standard action whose design chord (the Mac's, on our desktop) this shortcut is.
fn owner_of(shortcut: &Shortcut) -> Option<StandardAction> {
    let wanted: Chord = shortcut
        .default_chord()?
        .resolve(DESIGN_PLATFORM, Context::Normal)?;
    ConventionTable::for_platform(DESIGN_PLATFORM)
        .iter()
        .find_map(|(action, convention)| match convention {
            Convention::Bound(chord) => {
                (chord.resolve(DESIGN_PLATFORM, Context::Normal) == Some(wanted)).then_some(action)
            }
            Convention::Unbound(_) => None,
        })
}

const MODIFIER_GLYPHS: [char; 4] = ['⌃', '⌥', '⇧', '⌘'];

/// A drawn chord's caps: one per leading modifier glyph then the rest, or one per `+` part.
pub(super) fn split_caps(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if text.starts_with(MODIFIER_GLYPHS) {
        let lead = text.chars().take_while(|c| MODIFIER_GLYPHS.contains(c));
        let rest: String = text
            .chars()
            .skip_while(|c| MODIFIER_GLYPHS.contains(c))
            .collect();
        return lead
            .map(String::from)
            .chain((!rest.is_empty()).then_some(rest))
            .collect();
    }
    let (head, key) = match text.strip_suffix("++") {
        Some(head) => (head, "+"),
        None => match text.rsplit_once('+') {
            Some((head, key)) if !key.is_empty() => (head, key),
            _ => ("", text),
        },
    };
    head.split('+')
        .filter(|part| !part.is_empty())
        .chain([key])
        .map(str::to_owned)
        .collect()
}

pub(super) fn arrow(cap: &str) -> Option<GlyphKind> {
    matches!(cap, "↑" | "↓" | "←" | "→").then_some(GlyphKind::Arrow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vocab::ShortcutKey;
    use chordkit::{Desktop, Platform};

    fn keymap(platform: Platform) -> Keymap {
        Keymap::conventional(platform)
    }

    fn ours() -> Platform {
        Platform::Linux {
            desktop: Desktop::Ours,
        }
    }

    fn redo() -> Shortcut {
        Shortcut::standard(StandardAction::Redo)
    }

    fn command_k() -> Shortcut {
        Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')])
    }

    #[test]
    fn a_shortcut_reads_in_the_platforms_own_way() {
        let cases = [
            ("ours redo", ours(), redo(), "⇧⌘Z"),
            ("mac custom", Platform::MacOs, command_k(), "⌘K"),
            (
                "windows redo is the platforms",
                Platform::Windows,
                redo(),
                "Ctrl+Y",
            ),
            ("windows custom", Platform::Windows, command_k(), "Ctrl+K"),
            (
                "kde copy",
                Platform::Linux {
                    desktop: Desktop::Kde,
                },
                Shortcut::standard(StandardAction::Copy),
                "Ctrl+C",
            ),
            ("empty", ours(), Shortcut::default(), ""),
        ];
        for (name, platform, shortcut, want) in cases {
            assert_eq!(shortcut_text(&keymap(platform), &shortcut), want, "{name}");
        }
    }

    #[test]
    fn caps_split_glyphs_and_words() {
        let cases = [
            ("⇧⌘Z", vec!["⇧", "⌘", "Z"]),
            ("⌃↑", vec!["⌃", "↑"]),
            ("⌘Space", vec!["⌘", "Space"]),
            ("Esc", vec!["Esc"]),
            ("Ctrl+Shift+Z", vec!["Ctrl", "Shift", "Z"]),
            ("Ctrl++", vec!["Ctrl", "+"]),
            ("+", vec!["+"]),
            ("", vec![]),
        ];
        for (text, want) in cases {
            assert_eq!(split_caps(text), want, "{text:?}");
        }
    }

    #[test]
    fn what_is_no_chord_is_drawn_as_written_on_every_platform() {
        let sequence = Shortcut(vec![
            ShortcutKey::Super,
            ShortcutKey::Backspace,
            ShortcutKey::Enter,
        ]);
        let lone = Shortcut(vec![ShortcutKey::Super]);
        for platform in [ours(), Platform::Windows] {
            let keymap = keymap(platform);
            assert_eq!(shortcut_text(&keymap, &sequence), "⌘⌫↵");
            assert_eq!(shortcut_text(&keymap, &lone), "⌘");
            let caps: Vec<String> = shortcut_caps(&keymap, &sequence)
                .into_iter()
                .map(|cap| cap.text)
                .collect();
            assert_eq!(caps, ["⌘", "⌫", "↵"]);
        }
    }

    #[test]
    fn arrows_ask_for_the_larger_face() {
        let up = Shortcut::standard(StandardAction::MissionControl);
        let caps = shortcut_caps(&keymap(ours()), &up);
        assert_eq!(caps.last().and_then(|cap| cap.kind), Some(GlyphKind::Arrow));
        assert_eq!(caps.first().and_then(|cap| cap.kind), None);
    }
}
