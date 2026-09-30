//! How a test drives a document and reads it: [`Driver`] sends inputs and lets time pass,
//! [`DocQuery`] gives read access to the document, and [`Query`] is every assertion helper
//! built on it, so a second driver (a shell surface's headless host) gets them all by
//! implementing `DocQuery`.

use crate::error::HarnessError;
use crate::harness_style::{Part, Srgba};
use crate::input::Input;
use blitz_dom::util::ToColorColor;
use blitz_dom::{BaseDocument, LocalName, NodeId};
use ds::{Point, Px, Rect, Size};
use std::time::Duration;

/// What sends input to a document under test and moves its time.
pub trait Driver {
    /// Deliver `input` and bring the document up to date.
    fn send(&mut self, input: Input);
    /// Let `by` pass: fire due timers and render. At least `by` passes.
    fn advance(&mut self, by: Duration);
    /// Paint the document as it is now.
    fn render(&mut self) -> Result<image::RgbaImage, HarnessError>;
}

/// Read access to a document under test.
pub trait DocQuery {
    /// Run `read` on the document as it is now.
    fn with_doc<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> T;
}

/// Whether an element carries a class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClassPresence {
    /// The element carries the class as a whole token.
    Present,
    /// It does not, or no element matched.
    Absent,
}

/// Whether an element has the keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FocusState {
    /// The element has the keyboard focus.
    Focused,
    /// It does not, or no element matched.
    Unfocused,
}

/// The assertion helpers every [`DocQuery`] has. Selectors name the first matching element; an
/// unparseable selector matches nothing.
pub trait Query: DocQuery {
    /// The border-box rect of the first element matching `selector`, if any.
    fn rect(&self, selector: &str) -> Option<Rect> {
        self.with_doc(|doc| rect_of(doc, first(doc, selector)?))
    }

    /// The text inside the first element matching `selector`, if any.
    fn text_of(&self, selector: &str) -> Option<String> {
        self.with_doc(|doc| Some(doc.get_node(first(doc, selector)?)?.text_content()))
    }

    /// Attribute `name` of the first element matching `selector`, if both exist.
    fn attr(&self, selector: &str, name: &str) -> Option<String> {
        self.with_doc(|doc| {
            let node = doc.get_node(first(doc, selector)?)?;
            node.attr(LocalName::from(name)).map(str::to_owned)
        })
    }

    /// How many elements match `selector`.
    fn count(&self, selector: &str) -> usize {
        self.with_doc(|doc| {
            doc.query_selector_all(selector)
                .map_or(0, |found| found.len())
        })
    }

    /// Whether the first element matching `selector` carries `class` as a whole class token.
    fn has_class(&self, selector: &str, class: &str) -> ClassPresence {
        let carried = self
            .attr(selector, "class")
            .is_some_and(|classes| classes.split_ascii_whitespace().any(|token| token == class));
        if carried {
            ClassPresence::Present
        } else {
            ClassPresence::Absent
        }
    }

    /// The computed `color` of the first element matching `selector`.
    fn ink_of(&self, selector: &str) -> Option<Srgba> {
        self.with_doc(|doc| {
            let styles = doc.get_node(first(doc, selector)?)?.primary_styles()?;
            Some(Srgba(styles.clone_color().as_color_color().components))
        })
    }

    /// The computed `background-color` of the first element matching `selector`, or of its
    /// `::before` when `part` says so.
    fn fill_of(&self, selector: &str, part: Part) -> Option<Srgba> {
        self.with_doc(|doc| {
            let node = doc.get_node(part.node(doc, first(doc, selector)?)?)?;
            let styles = node.primary_styles()?;
            let colour = styles
                .get_background()
                .background_color
                .resolve_to_absolute(&styles.clone_color());
            Some(Srgba(colour.as_color_color().components))
        })
    }

    /// Whether the first element matching `selector` has the keyboard focus.
    fn focus_of(&self, selector: &str) -> FocusState {
        let focused = self.with_doc(|doc| {
            first(doc, selector).is_some_and(|node| doc.get_focussed_node_id() == Some(node))
        });
        if focused {
            FocusState::Focused
        } else {
            FocusState::Unfocused
        }
    }
}

impl<T: DocQuery + ?Sized> Query for T {}

/// The first element matching `selector`; an unparseable selector matches nothing.
pub(crate) fn first(doc: &BaseDocument, selector: &str) -> Option<NodeId> {
    doc.query_selector(selector).ok().flatten()
}

/// The border-box rect of `node`.
pub(crate) fn rect_of(doc: &BaseDocument, node: NodeId) -> Option<Rect> {
    let found = doc.get_client_bounding_rect(node)?;
    Some(Rect {
        origin: Point {
            x: Px(found.x as f32),
            y: Px(found.y as f32),
        },
        size: Size {
            width: Px(found.width as f32),
            height: Px(found.height as f32),
        },
    })
}
