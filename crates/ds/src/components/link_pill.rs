//! LinkPill: the real destination of a link, instant, and loud when the text lies
//! (design/04-COMPONENTS.md section 29, design/06-INTERACTIONS.md section 13). The honest/lying
//! decision is mailo's. The consumer mounts it in the reader the moment the pointer is over a
//! link and drops it on leave: no intent delay, no exit. It enters with `Anim::LinkPillIn`
//! (`hc-in` at `--t-quick --e-out`, `S:433`), reported as `data-presence` like every entrance.

use crate::components::popover::use_entrance;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::motion::anim::Anim;
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
    let presence = use_entrance(Anim::LinkPillIn).slug();
    match target {
        // The registered domain never truncates: the path gives way first (O-30).
        LinkTarget::Honest {
            scheme_sub,
            registered,
            path,
        } => rsx! {
            div { class: "ds-link-pill", "data-truth": "honest", "data-presence": presence, role: "status",
                span { class: "ds-link-pill-dim", "{scheme_sub}" }
                b { "{registered}" }
                span { class: "ds-link-pill-dim", "data-part": "path", "{path}" }
            }
        },
        LinkTarget::Lying { registered, shown } => rsx! {
            div { class: "ds-link-pill", "data-truth": "lying", "data-presence": presence, role: "status",
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

#[cfg(test)]
mod tests {
    use crate::motion::anim::Anim;

    const CSS: &str = include_str!("link_pill.css");

    #[test]
    fn the_pill_plays_its_own_recipe() {
        let recipe = Anim::LinkPillIn.recipe();
        let want = format!(
            "animation:{} {} {};",
            recipe.keyframes,
            recipe.duration.var().reference(),
            recipe.easing.var().reference()
        );
        assert!(CSS.contains(&want), "link_pill.css does not play {want}");
        let hover_card = Anim::HcIn.recipe();
        assert_ne!(
            (hover_card.duration, hover_card.easing),
            (recipe.duration, recipe.easing),
            "the pill must not borrow the hover card's recipe"
        );
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
