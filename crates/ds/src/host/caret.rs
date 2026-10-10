//! Where a field's caret is when a key arrives: the launcher's Right shows the
//! preview pane only with the caret at the end of the query. Blitz keeps the caret in the
//! document, so the host reads it; a host that cannot answers [`Caret::Unknown`].

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

/// Where a field's caret goes when the keyboard lands in it: a command palette
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

/// Whether a field focused before its first layout still owes the caret its landing asked for.
/// An empty field owes nothing: its caret sits at the start, which is its end too, so nothing is
/// placed after the editor is built and a key typed the moment it is (a summoned search field
/// typed into at once) is never moved away from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaretOwed {
    /// Nothing to place: the field is laid out, empty, or not a field.
    No,
    /// A field with text and no editor yet: its caret lands once the editor is built.
    AfterLayout,
}

/// A field's caret and selection as the host reads them: a masked field draws its
/// own caret over its dots, since Blitz measures the hidden text in another face than the dots.
/// Offsets count characters (`char`s) of the field's text, as the mask draws one dot for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldSelection {
    /// The field has the keyboard; its selection runs from `anchor` to `focus` (where the caret
    /// is), equal when it is a bare caret.
    Focused {
        /// Where the selection began, in characters.
        anchor: usize,
        /// Where the caret is, in characters.
        focus: usize,
    },
    /// The field does not have the keyboard.
    Unfocused,
    /// No host could read it now: the document busy, or the element not a laid-out field.
    Unknown,
}

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
