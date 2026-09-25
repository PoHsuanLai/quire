//! ModulePanel: a control-center module that holds content of its own (the Sound and Display
//! levels, Now Playing, Appearance, Battery) on a `ModuleTile`'s frame, so a consumer draws no
//! plate of its own (sill FINDINGS Q100; design/13-BEHAVIOUR-menus-windows.md section 13.3.7).
//!
//! A tile is a toggle with a glyph, a title and a status; a panel is not pressable at all: its
//! content (a `LevelControl`, a picker, buttons) takes every press, so the panel is a plain
//! `div` with no role. It spans the whole grid row by default, as a slider module does.

use crate::components::module_tile_kind::TileSpan;
use crate::components::text_runs::{Text, text};
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// Whether the panel paints the tile's plate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PanelPlate {
    /// The tile's plate: `--surface-2`, a hairline, `--r-tile`, the module's padding.
    #[default]
    Tile,
    /// No plate and no padding: the module where something else already is its plate (a bar
    /// item's dropdown, whose popover card is the module's).
    Bare,
}

impl PanelPlate {
    /// The `data-plate` word.
    fn slug(self) -> &'static str {
        match self {
            PanelPlate::Tile => "tile",
            PanelPlate::Bare => "bare",
        }
    }
}

/// A module with content: an optional header row (`glyph`, `title`, then `trailing` at the far
/// end, such as a level's percentage) over `children`. The header is drawn when any of the three
/// is given.
#[component]
pub fn ModulePanel(
    #[props(default)] glyph: Option<Icon>,
    #[props(default)] title: Option<Text>,
    #[props(default)] trailing: Option<Element>,
    #[props(default = TileSpan::Full)] span: TileSpan,
    #[props(default)] plate: PanelPlate,
    children: Element,
) -> Element {
    let head = header(glyph, title, trailing);
    rsx! {
        div { class: "ds-module-panel", "data-span": span.slug(), "data-plate": plate.slug(),
            {head}
            {children}
        }
    }
}

/// The header row, or nothing when the panel has neither glyph, title nor trailing slot.
fn header(glyph: Option<Icon>, title: Option<Text>, trailing: Option<Element>) -> Element {
    if glyph.is_none() && title.is_none() && trailing.is_none() {
        return rsx! {};
    }
    rsx! {
        div { class: "ds-module-panel-head",
            if let Some(icon) = glyph {
                span { class: "ds-module-panel-glyph",
                    Glyph { icon, size: IconSize::Base }
                }
            }
            span { class: "ds-module-panel-title",
                if let Some(title) = title {
                    {text(&title)}
                }
            }
            if let Some(trailing) = trailing {
                span { class: "ds-module-panel-trailing", {trailing} }
            }
        }
    }
}
