//! LinkPill: the real destination of a link, instant, and loud when the text lies
//! (design/04-COMPONENTS.md section 29). The honest/lying decision is mailo's.

use dioxus::prelude::*;

/// Where a link goes, as mail decided it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LinkTarget {
    /// The text and the href agree.
    Honest {
        /// Scheme and subdomain, dimmed: `https://www.`.
        scheme_sub: String,
        /// The registered domain, bold.
        registered: String,
        /// The path, dimmed.
        path: String,
    },
    /// The text names another registered domain.
    Lying {
        /// Where it really goes.
        registered: String,
        /// What the text claimed.
        shown: String,
    },
}

/// The link's destination.
#[component]
pub fn LinkPill(target: LinkTarget) -> Element {
    todo!()
}
