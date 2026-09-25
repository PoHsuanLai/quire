//! The frame documents inside a quire document, found by walking its `iframe` elements (and
//! theirs): where ds-native reads each frame's `data-frame-tag`, and finds a frame's document
//! again by its id to read a clicked link's text.

use crate::frame_tag::{FrameTag, TAG_ATTRIBUTE};
use crate::origin::FrameId;
use blitz_dom::{BaseDocument, LocalName, NodeId};

/// Every frame document inside `doc`, nested ones included, with its `iframe`'s tag.
pub(crate) fn live_frames(doc: &BaseDocument) -> Vec<(FrameId, Option<FrameTag>)> {
    let mut found = Vec::new();
    collect(doc, &mut found);
    found
}

fn collect(doc: &BaseDocument, found: &mut Vec<(FrameId, Option<FrameTag>)>) {
    for iframe in iframes(doc) {
        let Some(node) = doc.get_node(iframe) else {
            continue;
        };
        let Some(sub) = node.subdoc() else {
            continue;
        };
        let tag = FrameTag::read(node.attr(LocalName::from(TAG_ATTRIBUTE)));
        found.push((FrameId::of(sub.id()), tag));
        collect(&sub.inner(), found);
    }
}

/// Read `frame`'s document with `read`, if it is inside `doc`.
pub(crate) fn in_frame<T>(
    doc: &BaseDocument,
    frame: FrameId,
    read: &dyn Fn(&BaseDocument) -> T,
) -> Option<T> {
    iframes(doc).into_iter().find_map(|iframe| {
        let sub = doc.get_node(iframe)?.subdoc()?;
        let inner = sub.inner();
        if FrameId::of(sub.id()) == frame {
            Some(read(&inner))
        } else {
            in_frame(&inner, frame, read)
        }
    })
}

/// Every `iframe` element in `doc`.
fn iframes(doc: &BaseDocument) -> Vec<NodeId> {
    doc.query_selector_all("iframe")
        .map(|found| found.into_iter().collect())
        .unwrap_or_default()
}
