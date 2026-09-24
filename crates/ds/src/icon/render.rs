//! Drawing a glyph: `svg.ds-ic` with the stroke as attributes and `currentColor`, so the glyph
//! takes its parent's text colour (design/08-ICONS.md sections 1.3-1.5).
//!
//! Moved from mailo's `Glyph` (`mail-app/src/ui/icon/mod.rs`): `class` became `size`, and the
//! stroke moved from CSS `.ic` onto the element.

use super::Icon;
use super::shape::Shape;
use super::stroke::stroke_width;
use crate::root::use_scale;
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
    /// magnification, an icon a settings key sizes (sill FINDINGS Q16).
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

fn child(shape: &Shape) -> Element {
    match shape {
        Shape::Path(d) => rsx! { path { d: "{d}" } },
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
/// `data-size`), stroked in `currentColor` through attributes, never CSS (spike S6). At a
/// fractional device scale the stroke is snapped to whole device pixels (`super::stroke`).
#[component]
pub fn Glyph(icon: Icon, #[props(default)] size: IconSize) -> Element {
    let px = size.px();
    let stroke = stroke_width(size, use_scale());
    rsx! {
        svg {
            class: "ds-ic",
            "data-size": "{px}",
            // Presentation attributes: Blitz and browsers map an `svg`'s width and height to
            // the CSS properties, so the size needs no stylesheet rule (design/08-ICONS.md 1.4).
            width: "{px}",
            height: "{px}",
            view_box: "0 0 24 24",
            // SVG elements do not carry the HTML `aria_hidden` attribute.
            "aria-hidden": "true",
            "stroke": "currentColor",
            "stroke-width": stroke,
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": "none",
            for shape in icon.shapes() {
                {child(shape)}
            }
        }
    }
}
