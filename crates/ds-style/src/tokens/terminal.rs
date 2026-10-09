//! The terminal's palette (a terminal app draws its own cells, so it reads these as data): the
//! sixteen ANSI colours, the default foreground and background, the cursor and the selection,
//! for each scheme.
//!
//! Foreground and background are the card's ink and surface in the scheme, so a terminal sits
//! in a Space's card like any other content (design/21-SPACES.md: the card follows the
//! scheme, the Space paints the frame around it). The cursor and the selection follow the
//! accent, as a text caret and a selected row do. Every ANSI colour reads on the background at
//! 4.5:1 or better, so the "black" of a dark scheme and the "white" of a light one are lifted
//! or deepened from the usual: `ansi_colours_read_on_the_background` gates all sixteen in both.

use crate::appearance::{accent::Accent, theme::Scheme};
use crate::tokens::accent_table::accent_of;
use crate::tokens::hex::{Colour, Hex};
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

const fn rgb(value: u32) -> Colour {
    Colour::Solid(Hex([(value >> 16) as u8, (value >> 8) as u8, value as u8]))
}

/// One colour of the terminal palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "term-", kind = fixed, css = terminal_css)]
#[non_exhaustive]
pub enum TerminalColour {
    /// `--term-fg`: the default foreground.
    Fg,
    /// `--term-bg`: the default background.
    Bg,
    /// `--term-cursor`: the cursor block, the accent's text colour.
    Cursor,
    /// `--term-selection`: the wash behind selected cells, the accent's own.
    Selection,
    /// `--term-ansi-0`: black.
    #[token(name = "ansi-0")]
    Black,
    /// `--term-ansi-1`: red.
    #[token(name = "ansi-1")]
    Red,
    /// `--term-ansi-2`: green.
    #[token(name = "ansi-2")]
    Green,
    /// `--term-ansi-3`: yellow.
    #[token(name = "ansi-3")]
    Yellow,
    /// `--term-ansi-4`: blue.
    #[token(name = "ansi-4")]
    Blue,
    /// `--term-ansi-5`: magenta.
    #[token(name = "ansi-5")]
    Magenta,
    /// `--term-ansi-6`: cyan.
    #[token(name = "ansi-6")]
    Cyan,
    /// `--term-ansi-7`: white.
    #[token(name = "ansi-7")]
    White,
    /// `--term-ansi-8`: bright black.
    #[token(name = "ansi-8")]
    BrightBlack,
    /// `--term-ansi-9`: bright red.
    #[token(name = "ansi-9")]
    BrightRed,
    /// `--term-ansi-10`: bright green.
    #[token(name = "ansi-10")]
    BrightGreen,
    /// `--term-ansi-11`: bright yellow.
    #[token(name = "ansi-11")]
    BrightYellow,
    /// `--term-ansi-12`: bright blue.
    #[token(name = "ansi-12")]
    BrightBlue,
    /// `--term-ansi-13`: bright magenta.
    #[token(name = "ansi-13")]
    BrightMagenta,
    /// `--term-ansi-14`: bright cyan.
    #[token(name = "ansi-14")]
    BrightCyan,
    /// `--term-ansi-15`: bright white.
    #[token(name = "ansi-15")]
    BrightWhite,
}

/// A measure of the terminal's cell grid that is the same in every scheme and typeface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "term-", kind = fixed)]
#[non_exhaustive]
pub enum TerminalMetric {
    /// `--term-font`: the system's own monospace family, for a terminal's cells. Space Mono
    /// (`--font-code`) has no box drawing, CJK or emoji; the system's monospace covers them and
    /// falls back per glyph. No shipped face leads this stack, on purpose.
    #[token(name = "font", value = "monospace")]
    Font,
    /// `--term-lh`: a row's height as a multiple of the font size (`--term-font`); rows this close
    /// let box drawing join from one to the next.
    #[token(name = "lh", value = "1.2")]
    LineHeight,
}

