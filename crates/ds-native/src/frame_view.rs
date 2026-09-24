//! Reading a frame's sub-document from a test (G4): a `srcdoc` frame is a separate Blitz
//! document, so the harness's own queries never see into it (the isolation mailo's reader relies
//! on) and a test that wants its contents asks the frame.

use crate::harness::{Harness, first};
use crate::origin::FrameId;
use blitz_dom::{BaseDocument, LocalName, NodeId};
use ds::{Point, Px};

/// One `iframe` of a harness's document and the sub-document it shows.
#[derive(Debug, Clone, Copy)]
pub struct FrameView<'h> {
    harness: &'h Harness,
    /// The `iframe` element in the app's document.
    iframe: NodeId,
    /// Its document when the view was taken.
    id: FrameId,
}

impl<'h> FrameView<'h> {
    /// The first `iframe` matching `selector` in `harness`'s document, if it has a document.
    pub(crate) fn find(harness: &'h Harness, selector: &str) -> Option<Self> {
        harness.with_doc(|doc| {
            let iframe = first(doc, selector)?;
            let sub = doc.get_node(iframe)?.subdoc()?;
            Some(FrameView {
                harness,
                iframe,
                id: FrameId::of(sub.id()),
            })
        })
    }

    /// The frame's document: what its requests and link clicks carry as their origin.
    pub fn id(&self) -> FrameId {
        self.id
    }

    /// Every text in the frame's document, scripts' and styles' source included.
    pub fn text(&self) -> String {
        self.read(|doc| doc.root_element().text_content())
            .unwrap_or_default()
    }

    /// The frame's document as HTML.
    pub fn html(&self) -> String {
        self.read(|doc| doc.root_element().outer_html())
            .unwrap_or_default()
    }

    /// How many elements in the frame match `selector`.
    pub fn count(&self, selector: &str) -> usize {
        self.read(|doc| {
            doc.query_selector_all(selector)
                .map_or(0, |found| found.len())
        })
        .unwrap_or_default()
    }

    /// The text inside the frame's first element matching `selector`, if any.
    pub fn text_of(&self, selector: &str) -> Option<String> {
        self.read(|doc| Some(doc.get_node(first(doc, selector)?)?.text_content()))
            .flatten()
    }

    /// Attribute `name` of the frame's first element matching `selector`, if both exist.
    pub fn attr(&self, selector: &str, name: &str) -> Option<String> {
        self.read(|doc| {
            let node = doc.get_node(first(doc, selector)?)?;
            node.attr(LocalName::from(name)).map(str::to_owned)
        })
        .flatten()
    }

    /// The width of the frame's first element matching `selector`, in logical pixels: whether
    /// an image in the frame loaded.
    pub fn width(&self, selector: &str) -> Option<Px> {
        self.read(|doc| {
            let rect = doc.get_client_bounding_rect(first(doc, selector)?)?;
            Some(Px(rect.width as f32))
        })
        .flatten()
    }

    /// The centre of the frame's first element matching `selector`, in the app document's
    /// coordinates: where a test clicks it with [`Harness::click`].
    pub fn centre(&self, selector: &str) -> Option<Point> {
        self.harness.with_doc(|doc| {
            let node = doc.get_node(self.iframe)?;
            let offset = node.absolute_position(0.0, 0.0);
            let sub = node.subdoc()?.inner();
            let rect = sub.get_client_bounding_rect(first(&sub, selector)?)?;
            Some(Point {
                x: Px(offset.x + (rect.x + rect.width / 2.0) as f32),
                y: Px(offset.y + (rect.y + rect.height / 2.0) as f32),
            })
        })
    }

    /// Read the frame's document, if the `iframe` still has one.
    fn read<T>(&self, read: impl FnOnce(&BaseDocument) -> T) -> Option<T> {
        self.harness.with_doc(|doc| {
            let sub = doc.get_node(self.iframe)?.subdoc()?;
            Some(read(&sub.inner()))
        })
    }
}
