//! What an icon slot shows: a quire glyph, an icon quire does not draw (a status item's pixmap,
//! an icon theme's file, a third party's symbolic SVG, design/08-ICONS.md section 1.5), drawn
//! recoloured to the text colour (symbolic) or as it is (an image), or a status glyph.

use crate::components::content::status::family::StatusState;
use crate::style::icon::Icon;
use crate::style::icon::render::IconSize;
use crate::style::icon::url::IconUrl;

/// An external icon and the size it is drawn at, in the glyph scale.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalIcon {
    /// Where its pixels come from.
    pub url: IconUrl,
    /// How big it is drawn (a square).
    pub size: IconSize,
}

/// What an icon slot shows: one of quire's glyphs, or an external icon.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IconSource {
    /// A quire glyph, stroked in the text colour.
    Glyph(Icon),
    /// An external icon used as a mask: only its alpha counts, painted in the text colour, so
    /// it follows `--ink`, hover and `--f-ink*` exactly as a glyph does (a freedesktop
    /// `*-symbolic` icon, or a monochrome tray pixmap).
    Symbolic(ExternalIcon),
    /// An external icon drawn as it is: a coloured tray icon states a fact about its app.
    Image(ExternalIcon),
    /// A layered status glyph (Wi-Fi, battery, Bluetooth, volume) in its state, drawn by
    /// [`StatusGlyph`](crate::components::content::status::family::StatusGlyph) in the text colour at the slot's size, playing its
    /// own moments as the state changes (design/26-DETAILS.md 5.1). Hand the
    /// slot the state every render; put its `words()` in the slot's label (R8).
    Status(StatusState),
}

impl From<StatusState> for IconSource {
    fn from(state: StatusState) -> Self {
        IconSource::Status(state)
    }
}

impl From<Icon> for IconSource {
    fn from(icon: Icon) -> Self {
        IconSource::Glyph(icon)
    }
}
