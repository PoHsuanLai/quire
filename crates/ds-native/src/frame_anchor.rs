//! What a link in a frame says: its destination, its visible text and its title. mailo's link
//! honesty check compares the text a reader sees with where the link goes, so the text is read
//! from the frame's own document, whitespace-collapsed as it is laid out.

use blitz_dom::{BaseDocument, LocalName, NodeId};

/// One link's destination, text and title.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct LinkFacts {
    /// The target, resolved against the frame's base URL.
    pub(crate) href: String,
    /// The anchor's text content, runs of whitespace collapsed to one space and trimmed.
    pub(crate) text: String,
    /// The anchor's `title`, if it has one.
    pub(crate) title: Option<String>,
}

impl LinkFacts {
    /// A link to `href` whose anchor could not be found again: no text, no title.
    pub(crate) fn bare(href: &str) -> Self {
        LinkFacts {
            href: href.to_owned(),
            ..LinkFacts::default()
        }
    }
}

/// The nearest anchor with an `href` at or above `node` in `doc`, and what it says.
pub(crate) fn anchor_at(doc: &BaseDocument, node: NodeId) -> Option<(NodeId, LinkFacts)> {
    std::iter::successors(Some(node), |id| doc.get_node(*id)?.parent)
        .find_map(|id| Some((id, facts(doc, id)?)))
}

/// The link clicked in `doc` toward `href`: the anchor under the pointer (a click), else the
/// focused one (a key), else the first anchor that goes there; bare if none does.
pub(crate) fn clicked(doc: &BaseDocument, href: &str) -> LinkFacts {
    let pointed = [doc.get_hover_node_id(), doc.get_focussed_node_id()]
        .into_iter()
        .flatten()
        .filter_map(|node| anchor_at(doc, node).map(|(_, facts)| facts));
    let any = anchors(doc).into_iter().filter_map(|node| facts(doc, node));
    pointed
        .chain(any)
        .find(|facts| facts.href == href)
        .unwrap_or_else(|| LinkFacts::bare(href))
}

/// What `node` says, if it is an `a` element with an `href` that resolves.
fn facts(doc: &BaseDocument, node: NodeId) -> Option<LinkFacts> {
    let node = doc.get_node(node)?;
    let element = node.element_data()?;
    if &*element.name.local != "a" {
        return None;
    }
    let href = node.attr(LocalName::from("href"))?;
    Some(LinkFacts {
        href: doc.url().join(href).ok()?.to_string(),
        text: collapse(&node.text_content()),
        title: node.attr(LocalName::from("title")).map(str::to_owned),
    })
}

/// Every anchor with an `href` in `doc`.
fn anchors(doc: &BaseDocument) -> Vec<NodeId> {
    doc.query_selector_all("a[href]")
        .map(|found| found.into_iter().collect())
        .unwrap_or_default()
}

/// `text` with every run of whitespace one space, trimmed: what a reader sees.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::collapse;

    const CASES: &[(&str, &str)] = &[
        ("", ""),
        ("  Offer ", "Offer"),
        ("Your\n   bank\tlogin", "Your bank login"),
        ("\u{a0}paypal.com\u{a0}", "paypal.com"),
    ];

    #[test]
    fn text_is_collapsed_as_a_reader_sees_it() {
        for &(raw, want) in CASES {
            assert_eq!(collapse(raw), want, "{raw:?}");
        }
    }
}
