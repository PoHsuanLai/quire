//! The shadows made of plain layers, as numbers: the one table both the stylesheet's
//! `--shadow-window` and `--shadow-drag` and a compositor that draws its own window shadow read.
//!
//! A layer is a drop in logical pixels and an sRGB colour at an alpha in thousandths. The CSS is
//! written from the layers ([`layers_css`]), so the two cannot drift.

use crate::appearance::theme::Scheme;
use crate::tokens::elevation::Shadow;
use crate::tokens::hex::{Alpha, Hex};

/// One layer of a shadow: `dx dy blur spread colour`, as CSS `box-shadow` reads them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ShadowLayer {
    /// Horizontal offset in px.
    pub dx: i16,
    /// Vertical offset in px.
    pub dy: i16,
    /// Blur radius in px.
    pub blur: u16,
    /// Spread in px; negative shrinks the shadow.
    pub spread: i16,
    /// The colour.
    pub rgb: Hex,
    /// Its opacity in thousandths (`.12` is 120).
    pub alpha: Alpha,
}

impl ShadowLayer {
    /// A layer; `alpha` in thousandths.
    pub const fn new(
        (dx, dy): (i16, i16),
        blur: u16,
        spread: i16,
        rgb: [u8; 3],
        alpha: u16,
    ) -> ShadowLayer {
        ShadowLayer {
            dx,
            dy,
            blur,
            spread,
            rgb: Hex(rgb),
            alpha: Alpha(alpha),
        }
    }

    /// The colour as 8-bit `[r, g, b, a]`, the alpha rounded from thousandths.
    pub fn rgba(self) -> [u8; 4] {
        let [r, g, b] = self.rgb.0;
        let alpha = (u32::from(self.alpha.0.min(1000)) * 255 + 500) / 1000;
        [r, g, b, u8::try_from(alpha).unwrap_or(u8::MAX)]
    }

    /// The layer as CSS: `0 24px 64px -16px rgba(0,0,0,.4)`; a zero spread is left out.
    pub fn css(self) -> String {
        let length = |n: i32| {
            if n == 0 {
                "0".to_owned()
            } else {
                format!("{n}px")
            }
        };
        let spread = match self.spread {
            0 => String::new(),
            n => format!(" {}", length(i32::from(n))),
        };
        let [r, g, b] = self.rgb.0;
        format!(
            "{} {} {}{spread} rgba({r},{g},{b},{})",
            length(i32::from(self.dx)),
            length(i32::from(self.dy)),
            length(i32::from(self.blur)),
            self.alpha.css()
        )
    }
}

/// Layers as one `box-shadow` value, joined by commas.
pub fn layers_css(layers: &[ShadowLayer]) -> String {
    layers
        .iter()
        .map(|layer| layer.css())
        .collect::<Vec<_>>()
        .join(",")
}

const WINDOW_LIGHT: &[ShadowLayer] = &[
    ShadowLayer::new((0, 1), 3, 0, [0, 0, 0], 120),
    ShadowLayer::new((0, 24), 64, -16, [0, 0, 0], 400),
];
const WINDOW_DARK: &[ShadowLayer] = &[
    ShadowLayer::new((0, 1), 3, 0, [0, 0, 0], 400),
    ShadowLayer::new((0, 24), 64, -16, [0, 0, 0], 660),
];
const DRAG_LIGHT: &[ShadowLayer] = &[ShadowLayer::new((0, 18), 30, -12, [26, 30, 26], 400)];
const DRAG_DARK: &[ShadowLayer] = &[ShadowLayer::new((0, 22), 34, -14, [0, 0, 0], 800)];

impl Shadow {
    /// The shadow's layers in `scheme`, outermost last, for a shadow that is a plain list of
    /// drops: [`Shadow::Window`] and [`Shadow::Drag`]. Their CSS is written from these. Every
    /// other shadow reads a custom property (`--hair`, `--hairline`, `--f-pill`) and stays
    /// CSS-only: the slice is empty.
    ///
    /// ```
    /// use ds_style::appearance::theme::Scheme;
    /// use ds_style::tokens::elevation::Shadow;
    /// let layers = Shadow::Window.layers(Scheme::Light);
    /// assert_eq!(layers.len(), 2);
    /// assert_eq!(layers[1].blur, 64);
    /// ```
    pub fn layers(self, scheme: Scheme) -> &'static [ShadowLayer] {
        match (self, scheme) {
            (Shadow::Window, Scheme::Light) => WINDOW_LIGHT,
            (Shadow::Window, Scheme::Dark) => WINDOW_DARK,
            (Shadow::Drag, Scheme::Light) => DRAG_LIGHT,
            (Shadow::Drag, Scheme::Dark) => DRAG_DARK,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::layers_css;
    use crate::appearance::theme::Scheme;
    use crate::tokens::elevation::Shadow;
    use crate::tokens::token::{Token, TokenScope};

    /// Today's golden: the stylesheet snapshot declares these exact strings.
    const GOLDEN: &[(Shadow, Scheme, &str)] = &[
        (
            Shadow::Window,
            Scheme::Light,
            "0 1px 3px rgba(0,0,0,.12),0 24px 64px -16px rgba(0,0,0,.4)",
        ),
        (
            Shadow::Window,
            Scheme::Dark,
            "0 1px 3px rgba(0,0,0,.4),0 24px 64px -16px rgba(0,0,0,.66)",
        ),
        (
            Shadow::Drag,
            Scheme::Light,
            "0 18px 30px -12px rgba(26,30,26,.4)",
        ),
        (
            Shadow::Drag,
            Scheme::Dark,
            "0 22px 34px -14px rgba(0,0,0,.8)",
        ),
    ];

    const SNAPSHOT: &str = include_str!("../../../ds-shell/tests/snapshots/stylesheet.css");

    #[test]
    fn the_layers_write_the_css_the_stylesheet_had() {
        for (shadow, scheme, css) in GOLDEN {
            assert_eq!(layers_css(shadow.layers(*scheme)), *css, "{shadow:?}");
            let scope = TokenScope::BASE.in_scheme(*scheme);
            assert_eq!(shadow.css_value(scope).as_str(), *css, "{shadow:?}");
        }
    }

    #[test]
    fn the_stylesheet_snapshot_declares_the_generated_strings() {
        for (shadow, _, css) in GOLDEN {
            let declaration = format!("{}:{css};", shadow.var().as_str());
            assert!(SNAPSHOT.contains(&declaration), "{declaration}");
        }
    }

    #[test]
    fn a_layer_rounds_its_alpha_to_a_byte() {
        let layers = Shadow::Window.layers(Scheme::Light);
        assert_eq!(layers[0].rgba(), [0, 0, 0, 31]);
        assert_eq!(layers[1].rgba(), [0, 0, 0, 102]);
    }

    #[test]
    fn a_shadow_with_a_custom_property_has_no_layers() {
        assert!(Shadow::Pop.layers(Scheme::Light).is_empty());
    }
}
