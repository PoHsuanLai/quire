//! Where in the app's own markup a caret sits: an element the app marked, and an offset into its
//! text. The app renders `data-edit-node="{key}"` on every paragraph and object it can address
//! (mailo's `data-n`), so a position is in the app's vocabulary, not the renderer's.

/// The attribute that makes an element addressable; its value is the element's [`EditNode`].
pub const EDIT_NODE_ATTR: &str = "data-edit-node";

/// The attribute that says what an addressable element is ([`EditKind`]); absent means text.
pub const EDIT_KIND_ATTR: &str = "data-edit-kind";

/// An addressable element, named by the value of its `data-edit-node` attribute.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EditNode(pub String);

/// An offset into an addressable element. For text, UTF-8 bytes into the element's own text: the
/// text nodes whose nearest addressable ancestor it is, in document order. For an atom, `0` is
/// before it and `1` after it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct TextOffset(pub usize);

/// A caret position: an addressable element and an offset into it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextPosition {
    /// The element.
    pub node: EditNode,
    /// Where in it.
    pub offset: TextOffset,
}

impl TextPosition {
    /// The position `offset` into the element `node`.
    pub fn new(node: impl Into<String>, offset: usize) -> Self {
        TextPosition {
            node: EditNode(node.into()),
            offset: TextOffset(offset),
        }
    }
}

/// A selection, from where it started to where it ends now; either may come first.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextRange {
    /// Where the selection started.
    pub anchor: TextPosition,
    /// Where it ends now.
    pub focus: TextPosition,
}

/// What an addressable element is, written as its `data-edit-kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EditKind {
    /// Text: offsets are bytes into its text (the attribute may be left off).
    #[default]
    Text,
    /// An object the caret goes around, never into (mailo's signature, image, quoted reply):
    /// offset 0 is before it, 1 after it.
    Atom,
}

impl EditKind {
    /// The `data-edit-kind` value.
    pub fn slug(self) -> &'static str {
        match self {
            EditKind::Text => "text",
            EditKind::Atom => "atom",
        }
    }

    /// The kind a `data-edit-kind` value names; anything but `atom` is text.
    pub fn from_slug(slug: Option<&str>) -> Self {
        match slug {
            Some("atom") => EditKind::Atom,
            _ => EditKind::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EditKind;

    #[test]
    fn a_kind_round_trips_through_its_slug() {
        for kind in [EditKind::Text, EditKind::Atom] {
            assert_eq!(EditKind::from_slug(Some(kind.slug())), kind, "{kind:?}");
        }
        assert_eq!(EditKind::from_slug(None), EditKind::Text);
        assert_eq!(EditKind::from_slug(Some("atomic")), EditKind::Text);
    }
}
