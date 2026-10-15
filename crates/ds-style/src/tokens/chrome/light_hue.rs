//! The traffic lights' colours: the disc and the ink of its mark, read from the same table the
//! stylesheet's `--c-red`, `--c-amber`, `--c-green` (and their `-deep` and `-soft`) are written
//! from (`label_hue.rs`). `window_frame.css` paints the disc with the base and the mark with the
//! deep (light) or soft (dark) member; [`light_hue`] names that choice for a compositor.

use crate::appearance::theme::Scheme;
use crate::tokens::hex::Hex;
use crate::tokens::label_hue::{HueColour, HueMember, LabelHue};
use ds_core::word::Word;

/// One of the three traffic lights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
#[non_exhaustive]
pub enum Light {
    /// Close, red.
    Close,
    /// Minimise, amber.
    Minimise,
    /// Zoom, green.
    Zoom,
}

impl Light {
    /// The label hue its disc is drawn in.
    pub const fn hue(self) -> LabelHue {
        match self {
            Light::Close => LabelHue::Red,
            Light::Minimise => LabelHue::Amber,
            Light::Zoom => LabelHue::Green,
        }
    }

    /// The tokens the stylesheet reads for the disc and the mark's ink in `scheme`:
    /// `background:var(--c-red)` and `color:var(--c-red-deep)` (light) or `-soft` (dark).
    pub fn tokens(self, scheme: Scheme) -> (HueColour, HueColour) {
        let ink = match scheme {
            Scheme::Light => HueMember::Deep,
            Scheme::Dark => HueMember::Soft,
        };
        (self.hue().colour(HueMember::Base), self.hue().colour(ink))
    }
}

/// The disc colour and the mark's ink for `light` in `scheme`, the numbers behind the lights'
/// CSS.
///
/// ```
/// use ds_style::appearance::theme::Scheme;
/// use ds_style::tokens::chrome::light_hue::{Light, light_hue};
/// let (disc, _ink) = light_hue(Light::Close, Scheme::Light);
/// assert_eq!(disc.css(), "#e8483c");
/// ```
pub fn light_hue(light: Light, scheme: Scheme) -> (Hex, Hex) {
    let hue = light.hue();
    let (disc, ink) = light.tokens(scheme);
    (
        hue.value(disc.parts().1, scheme),
        hue.value(ink.parts().1, scheme),
    )
}

#[cfg(test)]
mod tests {
    use super::{Light, light_hue};
    use crate::appearance::theme::Scheme;
    use ds_core::word::Word;

    const CSS: &str = include_str!("../../../../ds/src/components/chrome/window_frame.css");

    #[test]
    fn the_lights_are_the_candy_hues_in_both_schemes() {
        const CASES: &[(Light, Scheme, &str, &str)] = &[
            (Light::Close, Scheme::Light, "#e8483c", "#b7352b"),
            (Light::Close, Scheme::Dark, "#ff6b60", "#3a1e1c"),
            (Light::Minimise, Scheme::Light, "#f0a81e", "#8e5a05"),
            (Light::Minimise, Scheme::Dark, "#f5bc4e", "#3a2c12"),
            (Light::Zoom, Scheme::Light, "#28b24a", "#1a7a33"),
            (Light::Zoom, Scheme::Dark, "#4fd06a", "#153520"),
        ];
        for (light, scheme, disc, ink) in CASES {
            let (got_disc, got_ink) = light_hue(*light, *scheme);
            assert_eq!(
                (got_disc.css(), got_ink.css()),
                (disc.to_string(), ink.to_string()),
                "{light:?} {scheme:?}"
            );
        }
    }

    /// The window frame's stylesheet reads exactly the tokens `Light::tokens` names.
    #[test]
    fn the_window_frame_css_reads_the_same_tokens() {
        let word = |light: Light| match light {
            Light::Close => "close",
            Light::Minimise => "minimize",
            Light::Zoom => "zoom",
        };
        for light in Light::ALL.iter().copied() {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let (disc, ink) = light.tokens(scheme);
                let prefix = match scheme {
                    Scheme::Light => ".ds-light",
                    Scheme::Dark => ".ds[*|data-theme=dark] .ds-light",
                };
                let ink_rule = format!(
                    "{prefix}[*|data-light={}]{{ color:var({}); }}",
                    word(light),
                    ink.var().as_str()
                );
                assert!(CSS.contains(&ink_rule), "{ink_rule}");
                let disc_rule = format!("background:var({}); }}", disc.var().as_str());
                assert!(CSS.contains(&disc_rule), "{disc_rule}");
            }
        }
    }
}
