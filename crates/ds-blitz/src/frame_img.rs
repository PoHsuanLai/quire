//! What an image in a frame is: the `src` it was drawn from, resolved against the frame's base
//! URL (a mail body's `cid:` reference stays `cid:`), and its `alt`. An image inside a link is
//! the link's: a click on it navigates (`crate::frame_links`), so it is not an image here.

use crate::frame_anchor::anchor_at;
use blitz_dom::{BaseDocument, LocalName, NodeId};

/// One image's source and alternative text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ImageFacts {
    /// The `src`, resolved against the frame's base URL; as written when it does not resolve.
    pub(crate) src: String,
    /// The `alt`, if it has one.
    pub(crate) alt: Option<String>,
}

/// What `node` in `doc` shows, if it is an `img` with a `src` and no link above it.
pub(crate) fn image_at(doc: &BaseDocument, node: NodeId) -> Option<ImageFacts> {
    let element = doc.get_node(node)?;
    if &*element.element_data()?.name.local != "img" || anchor_at(doc, node).is_some() {
        return None;
    }
    let written = element.attr(LocalName::from("src"))?.trim();
    if written.is_empty() {
        return None;
    }
    let src = doc
        .url()
        .join(written)
        .map_or_else(|_| written.to_owned(), |url| url.to_string());
    Some(ImageFacts {
        src,
        alt: element.attr(LocalName::from("alt")).map(str::to_owned),
    })
}
