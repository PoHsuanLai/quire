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

/// What a desktop widget's card is tinted with (design/23-WIDGETS.md section 4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CardTint {
    /// The `Widget` material's own tint.
    #[default]
    Material,
    /// The material with the Space's gradient over it at the material's frame alpha, as the
    /// tinted chrome carries it (design/03-COLOR.md section 17.2), so the Space's colour reaches
    /// the card.
    Space,
}

impl CardTint {
    /// `data-tint`: written only for a Space tint.
    pub fn slug(self) -> Option<&'static str> {
        match self {
            CardTint::Material => None,
            CardTint::Space => Some("space"),
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
