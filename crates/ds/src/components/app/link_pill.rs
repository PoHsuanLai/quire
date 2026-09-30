//! LinkPill: the link under the pointer, as a rounded pill (design/30 section 2.11). It shows the
//! real destination's registered domain; when the pointer rests on it (`HoverIntent`, the `Label`
//! profile) it expands to the whole address, and a click hands the address to the caller to copy
//! and says "Copied" until the pointer leaves. The honest or lying decision is mailo's: a lying
//! link turns the pill loud. The consumer mounts it in the reader the moment the pointer is over
//! a link; it fades in over `--t-quick` like every floating surface.

use crate::components::app::hover_open::use_hover_open;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_motion::anim::Anim;
use ds_motion::entrance::use_entrance;
use ds_motion::hover_intent::HoverProfile;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};

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

/// Whether the pill has been pressed since the pointer came over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Copied {
    No,
    Yes,
}

/// The link's destination. `href` is what a press hands to `oncopy`.
#[component]
pub fn LinkPill(
    target: LinkTarget,
    href: String,
    oncopy: EventHandler<String>,
    #[props(default)] common: Common,
) -> Element {
    let presence = use_entrance(Anim::PaletteFade).slug();
    let hover = use_hover_open(HoverProfile::Label);
    let mut copied = use_signal(|| Copied::No);
    let expanded = hover.shown();
    let (truth, words) = match (&target, copied()) {
        (_, Copied::Yes) => (
            truth(&target),
            rsx! {
                span { "Copied" }
            },
        ),
        (
            LinkTarget::Honest {
                scheme_sub,
                registered,
                path,
            },
            _,
        ) => (
            "honest",
            rsx! {
                if expanded == Shown::Visible {
                    span { class: "ds-link-pill-dim", "{scheme_sub}" }
                }
                b { "{registered}" }
                if expanded == Shown::Visible {
                    span { class: "ds-link-pill-dim", "data-part": "path", "{path}" }
                }
            },
        ),
        (LinkTarget::Lying { registered, shown }, _) => (
            "lying",
            rsx! {
                Glyph { icon: Icon::X, size: IconSize::Small }
                span {
                    "Goes to "
                    b { "{registered}" }
                    if expanded == Shown::Visible {
                        ", not {shown}"
                    }
                }
            },
        ),
    };
    let class = common.class("ds-link-pill");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "status",
            "data-truth": truth,
            "data-presence": presence,
            "data-expanded": (expanded == Shown::Visible).then_some("true"),
            "aria-label": common.aria_label.clone(),
            title: "Copy link",
            onmounted: move |event| common.mounted(event),
            onpointerenter: move |_| hover.over(),
            onpointerleave: move |_| {
                hover.out();
                copied.set(Copied::No);
            },
            onclick: {
                let href = href.clone();
                move |_| {
                    copied.set(Copied::Yes);
                    oncopy.call(href.clone());
                }
            },
            ..data,
            {words}
        }
    }
}

/// The `data-truth` word of `target`.
fn truth(target: &LinkTarget) -> &'static str {
    match target {
        LinkTarget::Honest { .. } => "honest",
        LinkTarget::Lying { .. } => "lying",
    }
}

#[cfg(test)]
mod tests {
    use super::{LinkTarget, truth};

    const CSS: &str = include_str!("link_pill.css");

    #[test]
    fn a_target_is_honest_or_lying() {
        let honest = LinkTarget::Honest {
            scheme_sub: "https://www.".to_string(),
            registered: "acme.example".to_string(),
            path: "/a".to_string(),
        };
        let lying = LinkTarget::Lying {
            registered: "g00gle.xyz".to_string(),
            shown: "google.com".to_string(),
        };
        assert_eq!(truth(&honest), "honest");
        assert_eq!(truth(&lying), "lying");
    }

    #[test]
    fn the_pill_fades_in_and_answers_the_pointer() {
        for rule in [
            "animation:fade var(--t-quick) var(--e-out);",
            "cursor:pointer;",
            "pointer-events:auto;",
        ] {
            assert!(CSS.contains(rule), "link_pill.css lacks {rule}");
        }
    }

    #[test]
    fn the_pill_is_ink_on_paper_and_danger_when_it_lies() {
        // design/04-COMPONENTS.md section 29: `--paper` on `--ink`; lying `--danger` with S's
        // `#fff` as `--danger-ink`.
        for rule in [
            ".ds-link-pill{",
            "background:var(--ink); color:var(--paper);",
            ".ds-link-pill b{ color:var(--paper);",
            ".ds-link-pill[*|data-truth=lying]{ background:var(--danger); color:var(--danger-ink); }",
        ] {
            assert!(CSS.contains(rule), "link_pill.css lacks {rule}");
        }
    }
}
