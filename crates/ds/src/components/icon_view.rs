//! IconView: any icon slot's content, a quire glyph or an external icon (design/08-ICONS.md
//! section 1.5). A glyph draws as `Glyph` does; an external icon is a square `span`:
//! `mask-image` over `currentColor` when symbolic (spike S7), `background-image` when an image
//! (spike S8). Both URLs load through the document's net provider, one frame late.

use crate::icon::external::{ExternalIcon, IconSource};
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// Which way an external icon is painted: the `data-kind` word and the property its URL is
/// written into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Paint {
    Mask,
    Picture,
}

impl Paint {
    fn slug(self) -> &'static str {
        match self {
            Paint::Mask => "symbolic",
            Paint::Picture => "image",
        }
    }

    fn property(self) -> &'static str {
        match self {
            Paint::Mask => "mask-image",
            Paint::Picture => "background-image",
        }
    }
}

/// `source` drawn in an icon slot. A glyph takes the slot's `size`; an external icon is drawn
/// at its own [`ExternalIcon::size`], the size its caller resolved it for.
#[component]
pub fn IconView(source: IconSource, #[props(default)] size: IconSize) -> Element {
    match source {
        IconSource::Glyph(icon) => rsx! {
            Glyph { icon, size }
        },
        IconSource::Symbolic(external) => external_icon(&external, Paint::Mask),
        IconSource::Image(external) => external_icon(&external, Paint::Picture),
    }
}

/// The inline style of an external icon: its size as `--ic-size` and its URL in the painting
/// property. The URL is an `IconUrl`, so it cannot break out of the quoted string.
pub(crate) fn external_style(external: &ExternalIcon, paint_property: &str) -> String {
    format!(
        "--ic-size:{}px;{paint_property}:url(\"{}\")",
        external.size.px(),
        external.url.as_str()
    )
}

fn external_icon(external: &ExternalIcon, paint: Paint) -> Element {
    let style = external_style(external, paint.property());
    rsx! {
        span {
            class: "ds-ext-icon",
            "data-kind": paint.slug(),
            "data-size": "{external.size.px()}",
            "aria-hidden": "true",
            style,
        }
    }
}

/// Marks the two conversions below, so they do not collide with dioxus's own.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphSlot;

/// `Button { icon: Icon::Send }` keeps compiling now that the slot takes an [`IconSource`].
impl dioxus::core::SuperFrom<crate::icon::Icon, GlyphSlot> for Option<IconSource> {
    fn super_from(icon: crate::icon::Icon) -> Self {
        Some(IconSource::Glyph(icon))
    }
}

/// `Button { icon: Some(Icon::Archive) }` keeps compiling too.
impl dioxus::core::SuperFrom<Option<crate::icon::Icon>, GlyphSlot> for Option<IconSource> {
    fn super_from(icon: Option<crate::icon::Icon>) -> Self {
        icon.map(IconSource::Glyph)
    }
}
