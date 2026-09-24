//! One `box-shadow` layer of a material's stack, written two ways: with its default alpha (what
//! [`super::recipe`] reports and the legibility tests read) and with its alpha read from the
//! stack's tuning input (what the stylesheet declares), so a settings key moves it live.

use crate::tokens::hex::thousandths;
use crate::tokens::{Alpha, Hex, VarName};

/// Drawn inside the box or outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    /// `inset`: an edge or a highlight.
    Inner,
    /// Outside: a hairline or a shadow.
    Outer,
}

/// Where a layer's alpha comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayerAlpha {
    /// Fixed.
    Fixed(Alpha),
    /// The input's value, with this default (the highlight and the hairline).
    Input(VarName, Alpha),
    /// This alpha times the input, whose default is 1 (the shadows' strength).
    Scaled(Alpha, VarName),
}

/// One layer: `inset 0 1px 0 rgba(255,255,255,.3)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Layer {
    pub side: Side,
    /// Offsets, blur and spread: `0 1px 0`.
    pub geometry: &'static str,
    pub colour: Hex,
    pub alpha: LayerAlpha,
}

impl Layer {
    /// The layer at its default alpha.
    pub fn css(self) -> String {
        let alpha = match self.alpha {
            LayerAlpha::Fixed(alpha)
            | LayerAlpha::Input(_, alpha)
            | LayerAlpha::Scaled(alpha, _) => alpha.css(),
        };
        self.write(&alpha)
    }

    /// The layer with its alpha read from its input, as the stylesheet declares it.
    pub fn tuned_css(self) -> String {
        let alpha = match self.alpha {
            LayerAlpha::Fixed(alpha) => alpha.css(),
            LayerAlpha::Input(input, alpha) => format!("var({},{})", input.as_str(), alpha.css()),
            LayerAlpha::Scaled(alpha, input) => {
                format!("calc({}*var({},1))", alpha.css(), input.as_str())
            }
        };
        self.write(&alpha)
    }

    fn write(self, alpha: &str) -> String {
        let Hex([r, g, b]) = self.colour;
        let inset = match self.side {
            Side::Inner => "inset ",
            Side::Outer => "",
        };
        format!("{inset}{} rgba({r},{g},{b},{alpha})", self.geometry)
    }
}

/// A list of layers as one `box-shadow` value, or `none`.
pub(crate) fn joined(layers: &[Layer], each: impl Fn(Layer) -> String) -> String {
    if layers.is_empty() {
        "none".to_owned()
    } else {
        layers
            .iter()
            .map(|layer| each(*layer))
            .collect::<Vec<_>>()
            .join(",")
    }
}

/// A fraction as CSS: `.3`.
pub(crate) fn fraction(alpha: Alpha) -> String {
    thousandths(i64::from(alpha.0))
}

#[cfg(test)]
mod tests {
    use super::{Layer, LayerAlpha, Side};
    use crate::tokens::{Alpha, Hex, VarName};

    #[test]
    fn a_layer_writes_its_default_and_its_tuned_form() {
        let highlight = Layer {
            side: Side::Inner,
            geometry: "0 1px 0",
            colour: Hex([255, 255, 255]),
            alpha: LayerAlpha::Input(VarName("--m-highlight-light"), Alpha(300)),
        };
        assert_eq!(highlight.css(), "inset 0 1px 0 rgba(255,255,255,.3)");
        assert_eq!(
            highlight.tuned_css(),
            "inset 0 1px 0 rgba(255,255,255,var(--m-highlight-light,.3))"
        );
        let drop = Layer {
            side: Side::Outer,
            geometry: "0 1px 2px",
            colour: Hex([0, 0, 0]),
            alpha: LayerAlpha::Scaled(Alpha(100), VarName("--m-shadow-strength")),
        };
        assert_eq!(drop.css(), "0 1px 2px rgba(0,0,0,.1)");
        assert_eq!(
            drop.tuned_css(),
            "0 1px 2px rgba(0,0,0,calc(.1*var(--m-shadow-strength,1)))"
        );
    }
}
