//! The words a `WidgetFrame` is described in (design/22-SETTINGS.md section 3.20; sill FINDINGS
//! Q182): its footprint on the grid, where it is drawn, and its title row.

use crate::components::text_runs::Text;
use crate::icon::Icon;

/// A widget's footprint on the grid unit (`widgets.desktop_cell_px`, `desktop_gap_px`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WidgetSize {
    /// One cell.
    #[default]
    Small,
    /// Two cells wide, one tall.
    Medium,
    /// Two cells wide, two tall.
    Large,
}

impl WidgetSize {
    /// The `data-size` word.
    pub fn slug(self) -> &'static str {
        match self {
            WidgetSize::Small => "small",
            WidgetSize::Medium => "medium",
            WidgetSize::Large => "large",
        }
    }
}

/// Where a widget is drawn, which decides what its card is made of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WidgetHost {
    /// On the desktop layer: a card of the `Widget` material, over the wallpaper.
    #[default]
    Desktop,
    /// In the notification center's widget column: a flatter tile on the Popover it sits in,
    /// with no material of its own (a second material inside the center would draw a second
    /// edge and drop).
    Tile,
}

impl WidgetHost {
    /// The `data-host` word.
    pub fn slug(self) -> &'static str {
        match self {
            WidgetHost::Desktop => "desktop",
            WidgetHost::Tile => "tile",
        }
    }
}

/// A widget's title row: a glyph and a name ("Batteries", a calendar's month).
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetTitle {
    /// The glyph before the name.
    pub glyph: Icon,
    /// The name.
    pub text: Text,
}

impl WidgetTitle {
    /// A title of `glyph` and `text`.
    pub fn new(glyph: Icon, text: impl Into<Text>) -> Self {
        WidgetTitle {
            glyph,
            text: text.into(),
        }
    }
}
