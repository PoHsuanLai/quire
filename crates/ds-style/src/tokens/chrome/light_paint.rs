//! The traffic lights' own colours, as macOS (Sonoma, Sequoia) draws them: they are not label
//! hues. Per light and scheme there are three: the disc, the slightly darker hairline rim macOS
//! draws round it, and the ink of the mark (x, minus, arrows) shown while the pointer is over the
//! group. The table is written as `--light-close`, `--light-close-rim`, `--light-close-mark` and
//! so on, and `window_frame.css` reads those; a compositor asks [`light_colours`].
//!
//! The values were observed on macOS Sonoma. The mark inks are the opaque colour of the
//! translucent dark mark over its disc.

use super::light_hue::Light;
use crate::appearance::theme::Scheme;
use crate::tokens::hex::Hex;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// The three colours of one light in one scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightColours {
    /// The disc.
    pub disc: Hex,
    /// The hairline rim round the disc.
    pub rim: Hex,
    /// The mark's ink.
    pub mark: Hex,
}

/// The disc, rim and mark ink of `light` in `scheme`. The dark scheme keeps the light one's
/// numbers: macOS draws the same discs on a dark titlebar.
pub const fn light_colours(light: Light, _scheme: Scheme) -> LightColours {
    match light {
        Light::Close => LightColours {
            disc: Hex([0xFF, 0x5F, 0x57]),
            rim: Hex([0xE2, 0x46, 0x3F]),
            mark: Hex([0x4D, 0x00, 0x00]),
        },
        Light::Minimise => LightColours {
            disc: Hex([0xFE, 0xBC, 0x2E]),
            rim: Hex([0xE1, 0xA1, 0x16]),
            mark: Hex([0x99, 0x57, 0x00]),
        },
        Light::Zoom => LightColours {
            disc: Hex([0x28, 0xC8, 0x40]),
            rim: Hex([0x12, 0xAC, 0x28]),
            mark: Hex([0x00, 0x65, 0x00]),
        },
    }
}

/// One colour of the traffic lights, as a custom property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "light-", kind = fixed, css = light_paint_css)]
#[non_exhaustive]
pub enum LightPaint {
    /// `--light-close`: the close disc.
    Close,
    /// `--light-close-rim`: its rim.
    CloseRim,
    /// `--light-close-mark`: its mark's ink.
    CloseMark,
    /// `--light-minimize`: the minimise disc.
    #[token(name = "minimize")]
    Minimise,
    /// `--light-minimize-rim`: its rim.
    #[token(name = "minimize-rim")]
    MinimiseRim,
    /// `--light-minimize-mark`: its mark's ink.
    #[token(name = "minimize-mark")]
    MinimiseMark,
    /// `--light-zoom`: the zoom disc.
    Zoom,
    /// `--light-zoom-rim`: its rim.
    ZoomRim,
    /// `--light-zoom-mark`: its mark's ink.
    ZoomMark,
}

/// Which of a light's three colours a [`LightPaint`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightPart {
    /// The disc.
    Disc,
    /// The rim.
    Rim,
    /// The mark's ink.
    Mark,
}

impl LightPaint {
    /// The token for `part` of `light`.
    pub const fn of(light: Light, part: LightPart) -> LightPaint {
        match (light, part) {
            (Light::Close, LightPart::Disc) => LightPaint::Close,
            (Light::Close, LightPart::Rim) => LightPaint::CloseRim,
            (Light::Close, LightPart::Mark) => LightPaint::CloseMark,
            (Light::Minimise, LightPart::Disc) => LightPaint::Minimise,
            (Light::Minimise, LightPart::Rim) => LightPaint::MinimiseRim,
            (Light::Minimise, LightPart::Mark) => LightPaint::MinimiseMark,
            (Light::Zoom, LightPart::Disc) => LightPaint::Zoom,
            (Light::Zoom, LightPart::Rim) => LightPaint::ZoomRim,
            (Light::Zoom, LightPart::Mark) => LightPaint::ZoomMark,
        }
    }

    /// The light and part this token is.
    const fn parts(self) -> (Light, LightPart) {
        match self {
            LightPaint::Close => (Light::Close, LightPart::Disc),
            LightPaint::CloseRim => (Light::Close, LightPart::Rim),
            LightPaint::CloseMark => (Light::Close, LightPart::Mark),
            LightPaint::Minimise => (Light::Minimise, LightPart::Disc),
            LightPaint::MinimiseRim => (Light::Minimise, LightPart::Rim),
            LightPaint::MinimiseMark => (Light::Minimise, LightPart::Mark),
            LightPaint::Zoom => (Light::Zoom, LightPart::Disc),
            LightPaint::ZoomRim => (Light::Zoom, LightPart::Rim),
            LightPaint::ZoomMark => (Light::Zoom, LightPart::Mark),
        }
    }

    /// The colour in `scheme`.
    pub const fn hex(self, scheme: Scheme) -> Hex {
        let (light, part) = self.parts();
        let colours = light_colours(light, scheme);
        match part {
            LightPart::Disc => colours.disc,
            LightPart::Rim => colours.rim,
            LightPart::Mark => colours.mark,
        }
    }
}

/// A light colour as the stylesheet writes it, in the scope's scheme.
fn light_paint_css(token: LightPaint, scope: TokenScope) -> CssValue {
    CssValue::computed(token.hex(scope.scheme).css())
}

#[cfg(test)]
mod tests {
    use super::{LightPaint, LightPart};
    use crate::appearance::theme::Scheme;
    use crate::tokens::chrome::light_hue::Light;
    use crate::tokens::token::{Token, TokenScope};
    use ds_core::word::Word;

    #[test]
    fn each_token_writes_its_property_and_the_macos_value() {
        const CASES: &[(LightPaint, &str, &str)] = &[
            (LightPaint::Close, "--light-close", "#ff5f57"),
            (LightPaint::CloseRim, "--light-close-rim", "#e2463f"),
            (LightPaint::CloseMark, "--light-close-mark", "#4d0000"),
            (LightPaint::Minimise, "--light-minimize", "#febc2e"),
            (LightPaint::MinimiseRim, "--light-minimize-rim", "#e1a116"),
            (LightPaint::MinimiseMark, "--light-minimize-mark", "#995700"),
            (LightPaint::Zoom, "--light-zoom", "#28c840"),
            (LightPaint::ZoomRim, "--light-zoom-rim", "#12ac28"),
            (LightPaint::ZoomMark, "--light-zoom-mark", "#006500"),
        ];
        for (token, var, value) in CASES {
            assert_eq!(token.var().as_str(), *var, "{token:?} property");
            for scheme in [Scheme::Light, Scheme::Dark] {
                let scope = TokenScope::BASE.in_scheme(scheme);
                assert_eq!(
                    token.css_value(scope).as_str(),
                    *value,
                    "{token:?} {scheme:?}"
                );
            }
        }
        assert_eq!(CASES.len(), LightPaint::ALL.len(), "every token has a row");
    }

    #[test]
    fn of_and_parts_are_inverse() {
        for light in Light::ALL.iter().copied() {
            for part in [LightPart::Disc, LightPart::Rim, LightPart::Mark] {
                assert_eq!(LightPaint::of(light, part).parts(), (light, part));
            }
        }
    }
}
