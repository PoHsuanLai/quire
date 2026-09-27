//! ModulePanel: a control-center module that holds content of its own (the Sound and Display
//! levels, Now Playing, Appearance, Battery) on a `ModuleTile`'s frame, so a consumer draws no
//! plate of its own (sill FINDINGS Q100; design/13-BEHAVIOUR-menus-windows.md section 13.3.7).
//!
//! A tile is a toggle with a glyph, a title and a status; a panel is not pressable at all: its
//! content (a `LevelControl`, a picker, buttons) takes every press, so the panel is a plain
//! `div` with no role. It spans the whole grid row by default, as a slider module does.

use crate::components::icon_view::IconView;
use crate::components::module_tile_kind::TileSpan;
use crate::components::text_runs::{Text, text};
use crate::components::vocab::Availability;
use crate::detail::FirstShow;
use crate::icon::external::IconSource;
use crate::icon::render::IconSize;
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
///
/// `glyph` is an `Icon` (it converts) or any [`IconSource`]: `IconSource::Status` draws a layered
/// status glyph (the Battery module's, sill Q391). `first` is its first frame: pass
/// `FirstShow::Animate` when the control center was just opened, and the battery's fill sweeps in
/// from empty over `--t-sweep`. `availability` is its content's: a module whose level is disabled
/// passes `Availability::Disabled` and its header glyph and trailing figure dim with it (Q491).
#[component]
pub fn ModulePanel(
    #[props(default)] glyph: Option<IconSource>,
    #[props(default)] title: Option<Text>,
    #[props(default)] trailing: Option<Element>,
    #[props(default = TileSpan::Full)] span: TileSpan,
    #[props(default)] plate: PanelPlate,
    #[props(default)] first: FirstShow,
    #[props(default)] availability: Availability,
    children: Element,
) -> Element {
    let head = header(glyph, first, title, trailing);
    rsx! {
        div { class: "ds-module-panel", "data-span": span.slug(), "data-plate": plate.slug(),
            "aria-disabled": availability.aria_disabled(),
            {head}
            {children}
        }
    }
}

/// The header row, or nothing when the panel has neither glyph, title nor trailing slot.
fn header(
    glyph: Option<IconSource>,
    first: FirstShow,
    title: Option<Text>,
    trailing: Option<Element>,
) -> Element {
    if glyph.is_none() && title.is_none() && trailing.is_none() {
        return rsx! {};
    }
    rsx! {
        div { class: "ds-module-panel-head",
            if let Some(source) = glyph {
                span { class: "ds-module-panel-glyph",
                    IconView { source, size: IconSize::Base, first }
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
