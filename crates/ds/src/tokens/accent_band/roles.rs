//! What an accent paints: the roles one hue becomes.

use crate::tokens::hex::{Alpha, Colour, Hex};

/// The roles [`super::accent_roles`] returns for one hue in one scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccentRoles {
    /// The solid fill: the primary button, the today disc, a toggle that is on, the control
    /// center tile's disc, the slider's fill.
    pub fill: Hex,
    /// Text and glyphs on [`Self::fill`].
    pub ink: Hex,
    /// The accent as text or a thin mark on the card: a link, the month's title, a menu's check.
    /// It reads at 4.5:1 on the card's grounds and on the wash over them.
    pub text: Hex,
    /// The same on a translucent material (a menu, the launcher, a banner, PolkitPrompt): it
    /// reads at 4.5:1 over a black or a white backdrop too, and on the wash over those. A root
    /// of such a material paints `--accent-text` in it.
    pub text_material: Hex,
    /// The wash's alpha: [`Self::fill`] at this alpha over whatever lies beneath is a selected
    /// row, the menu and launcher highlight, a chip, a tile that is on.
    pub wash: Alpha,
    /// The focus ring's alpha: [`Self::text`] at this alpha.
    pub ring: Alpha,
}

impl AccentRoles {
    /// The wash as CSS paints it.
    pub fn wash_colour(&self) -> Colour {
        Colour::Alpha(self.fill, self.wash)
    }

    /// The focus ring as CSS paints it.
    pub fn ring_colour(&self) -> Colour {
        Colour::Alpha(self.text, self.ring)
    }
}
