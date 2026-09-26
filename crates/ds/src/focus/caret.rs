//! Where a field's caret is when a key arrives (sill Q299): the launcher's Right shows the
//! preview pane only with the caret at the end of the query. Blitz keeps the caret in the
//! document, so the host reads it: ds-native provides [`HostCaret`] beside its focus writes, and
//! without one (a webview) the caret is [`Caret::Unknown`].

use dioxus::prelude::MountedData;

/// Where a field's caret is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Caret {
    /// Collapsed after the last character (an empty field's caret is here too).
    AtEnd,
    /// Anywhere else, or a selection of one character or more.
    Inside,
    /// No host could read it: not a Blitz field, the document busy, or no host seam.
    Unknown,
}

/// The host's caret read, provided as root context by ds-native (`launch`, its harness and
/// `ds_native::focus::provide`): where the caret is in the field `element`.
#[derive(Debug, Clone, Copy)]
pub struct HostCaret(pub fn(&MountedData) -> Caret);

/// Where a field's caret goes when the keyboard lands in it (sill Q341): a command palette
/// opened on a query puts it after the last character, as Spotlight does, so Right at once
/// reads [`Caret::AtEnd`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum InitialCaret {
    /// Collapsed after the last character.
    #[default]
    End,
    /// Collapsed before the first character.
    Start,
    /// The whole text selected, so the first key typed replaces it.
    SelectAll,
}

/// The host's caret write, provided as root context by ds-native beside [`HostCaret`]
/// (`launch`, its harness and `ds_native::focus::provide`): put the caret of the field `element`
/// at an [`InitialCaret`] place. Without one (a webview) the caret stays where the renderer put
/// it. `Focused::Busy` asks to be tried again a frame later.
#[derive(Debug, Clone, Copy)]
pub struct HostPlaceCaret(pub fn(&MountedData, InitialCaret) -> crate::focus::host::Focused);

/// Where a caret at byte `focus` of `text`, with the selection `collapsed` or not, sits.
pub fn caret_at(text: &str, focus: usize, collapsed: Collapsed) -> Caret {
    match collapsed {
        Collapsed::Yes if focus >= text.len() => Caret::AtEnd,
        Collapsed::Yes | Collapsed::No => Caret::Inside,
    }
}

/// Whether a field's selection is a bare caret.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Collapsed {
    /// A caret, no characters selected.
    Yes,
    /// One character or more selected.
    No,
}

#[cfg(test)]
mod tests {
    use super::{Caret, Collapsed, caret_at};

    #[test]
    fn only_a_bare_caret_after_the_last_character_is_at_the_end() {
        const CASES: &[(&str, usize, Collapsed, Caret)] = &[
            ("", 0, Collapsed::Yes, Caret::AtEnd),
            ("abc", 3, Collapsed::Yes, Caret::AtEnd),
            ("abc", 1, Collapsed::Yes, Caret::Inside),
            ("abc", 0, Collapsed::Yes, Caret::Inside),
            ("abc", 3, Collapsed::No, Caret::Inside),
            // Bytes, not characters: "é" is two.
            ("é", 2, Collapsed::Yes, Caret::AtEnd),
            ("é", 1, Collapsed::Yes, Caret::Inside),
        ];
        for &(text, focus, collapsed, want) in CASES {
            assert_eq!(caret_at(text, focus, collapsed), want, "{text:?} {focus}");
        }
    }
}
