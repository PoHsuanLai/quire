//! What the check measures: the visible boxes of a document and the content inside them, read
//! once from the Blitz document ([`crate::inset::read`]) so the measuring itself
//! ([`crate::inset::measure`]) is plain geometry a test can feed by hand.

use crate::inset::bounds::Bounds;

/// Why a box has an edge the eye sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Paint {
    /// A background colour that differs from what is behind it.
    Fill,
    /// A background gradient or image.
    Picture,
    /// A border on at least one side.
    Border,
    /// An outline.
    Outline,
    /// A box shadow.
    Shadow,
    /// A highlighted, selected or hovered state (`aria-selected`, `data-selected`, `:hover`)
    /// whose fill is the box's own, even where it matches what is behind it.
    Marked,
}

/// Which of a box's vertical edges are painted: a box with only a bottom hairline has neither.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Edges {
    /// The left edge.
    pub left: bool,
    /// The right edge.
    pub right: bool,
}

/// An element that paints an edge.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleBox {
    /// Tag, classes and id, from the document root down: `div.ds-list > li.ds-row`.
    pub path: String,
    /// Its classes, its short `data-*` attributes as `[data-size=mini]`, and its parent's, as
    /// `^[data-size=mini]`.
    pub tokens: Vec<String>,
    /// Its border box.
    pub bounds: Bounds,
    /// Which vertical edges are painted.
    pub edges: Edges,
    /// What paints them.
    pub paints: Vec<Paint>,
}

/// What kind of content a rectangle is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContentKind {
    /// A laid-out run of text.
    Text,
    /// A glyph, icon or image: an `svg`, `img`, a masked or image-filled element.
    Glyph,
}

/// Something inside a box that has to keep its distance from the box's edge.
#[derive(Debug, Clone, PartialEq)]
pub struct Content {
    /// Where it is.
    pub bounds: Bounds,
    /// Text or a glyph.
    pub kind: ContentKind,
    /// The path from the box down to the element holding it, and the text for a run.
    pub label: String,
    /// The box it sits in: the nearest box around it, by index into [`Scene::boxes`].
    pub owner: usize,
}

/// The boxes of a document and their content.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Scene {
    /// Every visible box, in document order.
    pub boxes: Vec<VisibleBox>,
    /// Every content rectangle, each owned by its nearest box.
    pub contents: Vec<Content>,
}
