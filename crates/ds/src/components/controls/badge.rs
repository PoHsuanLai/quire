//! Badge: the Dock and app badge (design/30 section 2.9): a capsule with a count, or a dot when
//! there is nothing to count. `Alert` is the red capsule of an unread count; `Quiet` is the
//! neutral capsule of a list's trailing count. More than 999 reads `999+`; zero draws nothing.
//! Markup: `span.ds-badge[data-tone][data-size]` holding `span.ds-badge-label` (no label on a dot).

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::tokens::control_size::ControlSize;

/// The most a badge counts before it says so.
const MOST: u32 = 999;

/// How loud the badge is, `data-tone`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum BadgeTone {
    /// A red capsule that asks for attention: unread mail, a missed call.
    #[default]
    Alert,
    /// A neutral capsule that only counts: the items in a list.
    Quiet,
}

/// What the badge shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BadgeContent {
    /// A count; zero draws no badge.
    Number(u32),
    /// A dot: something is new, and there is no number to give.
    Dot,
}

/// The text of a count: the number, or `999+` past the most.
fn label(count: u32) -> String {
    match count {
        0 => String::new(),
        n if n > MOST => format!("{MOST}+"),
        n => n.to_string(),
    }
}

/// A badge. `size` is one of Mini, Small and Regular; a Large badge draws as Regular.
#[component]
pub fn Badge(
    content: BadgeContent,
    #[props(default)] tone: BadgeTone,
    #[props(default)] size: ControlSize,
    #[props(default)] common: Common,
) -> Element {
    let text = match content {
        BadgeContent::Number(0) => return rsx! {},
        BadgeContent::Number(count) => Some(label(count)),
        BadgeContent::Dot => None,
    };
    let content_word = if text.is_some() { "number" } else { "dot" };
    let class = common.class("ds-badge");
    let data = common.data_attributes();
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-tone": tone.slug(),
            "data-size": size.slug(),
            "data-content": content_word,
            "aria-label": common.aria_label.clone(),
            onmounted: move |event| common.mounted(event),
            ..data,
            if let Some(text) = text {
                span { class: "ds-badge-label", "{text}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::label;

    #[test]
    fn a_count_reads_as_itself_up_to_999() {
        const CASES: &[(u32, &str)] = &[
            (0, ""),
            (1, "1"),
            (42, "42"),
            (999, "999"),
            (1000, "999+"),
            (u32::MAX, "999+"),
        ];
        for &(count, want) in CASES {
            assert_eq!(label(count), want, "{count}");
        }
    }
}
