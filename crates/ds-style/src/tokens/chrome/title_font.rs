//! The font of a window's title, as numbers: what `window_frame.css` sets on `.ds-titlebar`
//! (`font-size:var(--fs-control); font-weight:600`) for a compositor that draws the title itself.

use crate::tokens::token::{Token, TokenScope};
use crate::tokens::type_scale::{Family, FontSize};

/// The weight the titlebar sets its title in.
const TITLE_WEIGHT: u16 = 600;

/// A title's font: the family name its face registers as, a size in px and a weight.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct TitleFont {
    /// The family name the UI stack leads with: `Inter`.
    pub family: String,
    /// The size in logical pixels: `--fs-control`, 13.
    pub size_px: f32,
    /// The weight: 600.
    pub weight: u16,
}

/// The window title's font.
///
/// ```
/// use ds_style::tokens::chrome::title_font::title_font;
/// let font = title_font();
/// assert_eq!((font.size_px, font.weight), (13.0, 600));
/// ```
pub fn title_font() -> TitleFont {
    let size = FontSize::Control.css_value(TokenScope::BASE);
    TitleFont {
        family: Family::Ui.face_name(),
        size_px: size.as_str().trim_end_matches("px").parse().unwrap_or(13.0),
        weight: TITLE_WEIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::title_font;

    const CSS: &str = include_str!("../../../../ds/src/components/chrome/window_frame.css");

    #[test]
    fn the_title_is_control_size_at_600_and_the_css_says_so() {
        let font = title_font();
        assert_eq!(font.family, "Inter");
        assert_eq!((font.size_px, font.weight), (13.0, 600));
        assert!(CSS.contains("font-size:var(--fs-control); font-weight:600;"));
    }
}
