//! IconView: any icon slot's content, a quire glyph or an external icon (design/08-ICONS.md
//! section 1.5). A glyph draws as `Glyph` does; an external icon is a square `span`:
//! `mask-image` over `currentColor` when symbolic (spike S7), `background-image` when an image
//! (spike S8). Both URLs load through the document's net provider, one frame late.

use crate::icon::external::{ExternalIcon, IconSource};
use crate::icon::family::PlateFamily;
use crate::icon::plate_tint::{PlateTint, tint_style};
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
///
/// With a `plate`, the icon sits on an app-icon plate `size` square (design/08-ICONS.md
/// sections 2.1-2.5 and 4.1): the `n = 5` superellipse in the family's 135 degree gradient,
/// with the inner highlight, the rim and a drop shadow. A glyph or a symbolic icon is drawn in
/// the family's glyph colour at `--plate-glyph` (56 %) of the plate, an image at
/// `--plate-inset` (72 %). The dock's placeholder tile until the generated icons arrive.
///
/// With a `plate_tint` (`PlateTint::of(style, tint)`, the pair a consumer hands `retint`), the
/// plate's stops and glyph colour are re-coloured by the same rule as the icon's raster, so a
/// Muted or Monochrome dock is one hue, plate included (sill FINDINGS Q72). Without one, or
/// without a plate, it changes nothing.
#[component]
pub fn IconView(
    source: IconSource,
    #[props(default)] size: IconSize,
    #[props(default)] plate: Option<PlateFamily>,
    #[props(default)] plate_tint: Option<PlateTint>,
) -> Element {
    match plate {
        Some(family) => rsx! {
            span {
                class: "ds-plate",
                "data-family": family.slug(),
                "data-icon-style": plate_tint.map(PlateTint::slug),
                "data-size": "{size.px()}",
                style: plate_style(size, family, plate_tint),
                span { class: "ds-plate-face", "aria-hidden": "true" }
                {bare(source, size)}
            }
        },
        None => bare(source, size),
    }
}

/// The plate's inline custom properties: its size, and its tinted paint when it has one.
fn plate_style(size: IconSize, family: PlateFamily, tint: Option<PlateTint>) -> String {
    let side = format!("--ic-size:{}px", size.px());
    match tint {
        Some(tint) => format!("{side};{}", tint_style(family, tint)),
        None => side,
    }
}

/// The icon alone, as a slot draws it.
fn bare(source: IconSource, size: IconSize) -> Element {
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
