//! The traffic lights and their colours: the disc and the ink of its mark, from the table of
//! macOS's own light colours (`light_paint.rs`), which is also what the stylesheet's
//! `--light-close`, `--light-minimize`, `--light-zoom` (with `-rim` and `-mark`) are written from.
//! `window_frame.css` paints the disc, rim and mark with those; [`light_hue`] and [`light_rim`]
//! give a compositor the same numbers.

use super::light_paint::light_colours;
use crate::appearance::theme::Scheme;
use crate::tokens::hex::Hex;
use ds_core::word::Word;

/// One of the three traffic lights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
#[non_exhaustive]
pub enum Light {
    /// Close, red.
    Close,
    /// Minimise, yellow.
    Minimise,
    /// Zoom, green.
    Zoom,
}

/// The disc colour and the mark's ink for `light` in `scheme`, the numbers behind the lights'
/// CSS.
///
/// ```
/// use ds_style::appearance::theme::Scheme;
/// use ds_style::tokens::chrome::light_hue::{Light, light_hue};
/// let (disc, _ink) = light_hue(Light::Close, Scheme::Light);
/// assert_eq!(disc.css(), "#ff5f57");
/// ```
pub fn light_hue(light: Light, scheme: Scheme) -> (Hex, Hex) {
    let colours = light_colours(light, scheme);
    (colours.disc, colours.mark)
}

/// The hairline rim round `light`'s disc in `scheme`.
///
/// ```
/// use ds_style::appearance::theme::Scheme;
/// use ds_style::tokens::chrome::light_hue::{Light, light_rim};
/// assert_eq!(light_rim(Light::Zoom, Scheme::Dark).css(), "#12ac28");
/// ```
pub fn light_rim(light: Light, scheme: Scheme) -> Hex {
    light_colours(light, scheme).rim
}

#[cfg(test)]
mod tests {
    use super::{Light, light_hue, light_rim};
    use crate::appearance::theme::Scheme;
    use crate::tokens::chrome::light_paint::{LightPaint, LightPart};
    use ds_core::word::Word;

    const CSS: &str = include_str!("../../../../ds/src/components/chrome/window_frame.css");

    #[test]
    fn the_lights_are_the_macos_colours_in_both_schemes() {
        const CASES: &[(Light, &str, &str, &str)] = &[
            (Light::Close, "#ff5f57", "#4d0000", "#e2463f"),
            (Light::Minimise, "#febc2e", "#995700", "#e1a116"),
            (Light::Zoom, "#28c840", "#006500", "#12ac28"),
        ];
        for (light, disc, ink, rim) in CASES {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let (got_disc, got_ink) = light_hue(*light, scheme);
                assert_eq!(
                    (
                        got_disc.css(),
                        got_ink.css(),
                        light_rim(*light, scheme).css()
                    ),
                    (disc.to_string(), ink.to_string(), rim.to_string()),
                    "{light:?} {scheme:?}"
                );
            }
        }
    }

    /// The window frame's stylesheet reads exactly the tokens `LightPaint` names.
    #[test]
    fn the_window_frame_css_reads_the_same_tokens() {
        let word = |light: Light| match light {
            Light::Close => "close",
            Light::Minimise => "minimize",
            Light::Zoom => "zoom",
        };
        for light in Light::ALL.iter().copied() {
            let var = |part| LightPaint::of(light, part).var();
            let ink_rule = format!(
                ".ds-light[*|data-light={}]{{ color:var({}); }}",
                word(light),
                var(LightPart::Mark).as_str()
            );
            assert!(CSS.contains(&ink_rule), "{ink_rule}");
            let disc_rule = format!(
                "background:var({}); box-shadow:inset 0 0 0 var(--hair) var({}); }}",
                var(LightPart::Disc).as_str(),
                var(LightPart::Rim).as_str()
            );
            assert!(CSS.contains(&disc_rule), "{disc_rule}");
        }
    }
}
