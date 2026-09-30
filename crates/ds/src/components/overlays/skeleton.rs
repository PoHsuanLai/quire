//! Skeleton: a grey placeholder shape shown in place of content that is still loading (design/30
//! section 2.9, SwiftUI's `redacted`). It is kept although macOS has no counterpart, and moves the
//! macOS way: a static fill, no shimmer, and a cross-fade over `--t-quick` when the content
//! arrives (the caller hides it; it fades out before it is gone).
//!
//! A skeleton is shown while an `Operation` without a value is running; the caller says so with
//! `shown`. Three shapes (`SkeletonShape`): a `Line` of text, a `Block` (an image, a card) and a
//! `Circle` (an avatar).

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};

/// What a skeleton stands in for: `data-shape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SkeletonShape {
    /// A line of text: a rounded bar one line tall.
    #[default]
    Line,
    /// A block: an image or a card, with the card radius.
    Block,
    /// A circle: an avatar, `width` across.
    Circle,
}

/// The inline size: what the caller gave, in pixels. A line's height is the line's own, a circle
/// is as tall as it is wide, and a bare skeleton fills the width it is put in.
fn size_style(shape: SkeletonShape, width: Option<Px>, height: Option<Px>) -> Option<String> {
    let height = match shape {
        SkeletonShape::Circle => width,
        SkeletonShape::Line | SkeletonShape::Block => height,
    };
    match (width, height) {
        (None, None) => None,
        (Some(width), None) => Some(format!("width:{}px", width.0)),
        (None, Some(height)) => Some(format!("height:{}px", height.0)),
        (Some(width), Some(height)) => Some(format!("width:{}px;height:{}px", width.0, height.0)),
    }
}

/// A placeholder shape. `shown` is the caller's (`Visible` by default): hidden, it fades out over
/// `--t-quick` and `on_hidden` runs once it has settled, so the caller can drop it. `common` puts
/// the consumer's `id`, `data-*` and classes on it; it is decorative (`aria-hidden`) unless
/// `aria_label` names it.
#[component]
pub fn Skeleton(
    #[props(default)] shape: SkeletonShape,
    #[props(default)] width: Option<Px>,
    #[props(default)] height: Option<Px>,
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
    let class = common.class("ds-skeleton");
    let data = common.data_attributes();
    let named = common.aria_label.clone();
    let decorative = named.is_none().then_some("true");
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-shape": shape.slug(),
            "data-presence": drawn,
            "data-pulse": alias.slug(),
            "aria-hidden": decorative,
            "aria-label": named,
            style: size_style(shape, width, height),
            onmounted: move |event| common.mounted(event),
            ..data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SkeletonShape, size_style};
    use ds_core::geometry::units::Px;

    #[test]
    fn a_circle_is_as_tall_as_it_is_wide() {
        // (shape, width, height, style)
        let cases = [
            (SkeletonShape::Line, None, None, None),
            (SkeletonShape::Line, Some(120.0), None, Some("width:120px")),
            (
                SkeletonShape::Block,
                Some(200.0),
                Some(80.0),
                Some("width:200px;height:80px"),
            ),
            (
                SkeletonShape::Circle,
                Some(32.0),
                Some(99.0),
                Some("width:32px;height:32px"),
            ),
        ];
        for (shape, width, height, want) in cases {
            let got = size_style(shape, width.map(Px), height.map(Px));
            assert_eq!(got.as_deref(), want, "{shape:?}");
        }
    }
}