impl TerminalColour {
    /// The sixteen ANSI colours, in index order: 0 to 7, then their bright forms 8 to 15.
    pub const ANSI: [TerminalColour; 16] = [
        TerminalColour::Black,
        TerminalColour::Red,
        TerminalColour::Green,
        TerminalColour::Yellow,
        TerminalColour::Blue,
        TerminalColour::Magenta,
        TerminalColour::Cyan,
        TerminalColour::White,
        TerminalColour::BrightBlack,
        TerminalColour::BrightRed,
        TerminalColour::BrightGreen,
        TerminalColour::BrightYellow,
        TerminalColour::BrightBlue,
        TerminalColour::BrightMagenta,
        TerminalColour::BrightCyan,
        TerminalColour::BrightWhite,
    ];

    /// The colour in `scheme` with `accent` as the accent the cursor and selection follow.
    pub fn value(self, scheme: Scheme, accent: Accent) -> Colour {
        let roles = accent_of(accent, scheme);
        let (light, dark) = match self {
            TerminalColour::Cursor => return Colour::Solid(roles.text),
            TerminalColour::Selection => return roles.wash_colour(),
            other => other.fixed(),
        };
        match scheme {
            Scheme::Light => light,
            Scheme::Dark => dark,
        }
    }

    /// The colours that do not follow the accent, `(light, dark)`.
    fn fixed(self) -> (Colour, Colour) {
        match self {
            // The card's ink and surface (`ColourToken::Ink`, `Surface`), the dark ground being
            // the window paper's, which a terminal sits deepest in.
            TerminalColour::Fg => (rgb(0x202020), rgb(0xE8E8E8)),
            TerminalColour::Bg => (rgb(0xFFFFFF), rgb(0x1E1E1E)),
            TerminalColour::Black => (rgb(0x303030), rgb(0x8A8A8A)),
            TerminalColour::Red => (rgb(0xB3261E), rgb(0xF0786C)),
            TerminalColour::Green => (rgb(0x1B7A3E), rgb(0x5EC27F)),
            TerminalColour::Yellow => (rgb(0x8A6100), rgb(0xD9B04A)),
            TerminalColour::Blue => (rgb(0x1F5FBF), rgb(0x6EA3F2)),
            TerminalColour::Magenta => (rgb(0xA23B9C), rgb(0xD98BD3)),
            TerminalColour::Cyan => (rgb(0x0B7A85), rgb(0x4FC3CF)),
            TerminalColour::White => (rgb(0x5C5C5C), rgb(0xC8C8C8)),
            TerminalColour::BrightBlack => (rgb(0x6B6B6B), rgb(0x9A9A9A)),
            TerminalColour::BrightRed => (rgb(0xC7382D), rgb(0xFF8F84)),
            TerminalColour::BrightGreen => (rgb(0x187A3F), rgb(0x7ADB98)),
            TerminalColour::BrightYellow => (rgb(0x946A00), rgb(0xEBC866)),
            TerminalColour::BrightBlue => (rgb(0x2F6FD1), rgb(0x8DB8F8)),
            TerminalColour::BrightMagenta => (rgb(0xB14AAA), rgb(0xE8A3E2)),
            TerminalColour::BrightCyan => (rgb(0x0A7983), rgb(0x6FD6E0)),
            TerminalColour::BrightWhite => (rgb(0x4A4A4A), rgb(0xF5F5F5)),
            TerminalColour::Cursor | TerminalColour::Selection => (rgb(0x000000), rgb(0x000000)),
        }
    }
}

/// A terminal token as the stylesheet writes it: the cursor and the selection name the accent's
/// own variables, so they follow whichever accent a root selects; the rest is the table's value.
fn terminal_css(token: TerminalColour, scope: TokenScope) -> CssValue {
    match token {
        TerminalColour::Cursor => CssValue::fixed("var(--accent-text)"),
        TerminalColour::Selection => CssValue::fixed("var(--accent-soft)"),
        other => CssValue::computed(other.value(scope.scheme, Accent::Blue).css()),
    }
}

