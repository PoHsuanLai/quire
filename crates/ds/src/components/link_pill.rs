//! LinkPill: the real destination of a link, instant, and loud when the text lies
//! (design/04-COMPONENTS.md section 29, design/06-INTERACTIONS.md section 13). The honest/lying
//! decision is mailo's. The consumer mounts it in the reader the moment the pointer is over a
//! link and drops it on leave: no intent delay, no exit.

use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
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
    match target {
        // The registered domain never truncates: the path gives way first (O-30).
        LinkTarget::Honest {
            scheme_sub,
            registered,
            path,
        } => rsx! {
            div { class: "ds-link-pill", "data-truth": "honest", role: "status",
                span { class: "ds-link-pill-dim", "{scheme_sub}" }
                b { "{registered}" }
                span { class: "ds-link-pill-dim", "data-part": "path", "{path}" }
            }
        },
        LinkTarget::Lying { registered, shown } => rsx! {
            div { class: "ds-link-pill", "data-truth": "lying", role: "status",
                Glyph { icon: Icon::X, size: IconSize::Small }
                span {
                    "Goes to "
                    b { "{registered}" }
                    ", not {shown}"
                }
            }
        },
    }
}
