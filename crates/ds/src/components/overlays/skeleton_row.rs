//! SkeletonRow: the placeholder of one list row while the list is still loading (design/30
//! section 2.9): an avatar-sized `Circle` and one or two `Line`s, at the size of a settings row: its height
//! and avatar are the row tokens `--row-settings-h` and `--row-avatar` that `row.css` reads too,
//! and the title bar is 62% of the text column, the line under it 38% (the preset's proportions,
//! named in design/30 section 2.9). Static, no shimmer,
//! like every skeleton; the whole row fades in over `--t-quick` and, hidden, fades out and runs
//! `on_hidden` once settled, exactly as a [`Skeleton`](super::skeleton::Skeleton) does.
//!
//! Markup: `div.ds-skeleton-row[data-lines][data-presence]` of a `span.ds-skeleton` circle and
//! `div.ds-skeleton-row-lines` holding the line `span.ds-skeleton`s. It is decorative
//! (`aria-hidden`) unless `common.aria_label` names it.

use super::skeleton::SkeletonShape;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};

/// How many lines of text a row stands in for: `data-lines`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SkeletonLines {
    /// A title only.
    One,
    /// A title and the shorter line under it.
    #[default]
    Two,
}

/// A placeholder row: an avatar circle and `lines` bars. `shown` is the caller's (`Visible` by
/// default): hidden, it fades out over `--t-quick` and `on_hidden` runs once it has settled.
#[component]
pub fn SkeletonRow(
    #[props(default)] lines: SkeletonLines,
    #[props(default = Shown::Visible)] shown: Shown,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] common: Common,
) -> Element {
    let Presented { presence, alias } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::PaletteFade,
            exit: Exit::Fade,
        },
        on_hidden,
    );
    let Some(drawn) = presence.drawn_slug() else {
        return rsx! {};
    };
    let class = common.class("ds-skeleton-row");
    let data = common.data_attributes();
    let named = common.aria_label.clone();
    let decorative = named.is_none().then_some("true");
    rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-lines": lines.slug(),
            "data-presence": drawn,
            "data-pulse": alias.slug(),
            "aria-hidden": decorative,
            "aria-label": named,
            onmounted: move |event| common.mounted(event),
            ..data,
            span { class: "ds-skeleton", "data-shape": SkeletonShape::Circle.slug() }
            div { class: "ds-skeleton-row-lines",
                span { class: "ds-skeleton", "data-shape": SkeletonShape::Line.slug() }
                if lines == SkeletonLines::Two {
                    span { class: "ds-skeleton", "data-shape": SkeletonShape::Line.slug() }
                }
            }
        }
    }
}
