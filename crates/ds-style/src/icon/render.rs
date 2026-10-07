//! Drawing a glyph: `svg.ds-ic` with the paint as attributes and `currentColor`, so the glyph
//! takes its parent's text colour (design/08-ICONS.md sections 1.3-1.5). A solid glyph is
//! `fill="currentColor"` with no stroke; an outline one is the 2 px stroke with no fill.
//!
//! The paint is written on the element, not by CSS.

#[cfg(feature = "dioxus")]
use super::Icon;
#[cfg(feature = "dioxus")]
use super::posed::Thousandths;
#[cfg(feature = "dioxus")]
use super::shape::Shape;
#[cfg(feature = "dioxus")]
use super::slash::Cut;
#[cfg(feature = "dioxus")]
use super::stroke::stroke_width;
#[cfg(feature = "dioxus")]
use super::style::GlyphStyle;
#[cfg(feature = "dioxus")]
use crate::scale::use_scale;
#[cfg(feature = "dioxus")]
use dioxus::core::current_scope_id;
#[cfg(feature = "dioxus")]
use dioxus::prelude::*;

/// How big a glyph is drawn. Consumers pick a size; none writes an icon `width` in CSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconSize {
    /// 11 px: clip in a row, stop remove, person-chip remove.
    Micro,
    /// 12 px.
    Tiny,
    /// 13 px: Today close, toast tab, link pill.
    Small,
    /// 14 px: mini, star, strip, button.
    Compact,
    /// 15 px: sidebar item, foot button, search.
    Nav,
    /// 16 px: the base.
    #[default]
    Base,
    /// 17 px: a rich menu tile.
    Tile,
    /// 18 px: the all-accounts tile, image pickers.
    Large,
    /// 22 px: bar status items and control-center tiles (proposed).
    Bar,
    /// 48 px: a dock tile at rest (design/10-BEHAVIOUR-dock.md, `dock.tile_size_px`'s default).
    Tile48,
    /// 96 px: a dock tile at full magnification.
    Tile96,
    /// Any other size a caller resolved for itself: a dock tile between rest and full
    /// magnification, an icon a settings key sizes.
    Px(IconPx),
}

/// An icon's side in whole logical pixels, for [`IconSize::Px`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IconPx(pub u8);

impl IconSize {
    /// The logical size in pixels, written as `data-size`.
    pub fn px(self) -> u8 {
        match self {
            IconSize::Micro => 11,
            IconSize::Tiny => 12,
            IconSize::Small => 13,
            IconSize::Compact => 14,
            IconSize::Nav => 15,
            IconSize::Base => 16,
            IconSize::Tile => 17,
            IconSize::Large => 18,
            IconSize::Bar => 22,
            IconSize::Tile48 => 48,
            IconSize::Tile96 => 96,
            IconSize::Px(IconPx(side)) => side,
        }
    }
}

#[cfg(feature = "dioxus")]
pub(super) fn shape_element(shape: &Shape) -> Element {
    match shape {
        Shape::Path(d) => rsx! { path { d: "{d}" } },
        Shape::Solid(d) => rsx! { path { d: "{d}", fill: "currentColor" } },
        Shape::Circle { cx, cy, r } => rsx! { circle { cx: "{cx}", cy: "{cy}", r: "{r}" } },
        Shape::Rect {
            x,
            y,
            width,
            height,
            rx,
        } => rsx! {
            rect {
                x: "{x}",
                y: "{y}",
                width: "{width}",
                height: "{height}",
                rx: "{rx}",
            }
        },
    }
}

/// `icon`, drawn as an `svg` of class `ds-ic` at `size` (its `width`, `height` and
/// `data-size`) in `style` (solid unless a pair's off state says outline), painted in
/// `currentColor` through attributes, never CSS (spike S6). An outline's stroke is snapped to
/// whole device pixels at a fractional device scale (`super::stroke`). `cut` is how much of a
/// slash is drawn across it: the gap that slash leaves is cut out of the glyph (`super::slash`).
#[cfg(feature = "dioxus")]
#[component]
pub fn Glyph(
    icon: Icon,
    #[props(default)] size: IconSize,
    #[props(default)] style: GlyphStyle,
    #[props(default)] cut: Thousandths,
) -> Element {
    let px = size.px();
    let mask = use_hook(|| format!("ds-cut-{}", current_scope_id().0));
    let stroke = stroke_width(size, use_scale());
    let (paint, outline) = match style {
        GlyphStyle::Solid => (("none", "currentColor"), None),
        GlyphStyle::Outline => (("currentColor", "none"), Some(stroke)),
    };
    let round = outline.as_ref().map(|_| "round");
    rsx! {
        svg {
            class: "ds-ic",
            "data-size": "{px}",
            "data-style": match style {
                GlyphStyle::Solid => "solid",
                GlyphStyle::Outline => "outline",
            },
            // Presentation attributes: Blitz and browsers map an `svg`'s width and height to
            // the CSS properties, so the size needs no stylesheet rule (design/08-ICONS.md 1.4).
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            // SVG elements do not carry the HTML `aria_hidden` attribute.
            "aria-hidden": "true",
            "stroke": paint.0,
            "stroke-width": outline,
            "stroke-linecap": round,
            "stroke-linejoin": round,
            "fill": paint.1,
            Cut { id: mask, drawn: cut,
                for shape in icon.shapes_in(style) {
                    {shape_element(shape)}
                }
            }
        }
    }
}