#[cfg(test)]
mod tests {
    use super::{TerminalColour, TerminalMetric};
    use crate::appearance::{accent::Accent, theme::Scheme};
    use crate::tokens::hex::Hex;
    use crate::tokens::token::{Token, TokenScope};
    use ds_core::colour::contrast::ratio;
    use ds_core::word::Word;

    fn css(token: TerminalColour, scheme: Scheme) -> String {
        token.value(scheme, Accent::Blue).css()
    }

    #[test]
    fn the_foreground_reads_on_the_background() {
        for scheme in Scheme::ALL.iter().copied() {
            let (fg, bg) = (
                css(TerminalColour::Fg, scheme),
                css(TerminalColour::Bg, scheme),
            );
            let got = ratio(&fg, &bg).expect("hex pair");
            assert!(got >= 4.5, "{scheme:?}: fg {fg} on bg {bg} is {got:.2}");
        }
    }

    #[test]
    fn ansi_colours_read_on_the_background() {
        let mut failures = Vec::new();
        for scheme in Scheme::ALL.iter().copied() {
            let bg = css(TerminalColour::Bg, scheme);
            for (index, token) in TerminalColour::ANSI.iter().copied().enumerate() {
                let colour = css(token, scheme);
                let got = ratio(&colour, &bg).expect("hex pair");
                if got < 4.5 {
                    failures.push(format!(
                        "{scheme:?} ansi {index} {colour} on {bg}: {got:.2}"
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn the_cursor_and_the_foreground_over_the_selection_read_for_every_accent() {
        let mut failures = Vec::new();
        for scheme in Scheme::ALL.iter().copied() {
            let bg = Hex::parse(&css(TerminalColour::Bg, scheme)).expect("hex");
            for accent in Accent::ALL.iter().copied() {
                let cursor = TerminalColour::Cursor.value(scheme, accent).css();
                let wash = match TerminalColour::Selection.value(scheme, accent) {
                    crate::tokens::hex::Colour::Alpha(fill, alpha) => fill.over(alpha, bg).css(),
                    crate::tokens::hex::Colour::Solid(fill) => fill.css(),
                };
                let fg = css(TerminalColour::Fg, scheme);
                for (what, fore, back, floor) in [
                    ("cursor on bg", cursor, bg.css(), 3.0),
                    ("fg on selection", fg, wash, 4.5),
                ] {
                    let got = ratio(&fore, &back).expect("hex pair");
                    if got < floor {
                        failures.push(format!("{scheme:?} {accent:?} {what}: {got:.2}"));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }

    #[test]
    fn the_terminal_face_is_the_system_monospace_not_space_mono() {
        let face = TerminalMetric::Font.css_value(TokenScope::BASE);
        assert_eq!(TerminalMetric::Font.var().as_str(), "--term-font");
        assert_eq!(face.as_str(), "monospace");
    }

    #[test]
    fn a_terminal_row_is_one_point_two_times_its_font() {
        assert_eq!(TerminalMetric::LineHeight.var().as_str(), "--term-lh");
        assert_eq!(
            TerminalMetric::LineHeight
                .css_value(TokenScope::BASE)
                .as_str(),
            "1.2"
        );
    }

    #[test]
    fn the_cursor_and_selection_name_the_accent_and_the_rest_are_hex() {
        let scope = TokenScope::BASE;
        assert_eq!(
            TerminalColour::Cursor.css_value(scope).as_str(),
            "var(--accent-text)"
        );
        assert_eq!(
            TerminalColour::Selection.css_value(scope).as_str(),
            "var(--accent-soft)"
        );
        assert_eq!(TerminalColour::Black.var().as_str(), "--term-ansi-0");
        assert_eq!(TerminalColour::BrightWhite.var().as_str(), "--term-ansi-15");
        for token in TerminalColour::ALL.iter().copied() {
            let dark = token.css_value(scope.in_scheme(Scheme::Dark));
            assert!(
                dark.as_str().starts_with('#') || dark.as_str().starts_with("var("),
                "{token:?}: {}",
                dark.as_str()
            );
        }
    }
}
