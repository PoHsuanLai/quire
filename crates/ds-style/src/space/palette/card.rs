//! The card's own Post colours, which a Space's hue never moves (design/03-COLOR.md
//! section 3). Split from `palette.rs` to keep it under 400 lines.

use crate::appearance::theme::Scheme;

/// The card's own colours: Post, which a Space's hue never moves. the chosen accent is not here:
/// it is generated like every accent (`tokens::accent_of`, design/03-COLOR.md section 20).
///
/// The same literals as `tokens.css` and `tokens.dark.css`; `ui::style`'s tests hold the two
/// together. Here so the editor's contrast readout can measure against the card without
/// reading a stylesheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    /// A pane: the list and the reader.
    pub surface: &'static str,
    /// Body text on it.
    pub ink: &'static str,
}

/// The light card.
pub const POST_LIGHT: Card = Card {
    surface: "#F8F9F6",
    ink: "#1A1E1A",
};

/// The dark card.
pub const POST_DARK: Card = Card {
    surface: "#1D211B",
    ink: "#E7EBE3",
};

/// The card for a scheme.
pub fn card(scheme: Scheme) -> Card {
    match scheme {
        Scheme::Light => POST_LIGHT,
        Scheme::Dark => POST_DARK,
    }
}
